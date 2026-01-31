use clap::Args;
use path_absolutize::Absolutize;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub const DEFAULT_HOST: &'static str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_MOCKS_DIR_NAME: &'static str = "mocks";
pub const DEFAULT_MOCKS_RESPONSE_DELAY: u64 = 0;
pub const DEFAULT_CORS_ENABLED: bool = false;
pub const DEFAULT_VERBOSE_ENABLED: bool = false;
pub const CONFIG_FILE_NAME: &'static str = "config.json";

#[derive(Args, Debug, Clone)]
pub struct ServerParams {
    #[arg(long, default_value = DEFAULT_HOST, help = "Host to listen on")]
    pub host: String,

    #[arg(long, short, default_value_t = DEFAULT_PORT, help = "Port to listen on")]
    pub port: u16,

    #[arg(long, short, default_value_t = false, help = "Enable verbose logging")]
    pub verbose: bool,

    #[arg(long, short, default_value = DEFAULT_MOCKS_DIR_NAME, help = "Path to the directory containing mock files")]
    pub mocks: String,

    #[arg(long, short, default_value_t = false, help = "Enable CORS headers")]
    pub cors: bool,

    #[arg(long, short, default_value_t = 0, help = "Mocks response delay")]
    pub delay_ms: u64,

    #[arg(long, short, help = "Origin server where")]
    pub origin: Option<String>,

    #[arg(
        long,
        short,
        help = "Admin base URL. The entry point for all admin URLs. Disabled by default"
    )]
    pub admin_base_url: Option<String>,
}

impl ServerParams {
    fn check_admin_base_url(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let path = &self.admin_base_url.as_ref().map(PathBuf::from);
        let Some(path) = path else {
            return Ok(());
        };
        if path.is_absolute() || path.starts_with("/") {
            return Ok(());
        }
        let message = format!(
            "Admin base URL should be absolute path: \"{}\"",
            path.to_string_lossy().to_string()
        );
        Err(message.into())
    }

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
        let path = Path::new(&self.mocks).to_owned().absolutize()?.into_owned();
        match path.is_absolute() {
            true => Ok(path),
            false => Ok(std::env::current_dir()?.join(&self.mocks)),
        }
    }

    pub async fn test(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        if let Err(e) = self.expect_mocks_path_to_exist().await {
            return Err(e);
        }

        if let Err(e) = self.check_admin_base_url() {
            return Err(e);
        }

        Ok(())
    }
}
