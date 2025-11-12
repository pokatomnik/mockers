mod cmd;
mod server;

use clap::Parser;
use cmd::cli::Cli;
use server::server::start_server;

use crate::cmd::commands::Commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    if let Err(err) = match cli.command {
        Commands::Serve { params } => start_server(&params).await,
    } {
        eprintln!("Finished with error: {}", err.to_string());
    }
    return Ok(());
}
