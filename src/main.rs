mod cmd;
mod controllers;
mod libs;
mod middlewares;
mod server;

use clap::Parser;
use cmd::cli::Cli;
use libs::create_mock::create_mock;
use libs::ls::ls_mocks;
use server::server::start_server;
use std::error::Error as StdError;

use crate::cmd::commands::Commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let cli = Cli::parse();

    let result: Result<_, Box<dyn StdError>> = match cli.command {
        Commands::Serve(params) => match params.test().await {
            Err(err) => Err(err),
            Ok(_) => start_server(params)
                .await
                .map_err(|err| -> Box<dyn StdError> { err }),
        },
        Commands::Create(params) => match params.test().await {
            Err(err) => Err(err),
            Ok(_) => create_mock(params)
                .await
                .map_err(|err| -> Box<dyn StdError> { err }),
        },
        Commands::Ls(params) => ls_mocks(params)
            .await
            .map_err(|err| -> Box<dyn StdError> { err }),
    };

    if let Err(err) = result {
        eprintln!("Finished with error: {}", err.to_string());
        return Err(err);
    }

    return Ok(());
}
