use std::path::PathBuf;

use roxmltree::{Document, Node};

use crate::{
    error::{AppError, AppResult},
    problem::{CheckerVerdict, SolutionTag, ValidatorTestResult},
};

/// Everything this app's importer needs, extracted from a Polygon package's
/// `problem.xml`. Paths are relative to the package root.
pub struct PolygonProblem {
    pub display_name: String,
    pub checker_source: Option<PathBuf>,
    pub validator_source: Option<PathBuf>,
    /// Number of `<validator>` entries actually declared, so the caller can
    /// warn when more than one exists (only the first is imported).
    pub validator_count: usize,
    pub generator_sources: Vec<PathBuf>,
    /// `None` means the raw `tag` string on that `<solution>` was not a
    /// recognized `SolutionTag` value; the caller decides the fallback and warns.
    pub solutions: Vec<(PathBuf, Option<SolutionTag>)>,
    /// `<statements>` languages in document order.
    pub statement_languages: Vec<String>,
    /// Document order = checker regression test id 1..N.
    pub checker_verdicts: Vec<CheckerVerdict>,
    /// Document order = validator regression test id 1..N.
    pub validator_verdicts: Vec<ValidatorTestResult>,
    /// The judge test set, in document order.
    pub main_tests: Vec<PolygonMainTest>,
}

/// One entry from `<judging><testset><tests><test .../></tests></testset></judging>`.
/// `cmd: None` means `method="manual"` — the test's content is a literal
/// file already in the package at `tests/%02d` (document position,
/// 1-indexed). `cmd: Some(...)` means `method="generated"` — the cmd string
/// is already in this app's own `<generator> <args...>` Script-content
/// format verbatim.
pub struct PolygonMainTest {
    pub cmd: Option<String>,
    pub sample: bool,
    pub description: Option<String>,
}

pub fn parse_problem_xml(xml: &str) -> AppResult<PolygonProblem> {
    let doc = Document::parse(xml).map_err(|e| AppError::from(format!("problem.xml inválido: {e}")))?;
    let root = doc.root_element();

    let display_name = find_child(&root, "names")
        .and_then(|names| names.children().find(|c| c.has_tag_name("name")))
        .and_then(|name| {
            name.attribute("value")
                .or_else(|| name.attribute("name"))
                .map(str::to_string)
        })
        .unwrap_or_else(|| "Imported Problem".to_string());

    // Each language typically appears once per statement format (tex/html/pdf),
    // so dedupe while preserving first-seen order.
    let mut statement_languages: Vec<String> = Vec::new();
    if let Some(statements) = find_child(&root, "statements") {
        for lang in statements
            .children()
            .filter(|c| c.has_tag_name("statement"))
            .filter_map(|s| s.attribute("language"))
        {
            if !statement_languages.iter().any(|l| l == lang) {
                statement_languages.push(lang.to_string());
            }
        }
    }

    let all_executables: Vec<PathBuf> = find_child(&root, "files")
        .and_then(|files| find_child(&files, "executables"))
        .map(|execs| {
            execs
                .children()
                .filter(|c| c.has_tag_name("executable"))
                .filter_map(|e| find_child(&e, "source"))
                .filter_map(|source| source.attribute("path").map(PathBuf::from))
                .collect()
        })
        .unwrap_or_default();

    let assets = find_child(&root, "assets");

    let checker = assets.as_ref().and_then(|a| find_child(a, "checker"));
    let checker_source = checker
        .as_ref()
        .and_then(|c| find_child(c, "source"))
        .and_then(|s| s.attribute("path"))
        .map(PathBuf::from);
    let checker_verdicts = checker
        .as_ref()
        .and_then(|c| find_child(c, "testset"))
        .map(|ts| collect_verdicts(&ts))
        .unwrap_or_default();

    let validators: Vec<Node> = assets
        .as_ref()
        .and_then(|a| find_child(a, "validators"))
        .map(|v| v.children().filter(|c| c.has_tag_name("validator")).collect())
        .unwrap_or_default();
    let validator_count = validators.len();
    let first_validator = validators.first();
    let validator_source = first_validator
        .and_then(|v| find_child(v, "source"))
        .and_then(|s| s.attribute("path"))
        .map(PathBuf::from);
    let validator_verdicts = first_validator
        .and_then(|v| find_child(v, "testset"))
        .map(|ts| collect_verdicts(&ts))
        .unwrap_or_default();

    let generator_sources: Vec<PathBuf> = all_executables
        .into_iter()
        .filter(|p| Some(p) != checker_source.as_ref() && Some(p) != validator_source.as_ref())
        .collect();

    let solutions: Vec<(PathBuf, Option<SolutionTag>)> = assets
        .as_ref()
        .and_then(|a| find_child(a, "solutions"))
        .map(|solutions| {
            solutions
                .children()
                .filter(|c| c.has_tag_name("solution"))
                .filter_map(|s| {
                    let path = find_child(&s, "source")?.attribute("path")?.to_string();
                    let tag = s.attribute("tag").map(map_solution_tag).unwrap_or(Some(SolutionTag::Accepted));
                    Some((PathBuf::from(path), tag))
                })
                .collect()
        })
        .unwrap_or_default();

    let main_tests: Vec<PolygonMainTest> = find_child(&root, "judging")
        .and_then(|j| find_child(&j, "testset"))
        .and_then(|ts| find_child(&ts, "tests"))
        .map(|tests| {
            tests
                .children()
                .filter(|c| c.has_tag_name("test"))
                .map(|t| PolygonMainTest {
                    cmd: t.attribute("cmd").map(str::to_string),
                    sample: t.attribute("sample") == Some("true"),
                    description: t.attribute("description").map(str::to_string),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(PolygonProblem {
        display_name,
        checker_source,
        validator_source,
        validator_count,
        generator_sources,
        solutions,
        statement_languages,
        checker_verdicts,
        validator_verdicts,
        main_tests,
    })
}

fn find_child<'a, 'input>(node: &Node<'a, 'input>, tag: &str) -> Option<Node<'a, 'input>> {
    node.children().find(|c| c.has_tag_name(tag))
}

/// Reads `<testset>...<tests><test verdict="..."/>...</tests></testset>` in
/// document order (document order = test id 1..N). Verdicts are lowercase and
/// hyphen-separated in Polygon's export (`presentation-error`); both
/// `CheckerVerdict` and `ValidatorTestResult` already parse
/// `SCREAMING_SNAKE_CASE`/`UPPERCASE`, so this only needs to normalize the
/// separator and case before reusing their existing `FromStr` impls.
fn collect_verdicts<T: std::str::FromStr>(testset: &Node) -> Vec<T> {
    let Some(tests) = find_child(testset, "tests") else {
        return Vec::new();
    };

    tests
        .children()
        .filter(|c| c.has_tag_name("test"))
        .filter_map(|t| t.attribute("verdict"))
        .filter_map(|raw| normalize_verdict(raw).parse().ok())
        .collect()
}

fn normalize_verdict(raw: &str) -> String {
    raw.to_uppercase().replace('-', "_")
}

/// Maps a Polygon solution `tag` attribute (lowercase-hyphenated, e.g.
/// `"time-limit-exceeded-or-accepted"`) onto this app's `SolutionTag`, which
/// is `SCREAMING_SNAKE_CASE` on the wire. Returns `None` for an unrecognized
/// string so the caller can decide the fallback and warn instead of this
/// function silently picking one.
fn map_solution_tag(raw: &str) -> Option<SolutionTag> {
    let normalized = raw.to_uppercase().replace('-', "_");
    serde_json::from_value(serde_json::Value::String(normalized)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CABO_CARENTE_XML: &str = include_str!("test_fixtures/cabo_carente_problem.xml");
    const EXTREME_SUB_XML: &str = include_str!("test_fixtures/extreme_sub_problem.xml");

    #[test]
    fn parses_cabo_carente_sample() {
        let parsed = parse_problem_xml(CABO_CARENTE_XML).unwrap();

        assert_eq!(parsed.display_name, "Cabo Carente");
        assert_eq!(parsed.checker_source, Some(PathBuf::from("files/check.cpp")));
        assert_eq!(parsed.validator_source, Some(PathBuf::from("files/validator.cpp")));
        assert_eq!(parsed.validator_count, 1);
        assert_eq!(parsed.generator_sources, vec![PathBuf::from("files/generator.cpp")]);
        assert_eq!(parsed.statement_languages, vec!["english".to_string()]);
        assert!(parsed.checker_verdicts.is_empty());
        assert_eq!(parsed.validator_verdicts.len(), 9);
        assert!(matches!(parsed.validator_verdicts[0], ValidatorTestResult::Valid));
        assert!(matches!(parsed.validator_verdicts[2], ValidatorTestResult::Invalid));

        assert_eq!(parsed.solutions.len(), 4);
        let main = parsed
            .solutions
            .iter()
            .find(|(p, _)| p == &PathBuf::from("solutions/solution.cpp"))
            .unwrap();
        assert!(matches!(main.1, Some(SolutionTag::Main)));
        let wrong = parsed
            .solutions
            .iter()
            .find(|(p, _)| p == &PathBuf::from("solutions/solutionErrada.cpp"))
            .unwrap();
        assert!(matches!(wrong.1, Some(SolutionTag::WrongAnswer)));

        assert_eq!(parsed.main_tests.len(), 100);
        assert!(parsed.main_tests.iter().all(|t| t.cmd.as_deref() == Some("generator")));
        assert!(parsed.main_tests[0].sample);
        assert!(parsed.main_tests[1].sample);
        assert!(!parsed.main_tests[2].sample);
    }

    #[test]
    fn parses_extreme_sub_sample() {
        let parsed = parse_problem_xml(EXTREME_SUB_XML).unwrap();

        assert_eq!(parsed.display_name, "Extreme Subtraction");
        assert_eq!(parsed.solutions.len(), 7);
        assert_eq!(parsed.checker_verdicts.len(), 8);
        assert!(matches!(parsed.checker_verdicts[0], CheckerVerdict::Ok));
        assert!(matches!(parsed.checker_verdicts[1], CheckerVerdict::PresentationError));
        assert!(matches!(parsed.checker_verdicts[2], CheckerVerdict::Crashed));
        assert_eq!(parsed.validator_verdicts.len(), 10);

        let tle_or_accepted = parsed
            .solutions
            .iter()
            .find(|(p, _)| p == &PathBuf::from("solutions/TleC.cpp"))
            .unwrap();
        assert!(matches!(tle_or_accepted.1, Some(SolutionTag::TimeLimitExceededOrAccepted)));

        assert_eq!(parsed.main_tests.len(), 37);
        assert!(parsed.main_tests[0].cmd.is_none());
        assert!(parsed.main_tests[0].sample);
        assert_eq!(parsed.main_tests[0].description.as_deref(), Some("example test"));
        assert_eq!(
            parsed.main_tests[1].cmd.as_deref(),
            Some("gen-v1 -test-count 10000 -sum-n 30000 -yes-count 5000 -min-a 1 -max-a 10 -min-b 1 -max-b 10")
        );
    }

    #[test]
    fn missing_name_value_falls_back_to_default() {
        let xml = r#"<problem><names><name language="english"/></names></problem>"#;
        let parsed = parse_problem_xml(xml).unwrap();
        assert_eq!(parsed.display_name, "Imported Problem");
    }

    #[test]
    fn unrecognized_solution_tag_yields_none() {
        assert!(map_solution_tag("not-a-real-tag").is_none());
        assert!(matches!(map_solution_tag("main"), Some(SolutionTag::Main)));
    }

    #[test]
    fn malformed_xml_is_an_error() {
        assert!(parse_problem_xml("<not valid").is_err());
    }
}