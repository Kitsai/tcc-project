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

use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{export_answer_path, export_test_path, get_export_dir, get_export_manifest_path};
use crate::{
    constants::{DEFAULT_AUTHOR, SOLUTIONS_PATH},
    error::AppResult,
    fs::{copy_file, to_lf, write_file},
    problem::{
        get_default_checkers_path,
        moj::{archive::pack_tar_gz, format::*, statement},
        ExportManifest, Problem, ProblemFileType, SolutionDescription,
    },
    util::Persistant,
};

/// Verbatim copy of MOJ's `scripts/compare.sh` stub
/// (cd-moj/mojtools `testlib/compare-stub.sh`). It only points at the
/// checker bridge that lives on the judge, so it must not be edited. It is
/// never executed by this app, only shipped inside the archive. A Windows
/// checkout may turn it into CRLF (`#!/bin/bash\r` makes the judge fail to
/// run it), so it is always written through `to_lf`.
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
    write_file(&out_dir.join(AUTHOR_FILE), render_author(author))?;
    write_file(&out_dir.join(TAGS_FILE), render_tags(&problem.definition.tags))?;
    write_file(&out_dir.join(CONF_FILE), DEFAULT_CONF)?;

    let title = match problem.stmt.name.trim() {
        "" => problem.definition.name.as_str(),
        name => name,
    };
    write_file(&out_dir.join(META_FILE), render_meta(title)?)?;

    write_statement(problem, out_dir, &mut report)?;
    write_tests(&problem.path, out_dir, &mut report)?;
    write_solutions(problem, out_dir, &mut report)?;
    write_checker(problem, out_dir, &mut report)?;

    Ok(report)
}

pub fn export_moj_package(problem: &Problem, dest: &Path) -> AppResult<MojExportReport> {
    let staging = tempfile::tempdir().map_err(|e| e.to_string())?;
    let report = build_moj_package(problem, staging.path())?;
    pack_tar_gz(staging.path(), dest)?;
    Ok(report)
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

    write_file(&out_dir.join(STATEMENT_FILE), statement::render(stmt))?;

    if !stmt.tutorial.trim().is_empty() {
        write_file(
            &out_dir.join(TUTORIAL_FILE),
            format!("{}\n", statement::clean(&stmt.tutorial)),
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
            sample_name(sample_count)
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

        copy_file(&input, &out_dir.join(TESTS_INPUT_DIR).join(&name))?;
        copy_file(&answer, &out_dir.join(TESTS_OUTPUT_DIR).join(&name))?;
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

fn write_solutions(
    problem: &Problem,
    out_dir: &Path,
    report: &mut MojExportReport,
) -> AppResult<()> {
    let mut descriptions = SolutionDescription::load_all(&problem.path)?;
    descriptions.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    let mut good = 0;
    for desc in &descriptions {
        let Some(dir) = folder_for_tag(&desc.tag) else {
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
            &out_dir.join(SOLS_DIR).join(dir).join(&desc.file_name),
        )?;
        report.solutions += 1;
        if dir == GOOD_DIR {
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

    copy_file(&source, &out_dir.join(CHECKER_FILE))?;
    write_file(&out_dir.join(COMPARE_FILE), to_lf(COMPARE_STUB))?;
    report.custom_checker = true;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Read};

    use super::*;
    use crate::{
        problem::{ExportManifestEntry, ProblemDefinition, SolutionTag},
        test_support::temp_problem,
    };

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
        assert_eq!(
            read(out.path().join("scripts/compare.sh")),
            to_lf(COMPARE_STUB)
        );
        assert!(COMPARE_STUB.starts_with("#!/bin/bash"));
        assert!(report.custom_checker);
    }

    #[test]
    fn shipped_stub_never_has_crlf() {
        assert_eq!(to_lf("#!/bin/bash\r\nset -u\r\n"), "#!/bin/bash\nset -u\n");

        let (_dir, mut problem) = exported_problem();
        fs::write(problem.path.join("files/my.cpp"), "// checker").unwrap();
        problem.definition.checker = Some("my.cpp".into());
        let (out, _) = built(&problem);

        assert!(!read(out.path().join("scripts/compare.sh")).contains('\r'));
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
