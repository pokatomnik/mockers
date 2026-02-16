use crate::libs::create_params::CreateParams;
use crate::libs::ls::LsParams;
use crate::server::params::ServerParams;

use crate::libs::info_params::InfoParams;
use clap::Subcommand;
use crate::libs::delete_params::DeleteParams;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[clap(visible_aliases = ["run", "start", "s"], about = "Run server")]
    Serve(ServerParams),

    #[clap(visible_aliases = ["new", "n"], about = "Create mock by path")]
    Create(CreateParams),

    #[clap(visible_aliases = ["ls"], about = "List all file-based mocks")]
    List(LsParams),

    #[clap(visible_aliases = ["show", "i"], about = "Show full info about a specific mock")]
    Info(InfoParams),

    #[clap(visible_aliases = ["remove", "rm", "r"], about = "Remove mock by name")]
    Delete(DeleteParams),
}
