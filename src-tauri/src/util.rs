use std::{fmt::Display, fs, path::Path};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

pub trait EventEmitter: Clone + Send + 'static {
    fn emit<S: Serialize + Clone + Send + 'static>(&self, event: &str, payload: S);
}

impl<R: tauri::Runtime> EventEmitter for AppHandle<R> {
    fn emit<S: Serialize + Clone + Send + 'static>(&self, event: &str, payload: S) {
        Emitter::emit(self, event, payload).ok();
    }
}

pub trait ResultExt {
    type Ok;

    fn err_to_string(self) -> Result<Self::Ok, String>;
}

impl<T, E: Display> ResultExt for Result<T, E> {
    type Ok = T;

    fn err_to_string(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

pub trait Persistant: Sized {
    fn save(&self, path: &Path) -> Result<(), String>;
    fn load(path: &Path) -> Result<Self, String>;
}

/// Marker trait to opt-into the default Serde-based implementation of `Persistant`.
pub trait SerdePersistant: Serialize + for<'de> Deserialize<'de> {}

impl<T> Persistant for T
where
    T: SerdePersistant,
{
    fn save(&self, path: &Path) -> Result<(), String> {
        let file = std::fs::File::create(path).err_to_string()?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).err_to_string()
    }

    fn load(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).err_to_string()?;
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).err_to_string()
    }
}

pub fn next_available_id(path: &Path) -> u16 {
    let mut existing: std::collections::HashSet<u16> = std::collections::HashSet::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(name) = entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<u16>().ok())
            {
                existing.insert(name);
            }
        }
    }

    (1u16..).find(|id| !existing.contains(id)).unwrap_or(1)
}

pub fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
