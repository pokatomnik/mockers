use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

const FRONTMATTER_EDGE: &str = "---";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MockersPromptParams {
    #[serde(rename = "prompt")]
    prompt: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MockersFrontmatter {
    #[serde(rename = "$mockers")]
    mockers: Option<MockersPromptParams>,
}

pub(crate) struct PromptParser {
    source: String,
    processed: OnceLock<(Option<MockersFrontmatter>, String)>,
}

#[derive(Clone, Copy)]
enum ParseState {
    Start,
    Open,
    Closed,
}

impl PromptParser {
    pub fn new(source: impl AsRef<str>) -> Self {
        Self {
            source: source.as_ref().to_string(),
            processed: OnceLock::new(),
        }
    }

    fn get_prompt(&self) -> &(Option<MockersFrontmatter>, String) {
        self.processed
            .get_or_init(|| self.expect_process_source_typed())
    }

    fn expect_process_source_typed(&self) -> (Option<MockersFrontmatter>, String) {
        let Ok((frontmatter, content)) = self.process_source_typed() else {
            return (None, self.source.clone());
        };
        (frontmatter, content)
    }

    fn process_source_typed(&self) -> anyhow::Result<(Option<MockersFrontmatter>, String)> {
        let (frontmatter, content) = self.process_source();
        let Some(frontmatter) = frontmatter else {
            return Ok((None, content));
        };
        let parsed = yaml_serde::from_str::<MockersFrontmatter>(frontmatter.as_str())?;
        Ok((Some(parsed), content))
    }

    fn process_source(&self) -> (Option<String>, String) {
        if !self.source.starts_with(FRONTMATTER_EDGE) {
            return (None, self.source.clone());
        }

        let mut frontmatter_lines = Vec::new();
        let mut content_lines = Vec::new();

        let mut state = ParseState::Start;
        for line in self.source.as_str().lines() {
            match (line, state) {
                (line, ParseState::Start) if line == FRONTMATTER_EDGE => state = ParseState::Open,
                (line, ParseState::Open) if line == FRONTMATTER_EDGE => state = ParseState::Closed,
                // After the frontmatter is closed any line (including "---") belongs to the prompt body
                (_, ParseState::Start) => {}
                (_, ParseState::Open) => frontmatter_lines.push(line),
                (line, ParseState::Closed) => content_lines.push(line),
            }
        }

        let frontmatter = match frontmatter_lines.join("\n") {
            lines if lines.is_empty() => None,
            lines => Some(lines),
        };
        (frontmatter, content_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper that runs the parser and compares the result with the expected values.
    fn assert_processed(source: &str, expected_fm: Option<&str>, expected_body: &str) {
        let parser = PromptParser::new(source);
        let (fm, body) = parser.process_source();
        assert_eq!(fm.as_deref(), expected_fm, "frontmatter mismatch");
        assert_eq!(body, expected_body, "prompt body mismatch");
    }

    #[test]
    fn frontmatter_and_prompt() {
        let src = "---\ntitle: Hello\nauthor: Me\n---\nThis is the prompt.\nSecond line.";
        let expected_fm = Some("title: Hello\nauthor: Me");
        let expected_body = "This is the prompt.\nSecond line.";
        assert_processed(src, expected_fm, expected_body);
    }

    #[test]
    fn empty_source() {
        assert_processed("", None, "");
    }

    #[test]
    fn frontmatter_no_prompt() {
        let src = "---\nkey: value\n---";
        let expected_fm = Some("key: value");
        let expected_body = "";
        assert_processed(src, expected_fm, expected_body);
    }

    #[test]
    fn prompt_without_frontmatter() {
        let src = "Just a simple prompt line.\nAnother line.";
        let expected_fm = None;
        let expected_body = "Just a simple prompt line.\nAnother line.";
        assert_processed(src, expected_fm, expected_body);
    }

    #[test]
    fn extra_delimiters_inside_prompt() {
        let src =
            "---\nfoo: bar\n---\nPrompt starts here\n---\nand continues\n---\nwith more dashes.";
        let expected_fm = Some("foo: bar");
        let expected_body = "Prompt starts here\n---\nand continues\n---\nwith more dashes.";
        assert_processed(src, expected_fm, expected_body);
    }
}
