use std::{fmt::Display, fs, path::Path};

use serde::{Deserialize, Serialize};
use tauri::{
    webview::cookie::time::{Month, UtcDateTime, Weekday},
    AppHandle, Emitter,
};

use crate::error::AppResult;

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
    fn save(&self, path: &Path) -> AppResult<()>;
    fn load(path: &Path) -> AppResult<Self>;
}

/// Marker trait to opt-into the default Serde-based implementation of `Persistant`.
pub trait SerdePersistant: Serialize + for<'de> Deserialize<'de> {}

impl<T> Persistant for T
where
    T: SerdePersistant,
{
    fn save(&self, path: &Path) -> AppResult<()> {
        let file = std::fs::File::create(path).err_to_string()?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).err_to_string()?;
        Ok(())
    }

    fn load(path: &Path) -> AppResult<Self> {
        let file = std::fs::File::open(path).err_to_string()?;
        let reader = std::io::BufReader::new(file);
        Ok(serde_json::from_reader(reader).err_to_string()?)
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

/// Current UTC time as `"Tue Feb 10 00:28:04 UTC 2026"`.
pub fn now() -> String {
    let now = UtcDateTime::now();

    let weekday = match now.weekday() {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    };

    let month = match now.month() {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    };

    format!(
        "{weekday} {month} {:02} {:02}:{:02}:{:02} UTC {}",
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        now.year(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_available_id_is_one_for_an_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(next_available_id(dir.path()), 1);
    }

    #[test]
    fn next_available_id_fills_a_gap() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("01"), "").unwrap();
        std::fs::write(dir.path().join("03"), "").unwrap();
        assert_eq!(next_available_id(dir.path()), 2);
    }

    #[test]
    fn next_available_id_continues_past_contiguous_ids() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("01"), "").unwrap();
        std::fs::write(dir.path().join("02"), "").unwrap();
        assert_eq!(next_available_id(dir.path()), 3);
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Fixture {
        name: String,
        value: u32,
    }

    impl SerdePersistant for Fixture {}

    #[test]
    fn persistant_round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fixture.json");

        let original = Fixture {
            name: "hello".to_string(),
            value: 42,
        };
        original.save(&path).unwrap();

        let loaded = Fixture::load(&path).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn persistant_load_fails_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.json");

        assert!(Fixture::load(&path).is_err());
    }
}
