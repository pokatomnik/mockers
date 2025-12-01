use crate::libs::create_params::CreateParams;

use super::super::server::params::ServerParams;
use clap::{Subcommand, command};

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Start server
    Serve {
        #[command(flatten)]
        params: ServerParams,
    },

    /// Create mocks by path
    Create(CreateParams),
}
