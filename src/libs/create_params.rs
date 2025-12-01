use std::{
    fs::metadata,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use clap::Args;

use crate::server::params::{DEFAULT_MOCKS_DIR, DEFAULT_VERBOSE_ENABLED};

pub const DEFAULT_METHOD: &'static str = "GET";

#[derive(Args, Debug, Clone)]
pub struct CreateParams {
    #[arg(long, default_value = DEFAULT_METHOD, help = "Mock HTTP method")]
    pub method: String,

    #[arg(long, short, default_value_t = DEFAULT_VERBOSE_ENABLED, help = "Enable verbose logging")]
    pub verbose: bool,

    #[arg(long, short, default_value = DEFAULT_MOCKS_DIR, help = "Path to the directory containing mock files")]
    pub mocks: String,

    pub route: String,
}

impl CreateParams {
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
