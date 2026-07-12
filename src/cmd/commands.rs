use crate::cmd::controllers::completion_params::CompletionParams;
use crate::cmd::controllers::config::ConfigParams;
use crate::cmd::controllers::init_params::InitParams;
use crate::server::params::ServerParams;
use clap::Subcommand;

#[derive(Subcommand, Clone)]
pub enum Commands {
    #[clap(visible_aliases = ["run", "start", "s"], about = "Run server")]
    Serve(ServerParams),

    #[clap(
        visible_aliases = ["configuration", "settings", "preferences", "prefs"],
        about = "Show effective global Mockers configuration"
    )]
    Config(ConfigParams),

    #[clap(visible_aliases = ["setup"], about = "Initialize Mockers global configuration")]
    Init(InitParams),

    #[clap(about = "Prepare and print shell completion script")]
    Completion(CompletionParams),
}
