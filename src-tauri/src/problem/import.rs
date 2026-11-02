//! Importers: build a brand-new problem directory from another judge's package.
//! `moj` reads MOJ's native PACOTE, `polygon` reads a Polygon "full package".

mod moj;
mod polygon;

use std::{fs, path::Path};

use serde::Serialize;

use crate::{
    compile_service::CompileService,
    error::{AppError, AppResult},
    problem::{create_problem_dirs, Problem, ProblemFileType, ProgrammingLanguage},
    util::ResultExt,
};

pub use moj::import_moj_package;
pub use polygon::import_polygon_package;

#[derive(Serialize)]
pub struct ImportResult {
    pub problem: Problem,
    pub warnings: Vec<String>,
}

/// Creates the empty problem skeleton at `dest_path` (which must not exist yet)
/// and returns the problem's name, taken from the folder name.
fn create_destination(dest_path: &Path) -> AppResult<String> {
    if dest_path.exists() {
        return Err(AppError::from(
            "A folder with that name already exists at the selected destination",
        ));
    }

    let name = dest_path
        .file_name()
        .ok_or_else(|| AppError::from("Invalid destination name"))?
        .to_string_lossy()
        .into_owned();

    fs::create_dir_all(dest_path).err_to_string()?;
    create_problem_dirs(dest_path)?;

    Ok(name)
}

/// Copies `source_abs` into `dest_path/<file_type's dir>/`, compiles it, and
/// returns the basename on success. Any failure (missing source, unrecognized
/// extension, compile error) is pushed onto `warnings` and yields `None` —
/// the file is still left copied where possible so the user can fix/select
/// it manually later.
pub(super) async fn import_and_compile_file(
    source_abs: &Path,
    dest_path: &Path,
    file_type: ProblemFileType,
    compile_service: &CompileService,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let Some(file_name) = source_abs.file_name().map(|f| f.to_string_lossy().into_owned()) else {
        warnings.push(format!("Invalid file path: {:?}", source_abs));
        return None;
    };

    let relative = Path::new(file_type.directory()).join(&file_name);
    let destination = dest_path.join(&relative);

    if let Err(e) = fs::copy(source_abs, &destination) {
        warnings.push(format!("Failed to copy '{file_name}' ({file_type}): {e}"));
        return None;
    }

    let Some(language) = ProgrammingLanguage::get_from_path(&relative) else {
        warnings.push(format!("'{file_name}' ({file_type}) has an unrecognized extension and was not selected"));
        return None;
    };

    let _guard = compile_service.lock().await;
    if let Err(e) = compile_service.compile(&language, &relative, dest_path).await {
        warnings.push(format!("Failed to compile '{file_name}' ({file_type}): {e}"));
        return None;
    }

    Some(file_name)
}
