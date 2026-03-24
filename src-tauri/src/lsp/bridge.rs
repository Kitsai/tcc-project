use std::sync::{Arc, Mutex};

use tokio::process::Child;

pub struct LspBridge {
    processes: Arc<Mutex<Vec<Child>>>,
    listener_port: Arc<Mutex<Option<u16>>>,
}

impl LspBridge {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(Vec::new())),
            listener_port: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for LspBridge {
    fn default() -> Self {
        Self::new()
    }
}
