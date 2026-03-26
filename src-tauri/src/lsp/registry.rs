use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::lsp::server::LspServer;

pub struct LspRegistry {
    servers: HashMap<String, Arc<dyn LspServer>>,
}

impl LspRegistry {
    /// register new LSP
    pub fn register(&mut self, server: Arc<dyn LspServer>) {
        let language_id = server.language_id().to_string();
        println!(
            "Registered LSP Server '{}' for language '{}'",
            server.name(),
            language_id
        );
        self.servers.insert(language_id, server);
    }

    pub fn get_by_language(&self, language_id: &str) -> Option<Arc<dyn LspServer>> {
        self.servers.get(language_id).cloned()
    }

    pub fn get_by_extension(&self, extension: &str) -> Option<Arc<dyn LspServer>> {
        self.servers
            .values()
            .find(|server| server.file_extensions().contains(&extension))
            .cloned()
    }

    pub fn get_all(&self) -> Vec<Arc<dyn LspServer>> {
        self.servers.values().cloned().collect()
    }

    pub fn get_language_id(&self, extension: &str) -> Option<String> {
        self.get_by_extension(extension)
            .map(|server| server.language_id().to_string())
    }

    pub fn supported_languages(&self) -> Vec<String> {
        self.servers.keys().cloned().collect()
    }
}

pub struct LspRegistryBuilder {
    servers: Vec<Arc<dyn LspServer>>,
}

impl LspRegistryBuilder {
    pub fn instance() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    pub fn with(&mut self, server: Arc<dyn LspServer>) -> &mut Self {
        self.servers.push(server);
        self
    }

    pub fn build(&self) -> Arc<RwLock<LspRegistry>> {
        let instance = Arc::new(RwLock::new(LspRegistry {
            servers: HashMap::new(),
        }));

        {
            let mut lock = instance.write().unwrap();

            for server in self.servers.iter().cloned() {
                lock.register(server);
            }
        }

        instance
    }
}
