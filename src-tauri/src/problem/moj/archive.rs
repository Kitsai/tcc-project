//! Packing a package tree into a `.tar.gz` and unpacking one back.

use std::{
    fs,
    path::{Path, PathBuf},
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use crate::{
    error::{AppError, AppResult, FsOperation},
    fs::FsResultExt,
    util::ResultExt,
};

/// Archives `src` into `dest`. The archive is reproducible (sorted entries,
/// mtime 0) and the modes are explicit, so `scripts/compare.sh` keeps `+x` no
/// matter which OS built it.
pub fn pack_tar_gz(src: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).fs_context(FsOperation::CreateDir, parent)?;
    }

    let file = fs::File::create(dest).fs_context(FsOperation::Create, dest)?;
    let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::default()));

    append_dir(&mut builder, src, src)?;

    builder
        .into_inner()
        .and_then(|encoder| encoder.finish())
        .fs_context(FsOperation::Write, dest)?;

    Ok(())
}

fn is_executable(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("sh")
}

fn append_dir<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    root: &Path,
    dir: &Path,
) -> AppResult<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .fs_context(FsOperation::Read, dir)?
        .collect::<Result<_, _>>()
        .fs_context(FsOperation::Read, dir)?;
    // Sorted so the archive is reproducible.
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(root).map_err(|e| e.to_string())?;
        // Always `/`, whatever the host OS.
        let name = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        let mut header = tar::Header::new_gnu();
        header.set_mtime(0);

        if path.is_dir() {
            header.set_entry_type(tar::EntryType::Directory);
            header.set_mode(0o755);
            header.set_size(0);
            builder
                .append_data(&mut header, format!("{name}/"), std::io::empty())
                .fs_context(FsOperation::Write, &path)?;
            append_dir(builder, root, &path)?;
        } else {
            let file = fs::File::open(&path).fs_context(FsOperation::Read, &path)?;
            let size = file.metadata().fs_context(FsOperation::Read, &path)?.len();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_mode(if is_executable(&path) { 0o755 } else { 0o644 });
            header.set_size(size);
            builder
                .append_data(&mut header, &name, file)
                .fs_context(FsOperation::Write, &path)?;
        }
    }

    Ok(())
}

/// Resolves `source` to the package root directory. An archive is unpacked
/// into a temporary directory (returned so the caller keeps it alive); `tar`
/// refuses entries that would escape it (`..`, absolute paths).
pub fn open_package(source: &Path) -> AppResult<(Option<tempfile::TempDir>, PathBuf)> {
    if source.is_dir() {
        return Ok((None, source.to_path_buf()));
    }
    if !source.is_file() {
        return Err(AppError::from(
            "Invalid source: select a .tar.gz file or a folder",
        ));
    }

    let file_name = source
        .file_name()
        .map(|f| f.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let file = fs::File::open(source).err_to_string()?;

    let temp = tempfile::tempdir().err_to_string()?;
    if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
        tar::Archive::new(GzDecoder::new(file))
            .unpack(temp.path())
            .err_to_string()?;
    } else if file_name.ends_with(".tar") {
        tar::Archive::new(file)
            .unpack(temp.path())
            .err_to_string()?;
    } else {
        return Err(AppError::from(
            "Unsupported format: use a .tar.gz or .tar file, or the package folder",
        ));
    }

    let root = unwrap_single_directory(temp.path());
    Ok((Some(temp), root))
}

/// Some archives wrap everything in one top-level directory.
fn unwrap_single_directory(dir: &Path) -> PathBuf {
    let entries: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();

    match entries.as_slice() {
        [only] if only.is_dir() => only.clone(),
        _ => dir.to_path_buf(),
    }
}
