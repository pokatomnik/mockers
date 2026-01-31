use std::error::Error as StdError;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use clap::Args;
use path_absolutize::Absolutize;

use crate::libs::cache_mode::CacheMode;
use crate::server::params::{DEFAULT_MOCKS_DIR_NAME, DEFAULT_VERBOSE_ENABLED};

pub(crate) const DEFAULT_METHOD: &'static str = "GET";
pub(crate) const DEFAULT_STATUS_CODE: u16 = 200;
pub(crate) const DEFAULT_DELAY_MS: u64 = 0;

#[derive(Args, Debug, Clone)]
pub struct CreateParams {
    #[arg(long, default_value = DEFAULT_METHOD, help = "Mock HTTP method")]
    method: String,

    #[arg(long, short, default_value_t = DEFAULT_STATUS_CODE, help = "HTTP status code")]
    status_code: u16,

    #[arg(long, short, default_value_t = DEFAULT_DELAY_MS, help = "Delay in milliseconds before respond")]
    delay_ms: u64,

    #[arg(long = "header", help = "Custom header, example: 'X-Server: Mockers'")]
    headers: Vec<String>,

    #[arg(long, short, help = "Contents of the mock")]
    contents: Option<String>,

    #[arg(long, short, default_value_t = DEFAULT_VERBOSE_ENABLED, help = "Enable verbose logging")]
    verbose: bool,

    #[arg(long, help = "Should the server response be cached")]
    cache_mode: Option<CacheMode>,

    #[arg(long, short, default_value = DEFAULT_MOCKS_DIR_NAME, help = "Path to the directory containing mock files")]
    mocks: String,

    route: String,
}

impl CreateParams {
    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn route(&self) -> &str {
        &self.route
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn delay_ms(&self) -> u64 {
        self.delay_ms
    }

    pub fn cache_mode(&self) -> Option<CacheMode> {
        return self.cache_mode.clone();
    }

    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.iter().filter_map(|v| {
            v.split_once(':')
                .map(|(k, v)| (k.trim(), v.trim()))
                .filter(|(k, v)| !k.is_empty() && !v.is_empty())
        })
    }

    pub fn contents(&self) -> Option<&str> {
        if let Some(ref c) = self.contents {
            return Some(c);
        }
        None
    }

    async fn expect_mocks_path_to_exist(&self) -> Result<(), Box<dyn StdError + Sync + Send>> {
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

    pub fn get_absolute_mocks_path(&self) -> Result<PathBuf, Box<dyn StdError + Sync + Send>> {
        let path = Path::new(&self.mocks).to_owned().absolutize()?.into_owned();
        match path.is_absolute() {
            true => Ok(path),
            false => {
                let cwd = std::env::current_dir()?;
                return Ok(cwd.join(&self.mocks));
            }
        }
    }

    pub async fn test(&self) -> Result<(), Box<dyn StdError + Sync + Send>> {
        if let Err(e) = self.expect_mocks_path_to_exist().await {
            return Err(e);
        }

        Ok(())
    }
}
