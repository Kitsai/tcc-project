use std::{
    collections::HashMap,
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

        let mut child = Command::new(&binary_path)
            .args(&server.args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .current_dir(&workspace_dir)
            .kill_on_drop(true) // Ensure process dies when dropped or app exits
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
            
            // Cleanup: remove from active servers so it can be restarted on next request
            if let Ok(mut servers) = active_servers_clone.lock() {
                servers.remove(&language_id_clone);
                println!("LSP server for {} stopped and cleaned up", language_id_clone);
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
                match msg {
                    Message::Binary(data) => {
                        lsp_stdin.write_all(&data).await.map_err(|e| e.to_string())?;
                    }
                    Message::Text(data) => {
                        lsp_stdin.write_all(data.as_bytes()).await.map_err(|e| e.to_string())?;
                    }
                    _ => continue,
                }
                lsp_stdin.flush().await.map_err(|e| e.to_string())?;
            }
            Ok::<(), String>(())
        } => res,
        res = async {
            use tokio::io::AsyncReadExt;
            let mut buf = [0u8; 8192];
            loop {
                let n = lsp_stdout.read(&mut buf).await.map_err(|e| e.to_string())?;
                if n == 0 { break; }
                let msg = Message::Binary(buf[..n].to_vec().into());
                ws_tx.send(msg).await.map_err(|e| e.to_string())?;
            }
            Ok::<(), String>(())
        } => res,
    }
}
