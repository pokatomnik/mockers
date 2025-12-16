use super::commands::Commands;
use clap::{command, Parser};

#[derive(Parser)]
#[command(name = "Mockers")]
#[command(about = "Simple mock server written in Rust")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
