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

    pub fn get_prompt(&self) -> &(Option<MockersFrontmatter>, String) {
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

    #[test]
    fn get_prompt_frontmatter_and_prompt() {
        let src = "---\ntitle: Hello\nauthor: Me\n---\nThis is the prompt.\nSecond line.";
        let parser = PromptParser::new(src);
        let (frontmatter, body) = parser.get_prompt();

        assert!(matches!(
            frontmatter,
            Some(MockersFrontmatter { mockers: None })
        ));
        assert_eq!(body, "This is the prompt.\nSecond line.");
    }

    #[test]
    fn get_prompt_empty_source() {
        let parser = PromptParser::new("");
        let (frontmatter, body) = parser.get_prompt();

        assert!(frontmatter.is_none());
        assert_eq!(body, "");
    }

    #[test]
    fn get_prompt_frontmatter_no_prompt() {
        let src = "---\nkey: value\n---";
        let parser = PromptParser::new(src);
        let (frontmatter, body) = parser.get_prompt();

        assert!(matches!(
            frontmatter,
            Some(MockersFrontmatter { mockers: None })
        ));
        assert_eq!(body, "");
    }

    #[test]
    fn get_prompt_without_frontmatter() {
        let src = "Just a simple prompt line.\nAnother line.";
        let parser = PromptParser::new(src);
        let (frontmatter, body) = parser.get_prompt();

        assert!(frontmatter.is_none());
        assert_eq!(body, "Just a simple prompt line.\nAnother line.");
    }

    #[test]
    fn get_prompt_extra_delimiters_inside_prompt() {
        let src =
            "---\nfoo: bar\n---\nPrompt starts here\n---\nand continues\n---\nwith more dashes.";
        let parser = PromptParser::new(src);
        let (frontmatter, body) = parser.get_prompt();

        assert!(matches!(
            frontmatter,
            Some(MockersFrontmatter { mockers: None })
        ));
        assert_eq!(
            body,
            "Prompt starts here\n---\nand continues\n---\nwith more dashes."
        );
    }
}
