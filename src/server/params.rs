use clap::{Args, arg};

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;

#[derive(Args, Debug, Clone)]
pub struct ServerParams {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: String,

    #[arg(long, short, default_value_t = DEFAULT_PORT)]
    pub port: u16,

    #[arg(long, short, default_value_t = false)]
    pub verbose: bool,
}
