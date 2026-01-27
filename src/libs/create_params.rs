use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use clap::Args;
use path_absolutize::Absolutize;

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
    async fn expect_mocks_path_to_exist(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let path = Path::new(&self.mocks);
        let metadata_result = tokio::fs::metadata(path).await;

        if let Ok(ref metadata) = metadata_result
            && metadata.is_dir()
        {
            return Ok(());
        }

        if let Ok(ref metadata) = metadata_result
            && (metadata.is_file() || metadata.is_symlink())
        {
            let error = std::io::Error::new(
                ErrorKind::NotADirectory,
                format!("The specified path '{}' is not a directory", &self.mocks),
            );
            return Err(error.into());
        }

        if path.is_absolute() {
            let absolute_path = path.absolutize()?;
            tokio::fs::create_dir_all(absolute_path).await?;
            return Ok(());
        }

        let cwd = std::env::current_dir()?;
        let absolute_path: PathBuf = cwd.join(&path).absolutize()?.into();
        tokio::fs::create_dir_all(absolute_path).await?;

        Ok(())
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

    pub async fn test(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        if let Err(e) = self.expect_mocks_path_to_exist().await {
            return Err(e);
        }

        Ok(())
    }
}
