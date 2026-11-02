//! What the MOJ exporter (`export/moj.rs`) and importer (`import/moj.rs`) share:
//! the package layout and small text formats (`format`), the statement file
//! (`statement`) and the `.tar.gz` handling (`archive`).

pub mod archive;
pub mod format;
pub mod statement;
