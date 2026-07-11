use crate::cmd::commands::Doc;
use clap::Args;
use termimad::MadSkin;

#[derive(Args, Debug, Clone)]
pub(crate) struct DocParams {
    command: Doc,
}

impl DocParams {
    fn print_markdown(&self, markdown: &str) {
        let skin = MadSkin::default_dark();
        print!("{}", skin.term_text(markdown));
    }

    fn get_markdown_by_kind(&self, kind: Doc) -> &'static str {
        match kind {
            Doc::Serve => MD_DOC_SERVE,
            Doc::Create => MD_DOC_CREATE,
            Doc::List => MD_DOC_LIST,
            Doc::Info => MD_DOC_INFO,
            Doc::Delete => MD_DOC_DELETE,
            Doc::Enable => MD_DOC_ENABLE,
            Doc::Disable => MD_DOC_DISABLE,
            Doc::Config => MD_DOC_CONFIG,
            Doc::Init => MD_DOC_INIT,
            Doc::Completion => MD_DOC_COMPLETION,
            Doc::Doc => MD_DOC_HELP,
        }
    }

    fn print_markdown_by_kind(&self) {
        let markdown = self.get_markdown_by_kind(self.command);
        self.print_markdown(markdown);
    }

    pub async fn show_help(&self) -> anyhow::Result<()> {
        self.print_markdown_by_kind();
        Ok(())
    }
}

static MD_DOC_SERVE: &'static str = include_str!("../../../doc/serve.md");
static MD_DOC_CREATE: &'static str = include_str!("../../../doc/create.md");
static MD_DOC_LIST: &'static str = include_str!("../../../doc/list.md");
static MD_DOC_INFO: &'static str = include_str!("../../../doc/info.md");
static MD_DOC_DELETE: &'static str = include_str!("../../../doc/delete.md");
static MD_DOC_ENABLE: &'static str = include_str!("../../../doc/enable.md");
static MD_DOC_DISABLE: &'static str = include_str!("../../../doc/disable.md");
static MD_DOC_CONFIG: &'static str = include_str!("../../../doc/config.md");
static MD_DOC_INIT: &'static str = include_str!("../../../doc/init.md");
static MD_DOC_COMPLETION: &'static str = include_str!("../../../doc/completion.md");
static MD_DOC_HELP: &'static str = include_str!("../../../doc/doc.md");
