mod cmd;
mod controllers;
mod libs;
mod middlewares;
mod server;

use clap::Parser;
use cmd::cli::Cli;
use log::{LevelFilter, error};
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use std::error::Error as StdError;

use crate::cmd::commands::Commands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    init_logger();
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
        Commands::List(ls_params) => ls_params.ls_mocks().await,
        Commands::Info(info_params) => info_params.show_info().await,
        Commands::Delete(delete_params) => delete_params.delete_mock().await,
        Commands::Enable(activity_params) => activity_params.enable().await,
        Commands::Disable(activity_params) => activity_params.disable().await,
        Commands::Config(config_params) => Ok(config_params.show_config().await),
        Commands::Init(init_params) => init_params.init().await,
        Commands::Completion(completion_params) => completion_params.generate().await,
        Commands::Doc(help_params) => help_params.show_help().await,
    };

    let result = result.map_err(|e| e.to_string());

    if let Err(err) = result {
        error!("Finished with error: {}", err.to_string());
        return Err(err.into());
    }

    Ok(())
}

fn init_logger() {
    let mut config = ConfigBuilder::new();

    config
        .set_time_level(LevelFilter::Info)
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_time_format_rfc3339();

    let _ = config.set_time_offset_to_local();

    TermLogger::init(
        LevelFilter::Info,
        config.build(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    )
    .unwrap();
}
