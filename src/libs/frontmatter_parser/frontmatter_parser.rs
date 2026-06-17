use std::sync::OnceLock;

use crate::libs::frontmatter_parser::{
    consts::FRONTMATTER_EDGE, mockers_frontmatter::MockersFrontmatter,
};

pub(crate) struct FrontmatterParser {
    source: String,
    processed: OnceLock<Option<MockersFrontmatter>>,
}

#[derive(Clone, Copy)]
enum ParseState {
    Start,
    Open,
}

impl FrontmatterParser {
    pub fn new(source: impl AsRef<str>) -> Self {
        Self {
            source: source.as_ref().to_string(),
            processed: OnceLock::new(),
        }
    }

    pub fn frontmatter(&self) -> Option<&MockersFrontmatter> {
        self.processed
            .get_or_init(|| self.expect_process_source_typed())
            .as_ref()
    }

    fn expect_process_source_typed(&self) -> Option<MockersFrontmatter> {
        let Ok(frontmatter) = self.process_source_typed() else {
            return None;
        };
        frontmatter
    }

    fn process_source_typed(&self) -> anyhow::Result<Option<MockersFrontmatter>> {
        let frontmatter = self.process_source();
        let Some(frontmatter) = frontmatter else {
            return Ok(None);
        };
        let parsed = yaml_serde::from_str::<MockersFrontmatter>(frontmatter.as_str())?;
        Ok(Some(parsed))
    }

    fn process_source(&self) -> Option<String> {
        if !self.source.starts_with(FRONTMATTER_EDGE) {
            return None;
        }

        let mut frontmatter_lines = Vec::new();

        let mut state = ParseState::Start;
        for line in self.source.as_str().lines() {
            match (line, state) {
                (line, ParseState::Start) if line == FRONTMATTER_EDGE => state = ParseState::Open,
                (line, ParseState::Open) if line == FRONTMATTER_EDGE => break,
                (_, ParseState::Start) => {}
                (_, ParseState::Open) => frontmatter_lines.push(line),
            }
        }

        let frontmatter = match frontmatter_lines.join("\n") {
            lines if lines.is_empty() => None,
            lines => Some(lines),
        };
        frontmatter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_prompt_frontmatter_and_prompt() {
        let src = "---\ntitle: Hello\nauthor: Me\n---\nThis is the prompt.\nSecond line.";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        assert!(fm.mockers().is_none());
    }

    #[test]
    fn get_prompt_empty_source() {
        let parser = FrontmatterParser::new("");
        let result = parser.frontmatter();

        assert!(result.is_none());
    }

    #[test]
    fn get_prompt_frontmatter_no_prompt() {
        let src = "---\nkey: value\n---";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        assert!(fm.mockers().is_none());
    }

    #[test]
    fn get_prompt_without_frontmatter() {
        let src = "Just a simple prompt line.\nAnother line.";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        assert!(result.is_none());
    }

    #[test]
    fn get_prompt_extra_delimiters_inside_prompt() {
        let src =
            "---\nfoo: bar\n---\nPrompt starts here\n---\nand continues\n---\nwith more dashes.";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        assert!(fm.mockers().is_none());
    }

    #[test]
    fn get_prompt_with_mockers_config() {
        let src = "---\n$mockers:\n  prompt: true\n  model: gpt-4\n---\nThis is the prompt body.";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        let mp = fm.mockers().expect("should have mockers config");
        assert_eq!(mp.prompt(), Some(true));
        assert_eq!(mp.model(), Some("gpt-4"));
    }

    #[test]
    fn get_prompt_with_mockers_prompt_false() {
        let src = "---\n$mockers:\n  prompt: false\n---\nBody";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        let mp = fm.mockers().expect("should have mockers config");
        assert_eq!(mp.prompt(), Some(false));
    }

    #[test]
    fn get_prompt_only_model() {
        let src = "---\n$mockers:\n  model: claude-3\n---\nBody";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        let mp = fm.mockers().expect("should have mockers config");
        assert_eq!(mp.model(), Some("claude-3"));
        assert!(mp.prompt().is_none());
    }

    #[test]
    fn get_prompt_with_api_endpoint() {
        let src = "---\n$mockers:\n  api_endpoint: https://api.openai.com/v1\n---\nBody";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        let mp = fm.mockers().expect("should have mockers config");
        assert_eq!(mp.api_endpoint(), Some("https://api.openai.com/v1"));
        assert!(mp.prompt().is_none());
        assert!(mp.model().is_none());
        assert!(mp.env_key().is_none());
        assert!(mp.proxy().is_none());
    }

    #[test]
    fn get_prompt_with_env_key() {
        let src = "---\n$mockers:\n  env_key: MY_CUSTOM_KEY\n---\nBody";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        let mp = fm.mockers().expect("should have mockers config");
        assert_eq!(mp.env_key(), Some("MY_CUSTOM_KEY"));
        assert!(mp.prompt().is_none());
        assert!(mp.model().is_none());
    }

    #[test]
    fn get_prompt_with_proxy() {
        let src = "---\n$mockers:\n  proxy: http://localhost:8080\n---\nBody";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        let mp = fm.mockers().expect("should have mockers config");
        assert_eq!(mp.proxy(), Some("http://localhost:8080"));
        assert!(mp.prompt().is_none());
        assert!(mp.model().is_none());
    }

    #[test]
    fn get_prompt_with_all_mockers_fields() {
        let src = "---\n$mockers:\n  prompt: true\n  model: claude-opus\n  api_endpoint: https://api.anthropic.com\n  env_key: ANTHROPIC_KEY\n  proxy: http://proxy:3128\n---\nFull config";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        let fm = result.expect("should parse frontmatter");
        let mp = fm.mockers().expect("should have mockers config");
        assert_eq!(mp.prompt(), Some(true));
        assert_eq!(mp.model(), Some("claude-opus"));
        assert_eq!(mp.api_endpoint(), Some("https://api.anthropic.com"));
        assert_eq!(mp.env_key(), Some("ANTHROPIC_KEY"));
        assert_eq!(mp.proxy(), Some("http://proxy:3128"));
    }

    #[test]
    fn get_prompt_invalid_yaml() {
        let src = "---\ngarbage: [unclosed\n---\nbody";
        let parser = FrontmatterParser::new(src);
        let result = parser.frontmatter();

        // invalid YAML should return None (not panic)
        assert!(result.is_none());
    }
}
