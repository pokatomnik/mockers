use crate::cmd::cli::Cli;
use clap::{Args, CommandFactory};
use clap_complete::{Shell, generate};
use std::error::Error as StdError;

#[derive(Args, Debug, Clone)]
pub(crate) struct CompletionParams {
    #[arg(long, short, default_value_t = Shell::Bash, help = "Shell to generate completion script for")]
    shell: Shell,
}

impl CompletionParams {
    pub async fn generate(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let mut cmd = Cli::command();
        let name = &cmd.get_name().to_string();
        generate(self.shell, &mut cmd, name, &mut std::io::stdout());
        Ok(())
    }
}
