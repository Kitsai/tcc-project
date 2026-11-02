//! What the MOJ "PACOTE" looks like on disk: every file/folder name and the
//! small text formats (`author`, `tags`, `.moj-meta.json`, `sampleN`, `sols/*`)
//! that the exporter writes and the importer reads. Keeping both directions
//! next to each other is what makes the round trip safe to change.

use crate::{
    error::AppResult,
    problem::SolutionTag,
};

pub const AUTHOR_FILE: &str = "author";
pub const TAGS_FILE: &str = "tags";
pub const CONF_FILE: &str = "conf";
pub const META_FILE: &str = ".moj-meta.json";
pub const STATEMENT_FILE: &str = "docs/enunciado.md";
pub const TUTORIAL_FILE: &str = "docs/solucao.md";
pub const SAMPLE_NOTES_DIR: &str = "docs/notes";
pub const TESTS_INPUT_DIR: &str = "tests/input";
pub const TESTS_OUTPUT_DIR: &str = "tests/output";
pub const SOLS_DIR: &str = "sols";
pub const CHECKER_FILE: &str = "scripts/checker.cpp";
pub const VALIDATOR_FILE: &str = "scripts/validator.cpp";
pub const COMPARE_FILE: &str = "scripts/compare.sh";

pub const SAMPLE_PREFIX: &str = "sample";

pub const GOOD_DIR: &str = "good";
pub const PASS_DIR: &str = "pass";
pub const SLOW_DIR: &str = "slow";
pub const WRONG_DIR: &str = "wrong";
pub const UPCOMING_DIR: &str = "upcoming";

/// `sols/*` folders in the order the importer reads them, with the tag the app
/// gives their files. `good` also supplies the main solution (the first one),
/// which the importer promotes itself.
pub const SOLUTION_FOLDERS: [(&str, SolutionTag); 5] = [
    (GOOD_DIR, SolutionTag::Accepted),
    (PASS_DIR, SolutionTag::Accepted),
    (SLOW_DIR, SolutionTag::TimeLimitExceeded),
    (WRONG_DIR, SolutionTag::WrongAnswer),
    (UPCOMING_DIR, SolutionTag::None),
];

/// The `sols/` folder a solution is exported to. `Main` and `Accepted` both map
/// to `good`: MOJ has no "main" notion and calibrates the time limit from the
/// slowest `good` solution. Not the exact inverse of `SOLUTION_FOLDERS`
/// (`pass` and `upcoming` are never written), and `None` means "skip it".
pub fn folder_for_tag(tag: &SolutionTag) -> Option<&'static str> {
    match tag {
        SolutionTag::Main | SolutionTag::Accepted => Some(GOOD_DIR),
        SolutionTag::WrongAnswer => Some(WRONG_DIR),
        SolutionTag::TimeLimitExceeded => Some(SLOW_DIR),
        SolutionTag::TimeLimitExceededOrAccepted
        | SolutionTag::TimeLimitExceededOrMemoryLimitExceeded
        | SolutionTag::MemoryLimitExceeded
        | SolutionTag::None => None,
    }
}

/// Name of the `n`-th (1-based) example test.
pub fn sample_name(n: usize) -> String {
    format!("{SAMPLE_PREFIX}{n}")
}

pub fn is_sample_name(name: &str) -> bool {
    name.to_lowercase().starts_with(SAMPLE_PREFIX)
}

pub fn render_author(author: &str) -> String {
    format!("{author}\n")
}

/// First non-blank line of the `author` file.
pub fn parse_author(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

fn clean_tag(tag: &str) -> &str {
    tag.trim().trim_start_matches('#').trim()
}

/// One `#tag` per line; blank tags are dropped.
pub fn render_tags(tags: &[String]) -> String {
    tags.iter()
        .map(|t| clean_tag(t))
        .filter(|t| !t.is_empty())
        .map(|t| format!("#{t}\n"))
        .collect()
}

pub fn parse_tags(text: &str) -> Vec<String> {
    text.lines()
        .map(clean_tag)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

/// `.moj-meta.json`: only content fields, since `owner`/`public` are
/// server-side and ignored on upload.
pub fn render_meta(display_title: &str) -> AppResult<String> {
    let meta = serde_json::json!({
        "display_title": display_title,
        "languages": [],
        "collections": [],
    });
    Ok(serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?)
}

pub fn parse_display_title(text: &str) -> Option<String> {
    let meta: serde_json::Value = serde_json::from_str(text).ok()?;
    let title = meta.get("display_title")?.as_str()?.trim();
    (!title.is_empty()).then(|| title.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_round_trip_and_drop_blank_ones() {
        let tags = vec!["greedy".to_string(), "#math".into(), "  ".into()];

        let rendered = render_tags(&tags);

        assert_eq!(rendered, "#greedy\n#math\n");
        assert_eq!(parse_tags(&rendered), ["greedy", "math"]);
    }

    #[test]
    fn author_takes_the_first_non_blank_line() {
        assert_eq!(parse_author("\n  Ana \nBia\n").as_deref(), Some("Ana"));
        assert_eq!(parse_author("  \n"), None);
        assert_eq!(parse_author(&render_author("Ana")).as_deref(), Some("Ana"));
    }

    #[test]
    fn meta_round_trips_the_title() {
        let meta = render_meta("Dobro").unwrap();

        assert_eq!(parse_display_title(&meta).as_deref(), Some("Dobro"));
        assert_eq!(parse_display_title(r#"{"display_title":"  "}"#), None);
        assert_eq!(parse_display_title("not json"), None);
    }

    #[test]
    fn sample_names_are_recognised_case_insensitively() {
        assert_eq!(sample_name(2), "sample2");
        assert!(is_sample_name("sample2") && is_sample_name("Sample10"));
        assert!(!is_sample_name("02"));
    }

    #[test]
    fn every_exported_folder_is_read_back() {
        for tag in [
            SolutionTag::Main,
            SolutionTag::Accepted,
            SolutionTag::WrongAnswer,
            SolutionTag::TimeLimitExceeded,
        ] {
            let folder = folder_for_tag(&tag).unwrap();
            assert!(SOLUTION_FOLDERS.iter().any(|(f, _)| *f == folder));
        }
        assert!(folder_for_tag(&SolutionTag::MemoryLimitExceeded).is_none());
    }
}
