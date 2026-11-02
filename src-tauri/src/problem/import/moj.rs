use std::{fs, path::Path};

use super::{create_destination, import_and_compile_file, ImportResult};
use crate::{
    compile_service::CompileService,
    constants::SOLUTIONS_PATH,
    error::{AppError, AppResult},
    fs::file_names_in,
    problem::{
        moj::{archive::open_package, format::*, statement},
        Problem, ProblemFileType, ProblemStatement, ProgrammingLanguage, SolutionDescription,
        SolutionTag, TestDefinition, TestType,
    },
    util::{natural_cmp, Persistant},
};

const FALLBACK_CHECKER: &str = "@default:mojdiff.cpp";

pub async fn import_moj_package(
    source: &Path,
    dest_path: &Path,
    compile_service: &CompileService,
) -> AppResult<ImportResult> {
    let (_extracted, root) = open_package(source)?;

    if !looks_like_package(&root) {
        return Err(AppError::from(
            "The source is not a MOJ package (no docs/ or tests/ folder and no author file)",
        ));
    }

    let name = create_destination(dest_path)?;

    let mut problem = Problem::create(&name, dest_path.to_path_buf());
    let mut warnings = Vec::new();

    problem.definition.author = read(&root, AUTHOR_FILE).as_deref().and_then(parse_author);
    problem.definition.tags = parse_tags(&read(&root, TAGS_FILE).unwrap_or_default());

    let title = read(&root, META_FILE)
        .as_deref()
        .and_then(parse_display_title)
        .unwrap_or_else(|| name.clone());
    problem.stmt = import_statement(&root, &title);

    import_tests(&root, dest_path, &mut warnings)?;
    import_solutions(&root, dest_path, &mut warnings)?;
    let solutions = SolutionDescription::load_all(dest_path)?;
    problem.definition.main_solution = SolutionDescription::main_solution_file_name(&solutions);
    if problem.definition.main_solution.is_none() {
        warnings.push(
            "No solution in sols/good: the problem has no main solution, which is needed to \
             generate the expected answers"
                .to_string(),
        );
    }

    import_scripts(
        &root,
        dest_path,
        &mut problem,
        compile_service,
        &mut warnings,
    )
    .await;

    problem.save_to_disk()?;

    Ok(ImportResult { problem, warnings })
}

/// File contents, or `None` if it is missing or not UTF-8.
fn read(root: &Path, relative: &str) -> Option<String> {
    fs::read_to_string(root.join(relative)).ok()
}

fn looks_like_package(root: &Path) -> bool {
    root.join("docs").is_dir()
        || root.join("tests").is_dir()
        || root.join(AUTHOR_FILE).is_file()
}

fn import_statement(root: &Path, title: &str) -> ProblemStatement {
    let markdown = read(root, STATEMENT_FILE).unwrap_or_default();
    let mut sections = statement::split(&markdown);

    // Only when the statement has no `## Observações` of its own, fall back
    // to the per-sample notes MOJ keeps in `docs/notes/sampleN.md`.
    if sections.notes.is_empty() {
        sections.notes = read_sample_notes(root);
    }

    let tutorial = statement::clean(&read(root, TUTORIAL_FILE).unwrap_or_default());

    ProblemStatement {
        name: title.to_string(),
        legend: sections.legend,
        input: sections.input,
        output: sections.output,
        notes: sections.notes,
        tutorial,
    }
}

fn read_sample_notes(root: &Path) -> String {
    let mut files: Vec<String> = file_names_in(&root.join(SAMPLE_NOTES_DIR))
        .into_iter()
        .filter(|n| n.to_lowercase().ends_with(".md"))
        .collect();
    files.sort_by(|a, b| natural_cmp(a, b));

    files
        .iter()
        .filter_map(|f| fs::read_to_string(root.join(SAMPLE_NOTES_DIR).join(f)).ok())
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Each `tests/input/<name>` becomes a `Manual` test. Examples (`sampleN`)
/// come first, as the app lists them, then the rest in natural order. The
/// original file name is kept as the test's description.
fn import_tests(root: &Path, dest_path: &Path, warnings: &mut Vec<String>) -> AppResult<()> {
    let mut names = file_names_in(&root.join(TESTS_INPUT_DIR));
    if names.is_empty() {
        warnings.push("No tests found in tests/input".to_string());
        return Ok(());
    }

    names.sort_by(|a, b| {
        is_sample_name(b)
            .cmp(&is_sample_name(a))
            .then_with(|| natural_cmp(a, b))
    });

    let mut next_id: u16 = 1;
    for name in &names {
        let content = match fs::read_to_string(root.join(TESTS_INPUT_DIR).join(name)) {
            Ok(content) => content,
            Err(_) => {
                warnings.push(format!(
                    "Test '{name}' is unreadable (not UTF-8 text), skipped"
                ));
                continue;
            }
        };

        TestDefinition::new(next_id, TestType::Manual, &content, is_sample_name(name), name)
            .save(&TestDefinition::path(dest_path, next_id))?;
        next_id += 1;
    }

    Ok(())
}

/// A name that does not collide with a solution already imported from another
/// `sols/` folder (`good/a.cpp` and `wrong/a.cpp` are different programs).
fn unique_solution_name(solutions_dir: &Path, folder: &str, file_name: &str) -> String {
    if !solutions_dir.join(file_name).exists() {
        return file_name.to_string();
    }

    let mut candidate = format!("{folder}_{file_name}");
    let mut n = 2;
    while solutions_dir.join(&candidate).exists() {
        candidate = format!("{folder}{n}_{file_name}");
        n += 1;
    }
    candidate
}

fn import_solutions(root: &Path, dest_path: &Path, warnings: &mut Vec<String>) -> AppResult<()> {
    let solutions_dir = dest_path.join(SOLUTIONS_PATH);
    let mut main_chosen = false;

    for (folder, tag) in SOLUTION_FOLDERS {
        let source_dir = root.join(SOLS_DIR).join(folder);
        let mut files = file_names_in(&source_dir);
        files.sort();

        if !files.is_empty() && folder == PASS_DIR {
            warnings.push(
                "sols/pass was imported as Accepted: on export these solutions go back to sols/good"
                    .to_string(),
            );
        }
        if !files.is_empty() && folder == UPCOMING_DIR {
            warnings.push("sols/upcoming was imported without a tag".to_string());
        }

        for file in files {
            let name = unique_solution_name(&solutions_dir, folder, &file);

            if let Err(e) = fs::copy(source_dir.join(&file), solutions_dir.join(&name)) {
                warnings.push(format!(
                    "Failed to import solution '{file}' (sols/{folder}): {e}"
                ));
                continue;
            }
            SolutionDescription::new(name.clone()).save_solution(dest_path)?;

            if ProgrammingLanguage::get_from_path(Path::new(&name)).is_none() {
                warnings.push(format!(
                    "Solution '{name}' uses a language the app does not recognize"
                ));
            }

            // The first `good` solution is the main one.
            if folder == GOOD_DIR && !main_chosen {
                SolutionDescription::change_tag(dest_path, &name, SolutionTag::Main)?;
                main_chosen = true;
            } else if !matches!(tag, SolutionTag::Accepted) {
                SolutionDescription::change_tag(dest_path, &name, tag.clone())?;
            }
        }
    }

    Ok(())
}

/// `scripts/checker.cpp` → the selected checker; `scripts/validator.cpp` →
/// the selected validator. With no usable checker, the MOJ-like default is
/// selected instead (a package without one is judged by MOJ's own diff).
async fn import_scripts(
    root: &Path,
    dest_path: &Path,
    problem: &mut Problem,
    compile_service: &CompileService,
    warnings: &mut Vec<String>,
) {
    let checker = root.join(CHECKER_FILE);
    if checker.is_file() {
        problem.definition.checker = import_and_compile_file(
            &checker,
            dest_path,
            ProblemFileType::Checker,
            compile_service,
            warnings,
        )
        .await;
    } else if root.join(COMPARE_FILE).is_file() {
        warnings.push(
            "The package uses its own scripts/compare.sh, which cannot be imported: the \
             default checker (MOJ-like comparison) was selected instead"
                .to_string(),
        );
    }
    if problem.definition.checker.is_none() {
        problem.definition.checker = Some(FALLBACK_CHECKER.to_string());
    }

    let validator = root.join(VALIDATOR_FILE);
    if validator.is_file() {
        problem.definition.validator = import_and_compile_file(
            &validator,
            dest_path,
            ProblemFileType::Validator,
            compile_service,
            warnings,
        )
        .await;
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use super::*;
    use crate::{
        problem::{export_moj_package, ExportManifest, ExportManifestEntry},
        runner::{ExecutionInfo, Runner},
        test_support::{temp_problem, MockRunner},
    };

    fn compile_service() -> CompileService {
        let runner: Arc<dyn Runner> = Arc::new(MockRunner::exit_code(0));
        CompileService::new(runner)
    }

    fn failing_compile_service() -> CompileService {
        let runner: Arc<dyn Runner> = Arc::new(MockRunner::new(|_| {
            Ok(ExecutionInfo {
                stdout: String::new(),
                stderr: "boom".to_string(),
                execution_time: std::time::Duration::default(),
                exit_code: 1,
            })
        }));
        CompileService::new(runner)
    }

    fn write(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// A small hand-written package covering every folder the importer reads.
    fn package_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();

        write(r, "author", "Daniel Porto\n");
        write(r, "tags", "#greedy\n#math\n");
        write(
            r,
            ".moj-meta.json",
            r#"{"display_title":"Cabo Carente","languages":[],"collections":[]}"#,
        );
        write(
            r,
            "docs/enunciado.md",
            "A empresa quer cabos.\n\n## Entrada\n\nUm inteiro $N$.\n\n## Saída\n\nO custo.\n\n## Observações\n\nNo exemplo 1: 17.\n",
        );
        write(r, "docs/solucao.md", "Ordene e conecte.\n");

        write(r, "tests/input/10", "10\n");
        write(r, "tests/input/2", "2\n");
        write(r, "tests/input/sample1", "1\n");
        write(r, "tests/output/sample1", "1\n");

        write(r, "sols/good/b.cpp", "// b");
        write(r, "sols/good/a.cpp", "// a");
        write(r, "sols/wrong/a.cpp", "// wrong a");
        write(r, "sols/slow/s.cpp", "// slow");
        write(r, "scripts/checker.cpp", "// checker");
        write(r, "scripts/compare.sh", "#!/bin/bash\n");
        write(r, "scripts/validator.cpp", "// validator");

        dir
    }

    async fn import(source: &Path) -> (tempfile::TempDir, ImportResult) {
        let dest_parent = tempfile::tempdir().unwrap();
        let result = import_moj_package(
            source,
            &dest_parent.path().join("imported"),
            &compile_service(),
        )
        .await
        .unwrap();
        (dest_parent, result)
    }

    #[tokio::test]
    async fn imports_metadata_and_statement() {
        let src = package_dir();
        let (_keep, result) = import(src.path()).await;
        let p = &result.problem;

        assert_eq!(p.definition.author.as_deref(), Some("Daniel Porto"));
        assert_eq!(p.definition.tags, ["greedy", "math"]);
        assert_eq!(p.stmt.name, "Cabo Carente");
        assert_eq!(p.stmt.legend, "A empresa quer cabos.");
        assert_eq!(p.stmt.input, "Um inteiro $N$.");
        assert_eq!(p.stmt.output, "O custo.");
        assert_eq!(p.stmt.notes, "No exemplo 1: 17.");
        assert_eq!(p.stmt.tutorial, "Ordene e conecte.");
    }

    #[tokio::test]
    async fn imports_tests_samples_first_in_natural_order() {
        let src = package_dir();
        let (_keep, result) = import(src.path()).await;
        let path = &result.problem.path;

        let tests: Vec<TestDefinition> = (1..=3)
            .map(|id| TestDefinition::load(&TestDefinition::path(path, id)).unwrap())
            .collect();

        assert_eq!(
            tests
                .iter()
                .map(|t| t.description.as_str())
                .collect::<Vec<_>>(),
            ["sample1", "2", "10"]
        );
        assert_eq!(
            tests.iter().map(|t| t.example).collect::<Vec<_>>(),
            [true, false, false]
        );
        assert_eq!(tests[2].content, "10");
        assert!(!TestDefinition::path(path, 4).exists());
        assert!(!result.warnings.iter().any(|w| w.contains("tests/output")));
    }

    #[tokio::test]
    async fn imports_solutions_with_tags_and_a_main() {
        let src = package_dir();
        let (_keep, result) = import(src.path()).await;
        let all = SolutionDescription::load_all(&result.problem.path).unwrap();
        let tag_of = |file: &str| {
            all.iter()
                .find(|d| d.file_name == file)
                .map(|d| d.tag.clone())
        };

        assert!(matches!(tag_of("a.cpp"), Some(SolutionTag::Main)));
        assert!(matches!(tag_of("b.cpp"), Some(SolutionTag::Accepted)));
        assert!(matches!(
            tag_of("s.cpp"),
            Some(SolutionTag::TimeLimitExceeded)
        ));
        assert!(matches!(
            tag_of("wrong_a.cpp"),
            Some(SolutionTag::WrongAnswer)
        ));
        assert_eq!(
            result.problem.definition.main_solution.as_deref(),
            Some("a.cpp")
        );

        let solutions = result.problem.path.join(SOLUTIONS_PATH);
        assert_eq!(fs::read_to_string(solutions.join("a.cpp")).unwrap(), "// a");
        assert_eq!(
            fs::read_to_string(solutions.join("wrong_a.cpp")).unwrap(),
            "// wrong a"
        );
    }

    #[tokio::test]
    async fn imports_checker_and_validator() {
        let src = package_dir();
        let (_keep, result) = import(src.path()).await;
        let def = &result.problem.definition;

        assert_eq!(def.checker.as_deref(), Some("checker.cpp"));
        assert_eq!(def.validator.as_deref(), Some("validator.cpp"));
        assert!(result.problem.path.join("files/checker.cpp").exists());
        assert!(
            !result.warnings.iter().any(|w| w.contains("compare.sh")),
            "a compare.sh backed by a checker.cpp is just the stub"
        );
    }

    #[tokio::test]
    async fn no_scripts_selects_the_default_checker() {
        let src = package_dir();
        fs::remove_dir_all(src.path().join("scripts")).unwrap();
        let (_keep, result) = import(src.path()).await;

        assert_eq!(
            result.problem.definition.checker.as_deref(),
            Some(FALLBACK_CHECKER)
        );
        assert!(result.problem.definition.validator.is_none());
    }

    #[tokio::test]
    async fn custom_compare_script_without_checker_warns_and_uses_the_default() {
        let src = package_dir();
        fs::remove_file(src.path().join("scripts/checker.cpp")).unwrap();
        let (_keep, result) = import(src.path()).await;

        assert_eq!(
            result.problem.definition.checker.as_deref(),
            Some(FALLBACK_CHECKER)
        );
        assert!(result.warnings.iter().any(|w| w.contains("compare.sh")));
    }

    #[tokio::test]
    async fn checker_that_fails_to_compile_falls_back_with_a_warning() {
        let src = package_dir();
        let dest_parent = tempfile::tempdir().unwrap();
        let result = import_moj_package(
            src.path(),
            &dest_parent.path().join("imported"),
            &failing_compile_service(),
        )
        .await
        .unwrap();

        assert_eq!(
            result.problem.definition.checker.as_deref(),
            Some(FALLBACK_CHECKER)
        );
        assert!(result.warnings.iter().any(|w| w.contains("checker.cpp")));
    }

    #[tokio::test]
    async fn package_without_good_solutions_warns() {
        let src = package_dir();
        fs::remove_dir_all(src.path().join("sols/good")).unwrap();
        let (_keep, result) = import(src.path()).await;

        assert!(result.problem.definition.main_solution.is_none());
        assert!(result.warnings.iter().any(|w| w.contains("sols/good")));
    }

    #[tokio::test]
    async fn sample_notes_are_used_when_the_statement_has_no_notes_section() {
        let src = package_dir();
        write(
            src.path(),
            "docs/enunciado.md",
            "L\n\n## Entrada\n\na\n\n## Saída\n\nb\n",
        );
        write(src.path(), "docs/notes/sample2.md", "nota 2");
        write(src.path(), "docs/notes/sample1.md", "nota 1");
        let (_keep, result) = import(src.path()).await;

        assert_eq!(result.problem.stmt.notes, "nota 1\n\nnota 2");
    }

    #[tokio::test]
    async fn missing_author_stays_empty_instead_of_inventing_one() {
        let src = package_dir();
        fs::remove_file(src.path().join("author")).unwrap();
        let (_keep, result) = import(src.path()).await;

        assert!(result.problem.definition.author.is_none());
    }

    #[tokio::test]
    async fn rejects_a_folder_that_is_not_a_package() {
        let src = tempfile::tempdir().unwrap();
        write(src.path(), "notes.txt", "x");
        let dest_parent = tempfile::tempdir().unwrap();

        let err = import_moj_package(
            src.path(),
            &dest_parent.path().join("p"),
            &compile_service(),
        )
        .await
        .err()
        .unwrap();

        assert!(err.to_string().contains("not a MOJ package"));
        assert!(!dest_parent.path().join("p").exists());
    }

    #[tokio::test]
    async fn rejects_an_existing_destination() {
        let src = package_dir();
        let dest_parent = tempfile::tempdir().unwrap();
        fs::create_dir(dest_parent.path().join("p")).unwrap();

        let err = import_moj_package(
            src.path(),
            &dest_parent.path().join("p"),
            &compile_service(),
        )
        .await
        .err()
        .unwrap();

        assert!(err.to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn rejects_an_unsupported_archive_format() {
        let dir = tempfile::tempdir().unwrap();
        let zip = dir.path().join("p.zip");
        fs::write(&zip, "not a tar").unwrap();

        let err = import_moj_package(&zip, &dir.path().join("out"), &compile_service())
            .await
            .err()
            .unwrap();

        assert!(err.to_string().contains("Unsupported format"));
    }

    /// Exporting a problem with this app's exporter and importing the
    /// archive again must give back the same statement, tests, solutions,
    /// checker and author.
    #[tokio::test]
    async fn round_trips_through_the_exporter() {
        let (_dir, mut original) = temp_problem("orig");
        original.definition.author = Some("Ana".into());
        original.definition.tags = vec!["dp".into()];
        original.definition.checker = Some("my.cpp".into());
        original.stmt.name = "Dobro".into();
        original.stmt.legend = "Dado N, imprima 2N.".into();
        original.stmt.input = "Um inteiro $N$.".into();
        original.stmt.output = "Um inteiro.".into();
        original.stmt.notes = "Nota.".into();
        original.stmt.tutorial = "Multiplique.".into();
        fs::write(original.path.join("files/my.cpp"), "// my checker").unwrap();

        let export_dir = original.path.join("tests/export");
        fs::create_dir_all(&export_dir).unwrap();
        for id in 1..=3u16 {
            fs::write(export_dir.join(format!("{id:02}")), format!("{id}\n")).unwrap();
            fs::write(
                export_dir.join(format!("{id:02}.a")),
                format!("{}\n", id * 2),
            )
            .unwrap();
        }
        ExportManifest {
            tests: (1..=3u16)
                .map(|id| ExportManifestEntry {
                    final_id: id,
                    source_test_id: id,
                    example: id == 1,
                })
                .collect(),
        }
        .save(&export_dir.join("manifest.json"))
        .unwrap();

        for (file, tag) in [
            ("main.cpp", SolutionTag::Main),
            ("wa.cpp", SolutionTag::WrongAnswer),
        ] {
            fs::write(
                original.path.join(SOLUTIONS_PATH).join(file),
                format!("// {file}"),
            )
            .unwrap();
            let mut desc = SolutionDescription::new(file.to_string());
            desc.tag = tag;
            desc.save_solution(&original.path).unwrap();
        }

        let archive = tempfile::tempdir().unwrap();
        let archive_path = archive.path().join("orig.tar.gz");
        export_moj_package(&original, &archive_path).unwrap();

        let (_keep, result) = import(&archive_path).await;
        let back = &result.problem;

        assert_eq!(back.definition.author.as_deref(), Some("Ana"));
        assert_eq!(back.definition.tags, ["dp"]);
        assert_eq!(back.stmt.name, "Dobro");
        assert_eq!(back.stmt.legend, original.stmt.legend);
        assert_eq!(back.stmt.input, original.stmt.input);
        assert_eq!(back.stmt.output, original.stmt.output);
        assert_eq!(back.stmt.notes, original.stmt.notes);
        assert_eq!(back.stmt.tutorial, original.stmt.tutorial);
        assert_eq!(back.definition.main_solution.as_deref(), Some("main.cpp"));
        assert_eq!(back.definition.checker.as_deref(), Some("checker.cpp"));
        assert_eq!(
            fs::read_to_string(back.path.join("files/checker.cpp")).unwrap(),
            "// my checker"
        );

        let first = TestDefinition::load(&TestDefinition::path(&back.path, 1)).unwrap();
        assert!(first.example);
        assert_eq!(first.content, "1");
        assert!(TestDefinition::path(&back.path, 3).exists());
    }

    /// Not a regression test: imports a real package for manual inspection.
    /// `MOJ_IMPORT_SRC=/path/pkg.tar.gz cargo test import_real_package -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn import_real_package() {
        let src = PathBuf::from(std::env::var("MOJ_IMPORT_SRC").expect("set MOJ_IMPORT_SRC"));
        let dest_parent = tempfile::tempdir().unwrap();
        let dest = dest_parent.path().join("imported");

        let result = import_moj_package(&src, &dest, &compile_service())
            .await
            .unwrap();
        let p = &result.problem;

        println!("author     : {:?}", p.definition.author);
        println!("tags       : {:?}", p.definition.tags);
        println!("title      : {:?}", p.stmt.name);
        println!("legend     : {:?}", p.stmt.legend);
        println!("input      : {:?}", p.stmt.input);
        println!("output     : {:?}", p.stmt.output);
        println!("notes      : {:?}", p.stmt.notes);
        println!("checker    : {:?}", p.definition.checker);
        println!("validator  : {:?}", p.definition.validator);
        println!("main       : {:?}", p.definition.main_solution);
        for d in SolutionDescription::load_all(&p.path).unwrap() {
            println!(
                "solution   : {} {}",
                d.file_name,
                serde_json::to_string(&d.tag).unwrap()
            );
        }
        for id in 1..=20u16 {
            let path = TestDefinition::path(&p.path, id);
            if path.exists() {
                let t = TestDefinition::load(&path).unwrap();
                println!(
                    "test {id:02}    : example={} {:?} {:?}",
                    t.example, t.description, t.content
                );
            }
        }
        for w in &result.warnings {
            println!("WARNING    : {w}");
        }
    }
}
