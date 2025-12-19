use super::commands::Commands;
use clap::Parser;

#[derive(Parser)]
#[command(name = "Mockers")]
#[command(about = "Simple mock server written in Rust")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
