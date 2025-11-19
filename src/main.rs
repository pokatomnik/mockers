mod cmd;
mod server;

use clap::Parser;
use cmd::cli::Cli;
use server::server::start_server;

use crate::cmd::commands::Commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let result: Result<_, Box<dyn std::error::Error>> = match cli.command {
        Commands::Serve { params } => match params.test() {
            Err(err) => Err(err),
            Ok(_) => start_server(params)
                .await
                .map_err(|err| -> Box<dyn std::error::Error> { err }),
        },
    };

    if let Err(err) = result {
        eprintln!("Finished with error: {}", err.to_string());
        return Err(err);
    }

    return Ok(());
}
