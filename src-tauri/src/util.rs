use std::{fmt::Display, path::Path};

use serde::{Deserialize, Serialize};

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
