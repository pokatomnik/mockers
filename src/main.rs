mod cmd;
mod controllers;
mod libs;
mod middlewares;
mod server;

use clap::Parser;
use cmd::cli::Cli;
use std::error::Error as StdError;

use crate::cmd::commands::Commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Serve(server_params) => match server_params.test().await {
            Err(err) => Err(err),
            Ok(_) => server_params.start_server().await,
        },
        Commands::Create(create_params) => match create_params.test().await {
            Err(err) => Err(err),
            Ok(_) => create_params.create_mock().await,
        },
        Commands::Ls(ls_params) => ls_params.ls_mocks().await,
    };

    let result = result.map_err(|e| e.to_string());

    if let Err(err) = result {
        eprintln!("Finished with error: {}", err.to_string());
        return Err(err.into());
    }

    Ok(())
}
