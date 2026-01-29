use crate::libs::create_params::CreateParams;
use crate::libs::ls::LsParams;
use crate::server::params::ServerParams;

use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Start server
    Serve(ServerParams),

    /// Create mocks by path
    Create(CreateParams),

    /// List all file-based mocks
    Ls(LsParams),
}
