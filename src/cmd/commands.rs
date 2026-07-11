use crate::cmd::controllers::activity_params::ActivityParams;
use crate::cmd::controllers::completion_params::CompletionParams;
use crate::cmd::controllers::create_params::CreateParams;
use crate::cmd::controllers::doc_params::DocParams;
use crate::cmd::controllers::init_params::InitParams;
use crate::cmd::controllers::ls_params::LsParams;
use crate::server::params::ServerParams;

use crate::cmd::controllers::config::ConfigParams;
use crate::cmd::controllers::delete_params::DeleteParams;
use crate::cmd::controllers::info_params::InfoParams;
use clap::Subcommand;
use strum_macros::EnumDiscriminants;

#[derive(Subcommand, Clone, EnumDiscriminants)]
#[strum_discriminants(name(Doc), vis(pub(crate)), derive(clap::ValueEnum))]
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

    #[clap(visible_aliases = ["on"], about = "Enable mock if It exists")]
    Enable(ActivityParams),

    #[clap(visible_aliases = ["off"], about = "Disable mock if It exists")]
    Disable(ActivityParams),

    #[clap(visible_aliases = ["configuration", "settings", "preferences", "prefs"], about = "Show global Mockers configuration")]
    Config(ConfigParams),

    #[clap(visible_aliases = ["setup"], about = "Initialize Mockers global configuration")]
    Init(InitParams),

    #[clap(about = "Prepare and print shell completion script")]
    Completion(CompletionParams),

    #[clap(about = "Show documentation by command")]
    Doc(DocParams),
}
