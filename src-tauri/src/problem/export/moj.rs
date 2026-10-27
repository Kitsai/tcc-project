//! MOJ "PACOTE" exporter (the judge's native package format).
//!
//! Consumes the staging area the previous stages leave in `tests/export/`
//! (numbered inputs `NN`, answers `NN.a`, `manifest.json`) plus the problem's
//! statement, solutions and checker, and produces a `.tar.gz` that MOJ's
//! `/problems/upload` accepts.
//!
//! Two steps, kept separate so the tree can be inspected before it is packed:
//! `build_moj_package` writes the directory tree, `pack_tar_gz` archives it.
//! `export_moj_package` chains both through a throwaway `TempDir`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use flate2::{write::GzEncoder, Compression};
use serde::Serialize;

use super::{export_answer_path, export_test_path, get_export_dir, get_export_manifest_path};
use crate::{
    constants::{DEFAULT_AUTHOR, SOLUTIONS_PATH},
    error::{AppResult, FsOperation},
    fs::FsResultExt,
    problem::{
        get_default_checkers_path, ExportManifest, Problem, ProblemFileType, SolutionDescription,
        SolutionTag,
    },
    util::Persistant,
};

/// Verbatim copy of MOJ's `scripts/compare.sh` stub
/// (cd-moj/mojtools `testlib/compare-stub.sh`). It only points at the
/// checker bridge that lives on the judge, so it must not be edited. It is
/// never executed by this app, only shipped inside the archive.
const COMPARE_STUB: &str = include_str!("../../../resources/moj/compare-stub.sh");

const DEFAULT_CONF: &str = "TLMOD[calibrafactor]=1.35\nULIMITS[-u]=10000\nALLOWPARALLELTEST=y\n";

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MojExportReport {
    pub tests: usize,
    pub examples: usize,
    pub solutions: usize,
    pub custom_checker: bool,
    pub warnings: Vec<String>,
}

pub fn build_moj_package(problem: &Problem, out_dir: &Path) -> AppResult<MojExportReport> {
    let mut report = MojExportReport::default();

    let author = problem
        .definition
        .author
        .as_deref()
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .unwrap_or(DEFAULT_AUTHOR);
    write_file(&out_dir.join("author"), format!("{author}\n"))?;
    write_file(&out_dir.join("tags"), render_tags(&problem.definition.tags))?;
    write_file(&out_dir.join("conf"), DEFAULT_CONF)?;

    // Only content fields: `owner`/`public` are server-side and ignored on upload.
    let title = match problem.stmt.name.trim() {
        "" => problem.definition.name.as_str(),
        name => name,
    };
    let meta = serde_json::json!({
        "display_title": title,
        "languages": [],
        "collections": [],
    });
    write_file(
        &out_dir.join(".moj-meta.json"),
        serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?,
    )?;

    write_statement(problem, out_dir, &mut report)?;
    write_tests(&problem.path, out_dir, &mut report)?;
    write_solutions(problem, out_dir, &mut report)?;
    write_checker(problem, out_dir, &mut report)?;

    Ok(report)
}

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

pub fn export_moj_package(problem: &Problem, dest: &Path) -> AppResult<MojExportReport> {
    let staging = tempfile::tempdir().map_err(|e| e.to_string())?;
    let report = build_moj_package(problem, staging.path())?;
    pack_tar_gz(staging.path(), dest)?;
    Ok(report)
}

fn write_file(path: &Path, contents: impl AsRef<[u8]>) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).fs_context(FsOperation::CreateDir, parent)?;
    }
    fs::write(path, contents).fs_context(FsOperation::Write, path)
}

fn render_tags(tags: &[String]) -> String {
    tags.iter()
        .map(|t| t.trim().trim_start_matches('#').trim())
        .filter(|t| !t.is_empty())
        .map(|t| format!("#{t}\n"))
        .collect()
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_string()
}

fn write_statement(
    problem: &Problem,
    out_dir: &Path,
    report: &mut MojExportReport,
) -> AppResult<()> {
    let stmt = &problem.stmt;

    if stmt.input.trim().is_empty() {
        report
            .warnings
            .push("The statement has no input section".into());
    }
    if stmt.output.trim().is_empty() {
        report
            .warnings
            .push("The statement has no output section".into());
    }

    // MOJ's validation requires exactly these two headings.
    let mut sections = Vec::new();
    if !stmt.legend.trim().is_empty() {
        sections.push(normalize(&stmt.legend));
    }
    sections.push("## Entrada".to_string());
    sections.push(normalize(&stmt.input));
    sections.push("## Saída".to_string());
    sections.push(normalize(&stmt.output));
    if !stmt.notes.trim().is_empty() {
        sections.push("## Observações".to_string());
        sections.push(normalize(&stmt.notes));
    }
    sections.retain(|s| !s.is_empty());

    write_file(
        &out_dir.join("docs/enunciado.md"),
        format!("{}\n", sections.join("\n\n")),
    )?;

    if !stmt.tutorial.trim().is_empty() {
        write_file(
            &out_dir.join("docs/solucao.md"),
            format!("{}\n", normalize(&stmt.tutorial)),
        )?;
    }

    Ok(())
}

fn write_tests(problem_path: &Path, out_dir: &Path, report: &mut MojExportReport) -> AppResult<()> {
    let manifest_path = get_export_manifest_path(problem_path);
    if !manifest_path.exists() {
        return Err("No export tests found: run the generate/solve steps first".into());
    }

    let mut manifest = ExportManifest::load(&manifest_path)?;
    manifest.tests.sort_unstable_by_key(|t| t.final_id);

    if manifest.tests.is_empty() {
        return Err("The export has no tests".into());
    }

    let export_dir = get_export_dir(problem_path);
    let mut sample_count = 0;

    for entry in &manifest.tests {
        let name = if entry.example {
            sample_count += 1;
            format!("sample{sample_count}")
        } else {
            format!("{:02}", entry.final_id)
        };

        let input = export_test_path(&export_dir, entry.final_id);
        let answer = export_answer_path(&export_dir, entry.final_id);
        if !answer.exists() {
            return Err(format!(
                "Test {} has no answer file: run the solve step before exporting",
                entry.final_id
            )
            .into());
        }

        copy_file(&input, &out_dir.join("tests/input").join(&name))?;
        copy_file(&answer, &out_dir.join("tests/output").join(&name))?;
    }

    report.tests = manifest.tests.len();
    report.examples = sample_count;
    if sample_count == 0 {
        report
            .warnings
            .push("No test is marked as an example: the statement will show no samples".into());
    }

    Ok(())
}

fn copy_file(from: &Path, to: &Path) -> AppResult<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).fs_context(FsOperation::CreateDir, parent)?;
    }
    fs::copy(from, to).fs_context(FsOperation::Copy, from)?;
    Ok(())
}

/// `Main` and `Accepted` both map to `good`: MOJ has no "main" notion and
/// calibrates the time limit from the slowest `good` solution.
fn sols_dir(tag: &SolutionTag) -> Option<&'static str> {
    match tag {
        SolutionTag::Main | SolutionTag::Accepted => Some("good"),
        SolutionTag::WrongAnswer => Some("wrong"),
        SolutionTag::TimeLimitExceeded => Some("slow"),
        SolutionTag::TimeLimitExceededOrAccepted
        | SolutionTag::TimeLimitExceededOrMemoryLimitExceeded
        | SolutionTag::MemoryLimitExceeded
        | SolutionTag::None => None,
    }
}

fn write_solutions(
    problem: &Problem,
    out_dir: &Path,
    report: &mut MojExportReport,
) -> AppResult<()> {
    let mut descriptions = SolutionDescription::load_all(&problem.path)?;
    descriptions.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    let mut good = 0;
    for desc in &descriptions {
        let Some(dir) = sols_dir(&desc.tag) else {
            report.warnings.push(format!(
                "Solution {} has a tag MOJ has no folder for: skipped",
                desc.file_name
            ));
            continue;
        };

        let source = problem.path.join(SOLUTIONS_PATH).join(&desc.file_name);
        if !source.exists() {
            report.warnings.push(format!(
                "Solution {} is described but its source file is missing: skipped",
                desc.file_name
            ));
            continue;
        }

        copy_file(
            &source,
            &out_dir.join("sols").join(dir).join(&desc.file_name),
        )?;
        report.solutions += 1;
        if dir == "good" {
            good += 1;
        }
    }

    if good == 0 {
        report.warnings.push(
            "No accepted solution was exported: MOJ calibrates the time limit from `sols/good`"
                .into(),
        );
    }

    Ok(())
}

fn checker_source(problem: &Problem) -> Option<PathBuf> {
    let checker = problem.definition.checker.as_ref()?;

    Some(match checker.strip_prefix("@default:") {
        Some(name) => get_default_checkers_path()
            .map(|p| p.join(name))
            .unwrap_or_else(|| PathBuf::from(name)),
        None => problem
            .path
            .join(ProblemFileType::Checker.directory())
            .join(checker),
    })
}

fn write_checker(problem: &Problem, out_dir: &Path, report: &mut MojExportReport) -> AppResult<()> {
    let Some(source) = checker_source(problem) else {
        return Ok(());
    };

    if source.extension().and_then(|e| e.to_str()) != Some("cpp") {
        return Err(format!(
            "MOJ only supports C++ checkers, but the selected one is {:?}",
            source
        )
        .into());
    }
    if !source.exists() {
        return Err(format!("Checker source not found: {:?}", source).into());
    }

    copy_file(&source, &out_dir.join("scripts/checker.cpp"))?;
    write_file(&out_dir.join("scripts/compare.sh"), COMPARE_STUB)?;
    report.custom_checker = true;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        problem::{ExportManifestEntry, ProblemDefinition},
        test_support::temp_problem,
    };
    use std::io::Read;

    /// A problem with 3 exported tests (1 example), one solution per
    /// interesting tag, a filled statement and an author.
    fn exported_problem() -> (tempfile::TempDir, Problem) {
        let (dir, mut problem) = temp_problem("p");
        problem.definition.author = Some("Ana".into());
        problem.definition.tags = vec!["greedy".into(), "#math".into(), "  ".into()];
        problem.stmt.name = "Dobro".into();
        problem.stmt.legend = "Dado N,\r\nimprima 2N.".into();
        problem.stmt.input = "Um inteiro $N$.".into();
        problem.stmt.output = "Um inteiro.".into();
        problem.stmt.notes = "$1 \\le N \\le 10^3$".into();
        problem.stmt.tutorial = "Multiplique por dois.".into();

        let export_dir = problem.path.join("tests/export");
        fs::create_dir_all(&export_dir).unwrap();
        let entries = [(1, true), (2, false), (3, false)];
        for (id, _) in entries {
            fs::write(export_test_path(&export_dir, id), format!("{id}\n")).unwrap();
            fs::write(export_answer_path(&export_dir, id), format!("{}\n", id * 2)).unwrap();
        }
        ExportManifest {
            tests: entries
                .iter()
                .map(|&(id, example)| ExportManifestEntry {
                    final_id: id,
                    source_test_id: id,
                    example,
                })
                .collect(),
        }
        .save(&export_dir.join("manifest.json"))
        .unwrap();

        for (file, tag) in [
            ("main.cpp", SolutionTag::Main),
            ("ok.cpp", SolutionTag::Accepted),
            ("wa.cpp", SolutionTag::WrongAnswer),
            ("slow.cpp", SolutionTag::TimeLimitExceeded),
            ("mle.cpp", SolutionTag::MemoryLimitExceeded),
        ] {
            fs::write(
                problem.path.join("solutions").join(file),
                format!("// {file}"),
            )
            .unwrap();
            let mut desc = SolutionDescription::new(file.to_string());
            desc.tag = tag;
            desc.save_solution(&problem.path).unwrap();
        }

        (dir, problem)
    }

    fn read(path: impl AsRef<Path>) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn built(problem: &Problem) -> (tempfile::TempDir, MojExportReport) {
        let out = tempfile::tempdir().unwrap();
        let report = build_moj_package(problem, out.path()).unwrap();
        (out, report)
    }

    #[test]
    fn writes_metadata_files() {
        let (_dir, problem) = exported_problem();
        let (out, _) = built(&problem);

        assert_eq!(read(out.path().join("author")), "Ana\n");
        assert_eq!(read(out.path().join("tags")), "#greedy\n#math\n");
        assert_eq!(read(out.path().join("conf")), DEFAULT_CONF);

        let meta: serde_json::Value =
            serde_json::from_str(&read(out.path().join(".moj-meta.json"))).unwrap();
        assert_eq!(meta["display_title"], "Dobro");
        assert!(meta.get("owner").is_none() && meta.get("public").is_none());
    }

    #[test]
    fn display_title_falls_back_to_the_problem_name() {
        let (_dir, mut problem) = exported_problem();
        problem.stmt.name = "  ".into();
        let (out, _) = built(&problem);

        let meta: serde_json::Value =
            serde_json::from_str(&read(out.path().join(".moj-meta.json"))).unwrap();
        assert_eq!(meta["display_title"], "p");
    }

    #[test]
    fn missing_or_blank_author_falls_back_to_the_default() {
        let (_dir, mut problem) = exported_problem();

        for author in [None, Some("   ".to_string())] {
            problem.definition.author = author;
            let (out, _) = built(&problem);

            assert_eq!(
                read(out.path().join("author")),
                format!("{DEFAULT_AUTHOR}\n")
            );
        }
    }

    #[test]
    fn statement_has_required_headings_lf_endings_and_notes() {
        let (_dir, problem) = exported_problem();
        let (out, _) = built(&problem);

        let md = read(out.path().join("docs/enunciado.md"));
        assert_eq!(
            md,
            "Dado N,\nimprima 2N.\n\n## Entrada\n\nUm inteiro $N$.\n\n## Saída\n\nUm inteiro.\n\n\
             ## Observações\n\n$1 \\le N \\le 10^3$\n"
        );
        assert!(!md.contains('\r'));
        assert_eq!(
            read(out.path().join("docs/solucao.md")),
            "Multiplique por dois.\n"
        );
    }

    #[test]
    fn empty_notes_and_tutorial_write_nothing_extra() {
        let (_dir, mut problem) = exported_problem();
        problem.stmt.notes.clear();
        problem.stmt.tutorial.clear();
        let (out, _) = built(&problem);

        assert!(!read(out.path().join("docs/enunciado.md")).contains("Observações"));
        assert!(!out.path().join("docs/solucao.md").exists());
    }

    #[test]
    fn examples_become_samples_and_the_rest_keep_their_id() {
        let (_dir, problem) = exported_problem();
        let (out, report) = built(&problem);

        assert_eq!(read(out.path().join("tests/input/sample1")), "1\n");
        assert_eq!(read(out.path().join("tests/output/sample1")), "2\n");
        assert_eq!(read(out.path().join("tests/input/02")), "2\n");
        assert_eq!(read(out.path().join("tests/output/03")), "6\n");
        assert!(!out.path().join("tests/input/01").exists());
        assert_eq!((report.tests, report.examples), (3, 1));
    }

    #[test]
    fn tests_not_generated_is_a_clear_error() {
        let (_dir, mut problem) = temp_problem("empty");
        problem.definition.author = Some("Ana".into());

        let out = tempfile::tempdir().unwrap();
        let err = build_moj_package(&problem, out.path()).unwrap_err();
        assert!(err.to_string().contains("generate"));
    }

    #[test]
    fn test_without_answer_is_a_clear_error() {
        let (_dir, problem) = exported_problem();
        fs::remove_file(export_answer_path(&get_export_dir(&problem.path), 2)).unwrap();

        let out = tempfile::tempdir().unwrap();
        let err = build_moj_package(&problem, out.path()).unwrap_err();
        assert!(err.to_string().contains("solve"));
    }

    #[test]
    fn solutions_are_sorted_by_tag_and_lossy_ones_warn() {
        let (_dir, problem) = exported_problem();
        let (out, report) = built(&problem);
        let sols = out.path().join("sols");

        assert_eq!(read(sols.join("good/main.cpp")), "// main.cpp");
        assert!(sols.join("good/ok.cpp").exists());
        assert!(sols.join("wrong/wa.cpp").exists());
        assert!(sols.join("slow/slow.cpp").exists());
        assert!(!sols.join("good/mle.cpp").exists());
        assert!(
            !sols.join("good/main.cpp.desc").exists(),
            ".desc must not ship"
        );
        assert_eq!(report.solutions, 4);
        assert!(report.warnings.iter().any(|w| w.contains("mle.cpp")));
    }

    #[test]
    fn warns_when_there_is_no_good_solution() {
        let (_dir, problem) = exported_problem();
        for f in ["main.cpp", "ok.cpp"] {
            fs::remove_file(problem.path.join("solutions").join(f)).unwrap();
            fs::remove_file(problem.path.join("solutions").join(format!("{f}.desc"))).unwrap();
        }
        let (_out, report) = built(&problem);

        assert!(report.warnings.iter().any(|w| w.contains("sols/good")));
    }

    #[test]
    fn no_checker_means_no_scripts_dir() {
        let (_dir, problem) = exported_problem();
        let (out, report) = built(&problem);

        assert!(!out.path().join("scripts").exists());
        assert!(!report.custom_checker);
    }

    #[test]
    fn user_checker_ships_source_and_untouched_stub() {
        let (_dir, mut problem) = exported_problem();
        fs::write(problem.path.join("files/my.cpp"), "// checker").unwrap();
        problem.definition.checker = Some("my.cpp".into());
        let (out, report) = built(&problem);

        assert_eq!(read(out.path().join("scripts/checker.cpp")), "// checker");
        assert_eq!(read(out.path().join("scripts/compare.sh")), COMPARE_STUB);
        assert!(COMPARE_STUB.starts_with("#!/bin/bash"));
        assert!(report.custom_checker);
    }

    #[test]
    fn bundled_default_checker_is_resolved() {
        let (_dir, mut problem) = exported_problem();
        problem.definition.checker = Some("@default:mojdiff.cpp".into());
        let (out, _) = built(&problem);

        assert!(read(out.path().join("scripts/checker.cpp")).contains("testlib.h"));
    }

    #[test]
    fn non_cpp_or_missing_checker_is_an_error() {
        let (_dir, mut problem) = exported_problem();
        let out = tempfile::tempdir().unwrap();

        problem.definition.checker = Some("c.py".into());
        assert!(build_moj_package(&problem, out.path()).is_err());

        problem.definition.checker = Some("gone.cpp".into());
        assert!(build_moj_package(&problem, out.path()).is_err());
    }

    /// `(path, mode, content)` for every entry of the archive.
    fn list_archive(path: &Path) -> Vec<(String, u32, String)> {
        let decoder = flate2::read::GzDecoder::new(fs::File::open(path).unwrap());
        tar::Archive::new(decoder)
            .entries()
            .unwrap()
            .map(|e| {
                let mut e = e.unwrap();
                let mut content = String::new();
                if e.header().entry_type().is_file() {
                    e.read_to_string(&mut content).unwrap();
                }
                (
                    e.path().unwrap().to_string_lossy().into_owned(),
                    e.header().mode().unwrap(),
                    content,
                )
            })
            .collect()
    }

    #[test]
    fn archive_keeps_exec_bit_on_compare_sh_only() {
        let (_dir, mut problem) = exported_problem();
        fs::write(problem.path.join("files/my.cpp"), "// checker").unwrap();
        problem.definition.checker = Some("my.cpp".into());
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("out/p.tar.gz");

        let report = export_moj_package(&problem, &dest).unwrap();
        let entries = list_archive(&dest);
        let mode_of = |p: &str| {
            entries
                .iter()
                .find(|e| e.0 == p)
                .unwrap_or_else(|| panic!("{p} missing from archive"))
                .1
        };

        assert_eq!(mode_of("scripts/compare.sh") & 0o777, 0o755);
        assert_eq!(mode_of("scripts/checker.cpp") & 0o777, 0o644);
        assert_eq!(mode_of("author") & 0o777, 0o644);
        assert!(entries.iter().any(|e| e.0 == ".moj-meta.json"));
        assert!(entries.iter().any(|e| e.0 == "tests/input/sample1"));
        assert!(entries
            .iter()
            .all(|e| !e.0.starts_with('/') && !e.0.starts_with("./")));
        assert!(report.custom_checker);
    }

    #[test]
    fn archive_is_reproducible() {
        let (_dir, problem) = exported_problem();
        let tmp = tempfile::tempdir().unwrap();
        let (a, b) = (tmp.path().join("a.tar.gz"), tmp.path().join("b.tar.gz"));

        export_moj_package(&problem, &a).unwrap();
        export_moj_package(&problem, &b).unwrap();

        assert_eq!(fs::read(a).unwrap(), fs::read(b).unwrap());
    }

    #[test]
    fn old_prblm_without_author_or_tags_still_loads() {
        let json = r#"{"name":"x","checker":null,"validator":null,"mainSolution":null}"#;
        let def: ProblemDefinition = serde_json::from_str(json).unwrap();

        assert!(def.author.is_none() && def.tags.is_empty());
    }

    /// Not a regression test: writes a real package for manual inspection.
    /// `MOJ_DUMP_DIR=/some/dir cargo test dump_package -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn dump_package() {
        let (_dir, mut problem) = exported_problem();
        problem.definition.checker = Some("@default:mojdiff.cpp".into());
        let out = PathBuf::from(std::env::var("MOJ_DUMP_DIR").expect("set MOJ_DUMP_DIR"));

        let tree = out.join("tree");
        fs::create_dir_all(&tree).unwrap();
        let report = build_moj_package(&problem, &tree).unwrap();
        pack_tar_gz(&tree, &out.join("package.tar.gz")).unwrap();
        println!("{report:#?}");
    }
}
