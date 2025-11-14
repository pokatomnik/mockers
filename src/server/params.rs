use clap::{Args, arg};
use std::fs::metadata;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_MOCKS_DIR: &str = "mocks";

#[derive(Args, Debug, Clone)]
pub struct ServerParams {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: String,

    #[arg(long, short, default_value_t = DEFAULT_PORT)]
    pub port: u16,

    #[arg(long, short, default_value_t = false)]
    pub verbose: bool,

    #[arg(long, short, default_value = DEFAULT_MOCKS_DIR)]
    pub mocks: String,
}

impl ServerParams {
    fn check_mocks_path(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let path = Path::new(&self.mocks);
        let metadata = metadata(path)?;
        if !metadata.is_dir() {
            let error = std::io::Error::new(
                ErrorKind::NotADirectory,
                format!("The specified path '{}' is not a directory", &self.mocks),
            );
            return Err(Box::new(error));
        }

        return Ok(());
    }

    pub fn get_absolute_mocks_path(
        &self,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Sync + Send>> {
        let path = Path::new(&self.mocks).to_owned();
        match path.is_absolute() {
            true => Ok(path),
            false => {
                let cwd = std::env::current_dir()?;
                return Ok(cwd.join(&self.mocks));
            }
        }
    }

    pub fn test(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let mocks_path_err = self.check_mocks_path();
        if mocks_path_err.is_err() {
            return mocks_path_err;
        }
        return Ok(());
    }
}
