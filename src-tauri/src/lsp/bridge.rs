use std::{
    collections::HashMap,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex, RwLock},
};

use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    process::{Child, ChildStdin, ChildStdout, Command},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::lsp::registry::LspRegistry;

pub struct LspBridge {
    registry: Arc<RwLock<LspRegistry>>,
    active_servers: Arc<Mutex<HashMap<String, LspServerInstance>>>,
}

struct LspServerInstance {
    port: u16,
    _process: Child,
}

impl LspBridge {
    pub fn new(registry: Arc<RwLock<LspRegistry>>) -> Self {
        Self {
            registry,
            active_servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_for_language(
        &self,
        language_id: &str,
        workspace_dir: String,
    ) -> Result<u16, String> {
        {
            let servers = self.active_servers.lock().map_err(|e| e.to_string())?;
            if let Some(instance) = servers.get(language_id) {
                return Ok(instance.port);
            }
        }

        let server = {
            let registry = self.registry.read().map_err(|e| e.to_string())?;
            registry
                .get_by_language(language_id)
                .ok_or(format!("No LSP server for: {}", language_id))?
        };

        let binary_path = server
            .get_binary_path()
            .await
            .ok_or(format!("Binary not found: {}", server.binary_name()))?;

        // 1. Create compile_flags.txt for clangd if needed
        if server.name() == "clangd" {
            let mut flags_path = PathBuf::from(&workspace_dir);
            flags_path.push("compile_flags.txt");
            
            let mut content = String::from("-std=c++17\n");
            for include in server.custom_includes() {
                // Normalize to forward slashes for better clangd compatibility on Windows
                let normalized = include.replace("\\", "/");
                content.push_str(&format!("-I{}\n", normalized));
            }
            
            if let Err(e) = std::fs::write(&flags_path, content) {
                eprintln!("Warning: Failed to create compile_flags.txt at {:?}: {}", flags_path, e);
            } else {
                println!("Created compile_flags.txt at {:?}", flags_path);
            }
        }

        let mut child = Command::new(&binary_path)
            .args(&server.args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) 
            .current_dir(&workspace_dir)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to spawn: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to take stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to take stdout")?;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("Failed to bind: {}", e))?;

        let port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get WS port: {}", e))?
            .port();

        println!("Starting {} on port {}", server.name(), port);

        let active_servers_clone = Arc::clone(&self.active_servers);
        let language_id_clone = language_id.to_string();

        tokio::spawn(async move {
            // Accept exactly one connection for this server instance
            if let Ok((stream, _)) = listener.accept().await {
                let _ = proxy_lsp_connection(stream, stdin, stdout).await;
            }

            // FORCE CLEANUP: remove from active servers and kill process
            if let Ok(mut servers) = active_servers_clone.lock() {
                if let Some(mut instance) = servers.remove(&language_id_clone) {
                    let _ = instance._process.start_kill();
                    println!(
                        "LSP server for {} killed and cleaned up",
                        language_id_clone
                    );
                }
            }
        });

        {
            let mut servers = self.active_servers.lock().map_err(|e| e.to_string())?;
            servers.insert(
                language_id.to_string(),
                LspServerInstance {
                    port,
                    _process: child,
                },
            );
        }

        Ok(port)
    }

    pub fn stop_all(&self) -> Result<(), String> {
        let mut servers = self.active_servers.lock().map_err(|e| e.to_string())?;
        for (_, mut instance) in servers.drain() {
            let _ = instance._process.start_kill();
        }
        Ok(())
    }
}

async fn proxy_lsp_connection(
    stream: TcpStream,
    mut lsp_stdin: ChildStdin,
    mut lsp_stdout: ChildStdout,
) -> Result<(), String> {
    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| format!("WS handshake failed: {}", e))?;

    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // Use select! on inline async blocks to handle bidirectional traffic without extra spawns
    tokio::select! {
        res = async {
            use tokio::io::AsyncWriteExt;
            while let Some(Ok(msg)) = ws_rx.next().await {
                let data = match msg {
                    Message::Binary(data) => data.to_vec(),
                    Message::Text(data) => data.as_bytes().to_vec(),
                    _ => continue,
                };
                
                // Add LSP Content-Length header
                let header = format!("Content-Length: {}\r\n\r\n", data.len());
                lsp_stdin.write_all(header.as_bytes()).await.map_err(|e| e.to_string())?;
                lsp_stdin.write_all(&data).await.map_err(|e| e.to_string())?;
                lsp_stdin.flush().await.map_err(|e| e.to_string())?;
            }
            Ok::<(), String>(())
        } => res,
        res = async {
            use tokio::io::AsyncReadExt;

            let mut buffer = Vec::new();
            let mut temp_buf = [0u8; 8192];

            loop {
                let n = lsp_stdout.read(&mut temp_buf).await.map_err(|e| e.to_string())?;
                if n == 0 { break; }
                buffer.extend_from_slice(&temp_buf[..n]);

                while !buffer.is_empty() {
                    let s = String::from_utf8_lossy(&buffer);
                    if let Some(content_length_pos) = s.find("Content-Length: ") {
                        // If there is junk before Content-Length, clear it
                        if content_length_pos > 0 {
                            buffer.drain(..content_length_pos);
                            continue;
                        }

                        if let Some(header_end) = s.find("\r\n\r\n") {
                            let start = header_end + 4;
                            let length_str = s[16..header_end].trim();

                            if let Ok(length) = length_str.parse::<usize>() {
                                if buffer.len() >= start + length {
                                    // Extract ONLY the JSON part (strip headers)
                                    let json_data = buffer.drain(start..start + length).collect::<Vec<_>>();
                                    // Remove the headers too
                                    buffer.drain(..start);

                                    let json_str = String::from_utf8_lossy(&json_data).to_string();
                                    if ws_tx.send(Message::Text(json_str.into())).await.is_err() {
                                        return Ok(()); // Client disconnected
                                    }
                                    continue;
                                }
                            }
                        }
                    }
                    // If we reach here, we don't have a full message yet. 
                    // Break the inner loop to read more data from the LSP.
                    break;
                }
            }
            Ok::<(), String>(())
        } => res,
    }
}
