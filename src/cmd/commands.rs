use crate::libs::create_params::CreateParams;
use crate::libs::ls::LsParams;
use crate::server::params::ServerParams;

use clap::Subcommand;
use crate::libs::info_params::InfoParams;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[clap(visible_aliases = ["run", "start", "s"], about = "Run server")]
    Serve(ServerParams),

    #[clap(visible_aliases = ["new", "n"], about = "Create mock by path")]
    Create(CreateParams),

    #[clap(visible_aliases = ["list"], about = "List all file-based mocks")]
    Ls(LsParams),
    
    #[clap(visible_aliases = ["show"], about = "Show full info about a specific mock")]
    Info(InfoParams)
}
