use super::super::server::params::ServerParams;
use clap::{Subcommand, command};

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Serve {
        #[command(flatten)]
        params: ServerParams,
    },
}
