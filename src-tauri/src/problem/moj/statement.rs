//! `docs/enunciado.md`: the app keeps a statement as separate fields, MOJ keeps
//! one Markdown file split by `## Entrada` / `## Saída` / `## Observações`.
//! `render` joins the fields, `split` takes them apart again.
//!
//! The headings stay in Portuguese on purpose: MOJ's validation requires them.

use crate::{fs::to_lf, problem::ProblemStatement};

pub const INPUT_HEADING: &str = "## Entrada";
pub const OUTPUT_HEADING: &str = "## Saída";
pub const NOTES_HEADING: &str = "## Observações";

/// Same text with LF endings and no surrounding blank space.
pub fn clean(text: &str) -> String {
    to_lf(text).trim().to_string()
}

/// Legend, then input, output and (when there is one) notes, each under its
/// heading. Input and output are always written, even if empty.
pub fn render(stmt: &ProblemStatement) -> String {
    let mut sections = Vec::new();
    if !stmt.legend.trim().is_empty() {
        sections.push(clean(&stmt.legend));
    }
    sections.push(INPUT_HEADING.to_string());
    sections.push(clean(&stmt.input));
    sections.push(OUTPUT_HEADING.to_string());
    sections.push(clean(&stmt.output));
    if !stmt.notes.trim().is_empty() {
        sections.push(NOTES_HEADING.to_string());
        sections.push(clean(&stmt.notes));
    }
    sections.retain(|s| !s.is_empty());

    format!("{}\n", sections.join("\n\n"))
}

#[derive(Clone, Copy)]
enum Section {
    Legend,
    Input,
    Output,
    Notes,
}

#[derive(Debug, Default, PartialEq)]
pub struct StatementSections {
    pub legend: String,
    pub input: String,
    pub output: String,
    pub notes: String,
}

/// Lowercases and drops the accents of the few characters these headings use,
/// so `## Saída`, `## saida` and `## SAÍDA` all match.
fn normalize_title(title: &str) -> String {
    title
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ã' | 'â' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'õ' | 'ô' => 'o',
            'ú' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

fn section_of_heading(line: &str) -> Option<Section> {
    let title = line.trim_end().strip_prefix("## ")?;

    match normalize_title(title).as_str() {
        "entrada" => Some(Section::Input),
        "saida" => Some(Section::Output),
        "observacoes" => Some(Section::Notes),
        _ => None,
    }
}

/// Splits `enunciado.md` on its section headings. Whatever comes before the
/// first of them is the legend; any other heading stays inside the section it
/// appears in, and headings inside fenced code blocks are never treated as
/// section markers.
pub fn split(markdown: &str) -> StatementSections {
    let mut sections = StatementSections::default();
    let mut current = Section::Legend;
    let mut in_fence = false;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        }

        if !in_fence {
            if let Some(section) = section_of_heading(line) {
                current = section;
                continue;
            }
        }

        let target = match current {
            Section::Legend => &mut sections.legend,
            Section::Input => &mut sections.input,
            Section::Output => &mut sections.output,
            Section::Notes => &mut sections.notes,
        };
        target.push_str(line);
        target.push('\n');
    }

    for field in [
        &mut sections.legend,
        &mut sections.input,
        &mut sections.output,
        &mut sections.notes,
    ] {
        *field = field.trim().to_string();
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_statement_sections() {
        let s = split(
            "Legenda\r\n\r\n## Entrada\r\nem\r\n## Saída\r\nout\r\n## Observações\r\nnota\r\n",
        );

        assert_eq!(
            s,
            StatementSections {
                legend: "Legenda".into(),
                input: "em".into(),
                output: "out".into(),
                notes: "nota".into(),
            }
        );
    }

    #[test]
    fn heading_match_ignores_case_and_accents_but_not_other_headings() {
        let s = split("## ENTRADA\na\n## saida\nb\n## Restrições\nc\n");

        assert_eq!(s.input, "a");
        assert_eq!(s.output, "b\n## Restrições\nc");
    }

    #[test]
    fn headings_inside_code_fences_are_not_sections() {
        let s = split("## Entrada\n```\n## Saída\n```\nresto\n## Saída\nfim\n");

        assert_eq!(s.input, "```\n## Saída\n```\nresto");
        assert_eq!(s.output, "fim");
    }

    #[test]
    fn statement_without_headings_is_all_legend() {
        let s = split("só um texto");

        assert_eq!(s.legend, "só um texto");
        assert!(s.input.is_empty() && s.output.is_empty());
    }
}
