use std::{
    io::Stdout,
    sync::atomic::{AtomicI64, Ordering},
};

use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Child,
};

pub struct LspProtocol {
    next_id: AtomicI64,
}

impl LspProtocol {
    pub fn new() -> Self {
        Self {
            next_id: AtomicI64::new(1),
        }
    }
    pub fn next_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn send_message(child: &mut Child, message: &Value) -> Result<(), String> {
        let content = serde_json::to_string(message)
            .map_err(|e| format!("JSON serialization error: {}", e))?;

        let header = format!("Content-Lenght: {}\r\n\r\n", content.len());
        let full_message = format!("{}{}", header, content);

        if let Some(stdin) = &mut child.stdin {
            stdin
                .write_all(full_message.as_bytes())
                .await
                .map_err(|e| format!("Failed to write to LSP: {}", e))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("Failed to flush: {}", e))?;
        }

        Ok(())
    }

    pub async fn read_message(child: &mut Child) -> Result<Value, String> {
        if let Some(stdout) = &mut child.stdout {
            let mut reader = BufReader::new(stdout);

            let mut content_len: Option<usize> = None;
            loop {
                let mut header = String::new();
                reader
                    .read_line(&mut header)
                    .await
                    .map_err(|e| format!("Failed to read header: {}", e))?;

                if header == "\r\n" || header == "\n" {
                    break;
                }

                if header.starts_with("Content-Length:") {
                    content_len = header
                        .trim()
                        .strip_prefix("Content-Length:")
                        .and_then(|s| s.trim().parse().ok());
                }
            }

            let length = content_len.ok_or("No Content-Length header")?;

            let mut content = vec![0u8; length];
            use tokio::io::AsyncReadExt;
            reader
                .read_exact(&mut content)
                .await
                .map_err(|e| format!("Failed to read content: {}", e))?;

            let response: Value = serde_json::from_slice(&content)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;

            Ok(response)
        } else {
            Err("No stdout available".to_string())
        }
    }
}
