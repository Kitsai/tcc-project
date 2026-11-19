use std::{
    collections::HashSet,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::{
    webview::cookie::time::{Month, UtcDateTime, Weekday},
    AppHandle, Emitter,
};

use crate::error::{AppError, AppResult};

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
    let existing = used_ids(path);

    (1u16..).find(|id| !existing.contains(id)).unwrap_or(1)
}

fn used_ids(path: &Path) -> HashSet<u16> {
    let mut existing = HashSet::new();

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

    existing
}

/// `count` ids not used by any file in `path`, in increasing order, searching
/// from `first_id` upwards. Writes nothing. Fails if fewer than `count` free
/// ids remain (the highest id, `u16::MAX`, is never handed out).
pub fn assign_ids(path: &Path, first_id: u16, count: usize) -> AppResult<Vec<u16>> {
    let used = used_ids(path);
    let ids: Vec<u16> = (first_id..u16::MAX)
        .filter(|id| !used.contains(id))
        .take(count)
        .collect();

    if ids.len() < count {
        return Err(AppError::from(format!(
            "Not enough free test ids from {first_id} upwards: needed {count}, found {}",
            ids.len()
        )));
    }

    Ok(ids)
}

pub fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

fn take_digits(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut digits = String::new();
    while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
        digits.push(c);
    }
    digits.trim_start_matches('0').to_string()
}

/// Orders `2` before `10` (and `sample2` before `sample10`).
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());

    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let (da, db) = (take_digits(&mut a), take_digits(&mut b));
                let order = da.len().cmp(&db.len()).then_with(|| da.cmp(&db));
                if order != Ordering::Equal {
                    return order;
                }
            }
            (Some(x), Some(y)) => {
                if x != y {
                    return x.cmp(&y);
                }
                a.next();
                b.next();
            }
        }
    }
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

pub trait PathExt {
    fn join_id(&self, id: u16) -> PathBuf;
}

impl PathExt for Path {
    fn join_id(&self, id: u16) -> PathBuf {
        self.join(format!("{:02}", id))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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

    #[test]
    fn natural_order_puts_2_before_10() {
        let mut names = vec!["10", "2", "sample10", "sample2", "01", "1"];
        names.sort_by(|a, b| natural_cmp(a, b));

        assert_eq!(names, ["01", "1", "2", "10", "sample2", "sample10"]);
    }

    #[test]
    fn join_id_pads_ids_to_two_digits() {
        let base_path = PathBuf::from("test/path");

        assert_eq!(base_path.join_id(4), base_path.join("04"));
        assert_eq!(base_path.join_id(44), base_path.join("44"));
    }

    #[test]
    fn join_id_does_not_truncate_longer_ids() {
        let base_path = PathBuf::from("test/path");

        assert_eq!(base_path.join_id(344), base_path.join("344"));
    }

    #[test]
    fn join_id_works_on_a_borrowed_path() {
        let base_path = PathBuf::from("test/path");
        let borrowed: &Path = &base_path;

        assert_eq!(borrowed.join_id(7), base_path.join("07"));
    }

    /// A directory holding one (empty) file per id, named the way tests are
    /// stored on disk (`01`, `02`, ...). Kept alive by the returned `TempDir`.
    fn dir_with_ids(ids: &[u16]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for id in ids {
            fs::write(dir.path().join_id(*id), "").unwrap();
        }
        dir
    }

    #[test]
    fn assign_ids_in_an_empty_dir_starts_at_first_id() {
        let dir = dir_with_ids(&[]);

        assert_eq!(assign_ids(dir.path(), 1, 3).unwrap(), [1, 2, 3]);
        assert_eq!(assign_ids(dir.path(), 5, 2).unwrap(), [5, 6]);
    }

    #[test]
    fn assign_ids_skips_ids_already_in_use() {
        let dir = dir_with_ids(&[2, 5]);

        assert_eq!(assign_ids(dir.path(), 1, 3).unwrap(), [1, 3, 4]);
        assert_eq!(assign_ids(dir.path(), 1, 4).unwrap(), [1, 3, 4, 6]);
    }

    #[test]
    fn assign_ids_moves_on_when_first_id_is_taken() {
        let dir = dir_with_ids(&[1, 2]);

        assert_eq!(assign_ids(dir.path(), 1, 2).unwrap(), [3, 4]);
    }

    #[test]
    fn assign_ids_never_goes_below_first_id() {
        // ids 1 and 2 are free, but the search starts at 3
        let dir = dir_with_ids(&[3]);

        assert_eq!(assign_ids(dir.path(), 3, 2).unwrap(), [4, 5]);
    }

    #[test]
    fn assign_ids_with_count_zero_is_empty() {
        let dir = dir_with_ids(&[1]);

        assert!(assign_ids(dir.path(), 1, 0).unwrap().is_empty());
    }

    #[test]
    fn assign_ids_fails_when_there_are_not_enough_free_ids() {
        let dir = dir_with_ids(&[]);

        let err = assign_ids(dir.path(), u16::MAX - 1, 2).unwrap_err();

        assert!(err.to_string().contains("needed 2, found 1"));
        assert!(assign_ids(dir.path(), u16::MAX, 1).is_err());
    }
}
