use std::{fs, path::Path};

use serde::Serialize;

use crate::{
    compile_service::CompileService,
    constants::{CHECKER_TESTS_PATH, VALIDATOR_TESTS_PATH},
    error::{AppError, AppResult},
    polygon::xml::{self, PolygonProblem},
    problem::{
        create_problem_dirs, CheckerTest, CheckerVerdict, Problem, ProblemFileType,
        ProgrammingLanguage, SolutionDescription, ValidatorTest, ValidatorTestResult,
    },
    util::{Persistant, ResultExt},
};

#[derive(Serialize)]
pub struct ImportResult {
    pub problem: Problem,
    pub warnings: Vec<String>,
}

const STATEMENT_REQUIRED_FILES: [&str; 5] = ["name.tex", "legend.tex", "input.tex", "output.tex", "notes.tex"];

/// Imports a Polygon "full package" export at `source` into a brand-new
/// problem directory at `dest_path`. See the "Import Polygon Package" plan
/// for the full rationale; in short: `tests/main` (the judge test set) and
/// Polygon's category tags are intentionally not imported, checker/validator/
/// generator compilation is eager but soft-fails per file (the file is still
/// copied in, just left unselected, and a warning is returned instead), and
/// checker/validator regression self-tests are imported.
pub async fn import_polygon_package(
    source: &Path,
    dest_path: &Path,
    compile_service: &CompileService,
) -> AppResult<ImportResult> {
    if !source.is_dir() {
        return Err(AppError::from("Caminho de origem não é um diretório válido"));
    }

    let xml_path = source.join("problem.xml");
    if !xml_path.exists() {
        return Err(AppError::from(
            "Pasta selecionada não é um pacote Polygon (problem.xml não encontrado)",
        ));
    }

    let xml_text = fs::read_to_string(&xml_path).map_err(|e| AppError::from(format!("Falha ao ler problem.xml: {e}")))?;
    let parsed = xml::parse_problem_xml(&xml_text)?;

    if dest_path.exists() {
        return Err(AppError::from("Já existe uma pasta com esse nome no destino selecionado"));
    }

    let name = dest_path
        .file_name()
        .ok_or_else(|| AppError::from("Nome de destino inválido"))?
        .to_string_lossy()
        .into_owned();

    fs::create_dir_all(dest_path).err_to_string()?;
    create_problem_dirs(dest_path)?;

    let mut problem = Problem::create(&name, dest_path.to_path_buf());
    let mut warnings = Vec::new();

    if let Some(rel) = &parsed.checker_source {
        if let Some(file_name) =
            import_and_compile_file(&source.join(rel), dest_path, ProblemFileType::Checker, compile_service, &mut warnings).await
        {
            problem.definition.checker = Some(file_name);
        }
    }

    if let Some(rel) = &parsed.validator_source {
        if let Some(file_name) =
            import_and_compile_file(&source.join(rel), dest_path, ProblemFileType::Validator, compile_service, &mut warnings).await
        {
            problem.definition.validator = Some(file_name);
        }
    }
    if parsed.validator_count > 1 {
        warnings.push(format!(
            "O pacote possui {} validadores; apenas o primeiro foi importado.",
            parsed.validator_count
        ));
    }

    for rel in &parsed.generator_sources {
        if let Some(file_name) =
            import_and_compile_file(&source.join(rel), dest_path, ProblemFileType::Generator, compile_service, &mut warnings).await
        {
            if !problem.definition.generators.contains(&file_name) {
                problem.definition.generators.push(file_name);
            }
        }
    }

    import_solutions(source, dest_path, &parsed, &mut warnings)?;
    let solutions = SolutionDescription::load_all(dest_path)?;
    problem.definition.main_solution = SolutionDescription::main_solution_file_name(&solutions);

    problem.stmt = import_statement(source, &parsed, &mut warnings);

    import_checker_tests(source, dest_path, &parsed.checker_verdicts, &mut warnings)?;
    import_validator_tests(source, dest_path, &parsed.validator_verdicts, &mut warnings)?;

    problem.save_to_disk()?;

    Ok(ImportResult { problem, warnings })
}

/// Copies `source_abs` into `dest_path/<file_type's dir>/`, compiles it, and
/// returns the basename on success. Any failure (missing source, unrecognized
/// extension, compile error) is pushed onto `warnings` and yields `None` —
/// the file is still left copied where possible so the user can fix/select
/// it manually later.
async fn import_and_compile_file(
    source_abs: &Path,
    dest_path: &Path,
    file_type: ProblemFileType,
    compile_service: &CompileService,
    warnings: &mut Vec<String>,
) -> Option<String> {
    let Some(file_name) = source_abs.file_name().map(|f| f.to_string_lossy().into_owned()) else {
        warnings.push(format!("Caminho de arquivo inválido: {:?}", source_abs));
        return None;
    };

    let relative = Path::new(file_type.directory()).join(&file_name);
    let destination = dest_path.join(&relative);

    if let Err(e) = fs::copy(source_abs, &destination) {
        warnings.push(format!("Falha ao copiar '{file_name}' ({file_type}): {e}"));
        return None;
    }

    let Some(language) = ProgrammingLanguage::get_from_path(&relative) else {
        warnings.push(format!("'{file_name}' ({file_type}) tem uma extensão não reconhecida e não foi selecionado"));
        return None;
    };

    let _guard = compile_service.lock().await;
    if let Err(e) = compile_service.compile(&language, &relative, dest_path).await {
        warnings.push(format!("Falha ao compilar '{file_name}' ({file_type}): {e}"));
        return None;
    }

    Some(file_name)
}

fn import_solutions(source: &Path, dest_path: &Path, parsed: &PolygonProblem, warnings: &mut Vec<String>) -> AppResult<()> {
    for (rel, tag) in &parsed.solutions {
        let Some(file_name) = rel.file_name().map(|f| f.to_string_lossy().into_owned()) else {
            warnings.push(format!("Caminho de solução inválido: {:?}", rel));
            continue;
        };

        if let Err(e) = SolutionDescription::create_from_existing(source.join(rel), dest_path) {
            warnings.push(format!("Falha ao importar a solução '{file_name}': {e}"));
            continue;
        }

        let tag = match tag {
            Some(tag) => tag.clone(),
            None => {
                warnings.push(format!(
                    "Tag de solução desconhecida para '{file_name}', marcada como Accepted"
                ));
                continue;
            }
        };

        if !matches!(tag, crate::problem::SolutionTag::Accepted) {
            SolutionDescription::change_tag(dest_path, &file_name, tag)?;
        }
    }

    Ok(())
}

/// Resolves the first `statement-sections/<lang>/` directory (trying the
/// languages declared in `problem.xml` first, then falling back to scanning
/// the directory) that has all 5 required section files, and reads it into a
/// `ProblemStatement`. `tutorial.tex` is optional and defaults to `""`. If no
/// usable directory is found at all, a warning is pushed and an empty
/// statement (with just the parsed display name) is returned.
fn import_statement(source: &Path, parsed: &PolygonProblem, warnings: &mut Vec<String>) -> crate::problem::ProblemStatement {
    use crate::problem::ProblemStatement;

    let sections_root = source.join("statement-sections");

    let dir = parsed
        .statement_languages
        .iter()
        .map(|lang| sections_root.join(lang))
        .find(|dir| STATEMENT_REQUIRED_FILES.iter().all(|f| dir.join(f).exists()))
        .or_else(|| {
            fs::read_dir(&sections_root).ok()?.flatten().map(|e| e.path()).find(|dir| {
                dir.is_dir() && STATEMENT_REQUIRED_FILES.iter().all(|f| dir.join(f).exists())
            })
        });

    let Some(dir) = dir else {
        warnings.push("Nenhuma seção de enunciado utilizável foi encontrada no pacote".to_string());
        return ProblemStatement::new(&parsed.display_name);
    };

    let read = |file: &str| fs::read_to_string(dir.join(file)).unwrap_or_default();

    ProblemStatement {
        name: read("name.tex"),
        legend: read("legend.tex"),
        input: read("input.tex"),
        output: read("output.tex"),
        notes: read("notes.tex"),
        tutorial: read("tutorial.tex"),
    }
}

fn import_checker_tests(source: &Path, dest_path: &Path, verdicts: &[CheckerVerdict], warnings: &mut Vec<String>) -> AppResult<()> {
    let src_dir = source.join("files/tests/checker-tests");

    for (i, verdict) in verdicts.iter().enumerate() {
        let id = (i + 1) as u16;
        let stem = format!("{:02}", id);

        let (input, output, answer) = match (
            fs::read_to_string(src_dir.join(&stem)),
            fs::read_to_string(src_dir.join(format!("{stem}.o"))),
            fs::read_to_string(src_dir.join(format!("{stem}.a"))),
        ) {
            (Ok(i), Ok(o), Ok(a)) => (i, o, a),
            _ => {
                warnings.push(format!("Teste de checker {stem} ausente ou ilegível, ignorado"));
                continue;
            }
        };

        CheckerTest::new(id, &input, &output, &answer, verdict.clone()).save(&dest_path.join(CHECKER_TESTS_PATH).join(&stem))?;
    }

    Ok(())
}

fn import_validator_tests(
    source: &Path,
    dest_path: &Path,
    verdicts: &[ValidatorTestResult],
    warnings: &mut Vec<String>,
) -> AppResult<()> {
    let src_dir = source.join("files/tests/validator-tests");

    for (i, verdict) in verdicts.iter().enumerate() {
        let id = (i + 1) as u16;
        let stem = format!("{:02}", id);

        let input = match fs::read_to_string(src_dir.join(&stem)) {
            Ok(input) => input,
            Err(_) => {
                warnings.push(format!("Teste de validador {stem} ausente ou ilegível, ignorado"));
                continue;
            }
        };

        ValidatorTest::new(id, &input, verdict.clone()).save(&dest_path.join(VALIDATOR_TESTS_PATH).join(&stem))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        problem::SolutionTag,
        runner::Runner,
        test_support::MockRunner,
    };

    fn build_package(dir: &Path) {
        fs::create_dir_all(dir.join("files")).unwrap();
        fs::create_dir_all(dir.join("solutions")).unwrap();
        fs::create_dir_all(dir.join("statement-sections/english")).unwrap();
        fs::create_dir_all(dir.join("files/tests/checker-tests")).unwrap();
        fs::create_dir_all(dir.join("files/tests/validator-tests")).unwrap();

        fs::write(dir.join("files/checker.cpp"), "int main(){return 0;}").unwrap();
        fs::write(dir.join("files/validator.cpp"), "int main(){return 0;}").unwrap();
        fs::write(dir.join("solutions/main.cpp"), "int main(){return 0;}").unwrap();

        for (name, content) in [
            ("name.tex", "Test Problem"),
            ("legend.tex", "Legend"),
            ("input.tex", "Input"),
            ("output.tex", "Output"),
            ("notes.tex", "Notes"),
        ] {
            fs::write(dir.join("statement-sections/english").join(name), content).unwrap();
        }

        fs::write(dir.join("files/tests/checker-tests/01"), "in").unwrap();
        fs::write(dir.join("files/tests/checker-tests/01.o"), "out").unwrap();
        fs::write(dir.join("files/tests/checker-tests/01.a"), "ans").unwrap();
        fs::write(dir.join("files/tests/validator-tests/01"), "5\n").unwrap();

        let xml = r#"<problem>
            <names><name language="english" value="Test Problem"/></names>
            <statements><statement language="english" path="statements/english/problem.tex"/></statements>
            <files><executables>
                <executable><source path="files/checker.cpp"/></executable>
                <executable><source path="files/validator.cpp"/></executable>
            </executables></files>
            <assets>
                <checker><source path="files/checker.cpp"/>
                    <testset><tests><test verdict="ok"/></tests></testset>
                </checker>
                <validators><validator><source path="files/validator.cpp"/>
                    <testset><tests><test verdict="valid"/></tests></testset>
                </validator></validators>
                <solutions>
                    <solution tag="main"><source path="solutions/main.cpp"/></solution>
                </solutions>
            </assets>
        </problem>"#;
        fs::write(dir.join("problem.xml"), xml).unwrap();
    }

    #[tokio::test]
    async fn imports_a_synthetic_package_end_to_end() {
        let source_dir = tempfile::tempdir().unwrap();
        build_package(source_dir.path());

        let dest_dir = tempfile::tempdir().unwrap();
        let dest_path = dest_dir.path().join("imported");

        let runner: Arc<dyn Runner> = Arc::new(MockRunner::exit_code(0));
        let compile_service = CompileService::new(runner);

        let result = import_polygon_package(source_dir.path(), &dest_path, &compile_service)
            .await
            .unwrap();

        assert_eq!(result.problem.definition.checker, Some("checker.cpp".to_string()));
        assert_eq!(result.problem.definition.validator, Some("validator.cpp".to_string()));
        assert_eq!(result.problem.definition.main_solution, Some("main.cpp".to_string()));
        assert_eq!(result.problem.stmt.legend, "Legend");
        assert!(dest_path.join(CHECKER_TESTS_PATH).join("01").exists());
        assert!(dest_path.join(VALIDATOR_TESTS_PATH).join("01").exists());

        let solutions = SolutionDescription::load_all(&dest_path).unwrap();
        assert_eq!(solutions.len(), 1);
        assert!(matches!(solutions[0].tag, SolutionTag::Main));
    }

    #[tokio::test]
    async fn soft_fails_a_checker_that_does_not_compile() {
        let source_dir = tempfile::tempdir().unwrap();
        build_package(source_dir.path());

        let dest_dir = tempfile::tempdir().unwrap();
        let dest_path = dest_dir.path().join("imported");

        let runner: Arc<dyn Runner> = Arc::new(MockRunner::exit_code(1));
        let compile_service = CompileService::new(runner);

        let result = import_polygon_package(source_dir.path(), &dest_path, &compile_service)
            .await
            .unwrap();

        assert_eq!(result.problem.definition.checker, None);
        assert!(dest_path.join("files/checker.cpp").exists());
        assert!(result.warnings.iter().any(|w| w.contains("checker.cpp")));
    }

    #[tokio::test]
    async fn errors_when_source_has_no_problem_xml() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        let dest_path = dest_dir.path().join("imported");

        let runner: Arc<dyn Runner> = Arc::new(MockRunner::unreachable());
        let compile_service = CompileService::new(runner);

        let err = import_polygon_package(source_dir.path(), &dest_path, &compile_service)
            .await
            .err()
            .unwrap();
        assert!(err.to_string().contains("problem.xml"));
    }

    #[tokio::test]
    async fn errors_when_destination_already_exists() {
        let source_dir = tempfile::tempdir().unwrap();
        build_package(source_dir.path());

        let dest_dir = tempfile::tempdir().unwrap();
        let dest_path = dest_dir.path().join("imported");
        fs::create_dir_all(&dest_path).unwrap();

        let runner: Arc<dyn Runner> = Arc::new(MockRunner::unreachable());
        let compile_service = CompileService::new(runner);

        let err = import_polygon_package(source_dir.path(), &dest_path, &compile_service)
            .await
            .err()
            .unwrap();
        assert!(err.to_string().contains("Já existe"));
    }
}