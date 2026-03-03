use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::Metadata;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::libs::absolute_mocks_path::{AbsoluteMocksPath, WithMocks};
use crate::libs::cache_mode::CacheMode;
use crate::libs::global_config::{GlobalConfigAPI, WithGlobalConfigAPI};
use crate::libs::mock_config::MockConfig;
use crate::libs::path_buf_ext::PathBufExt;
use crate::server::params::CONFIG_FILE_NAME;
use clap::Args;
use path_absolutize::Absolutize;
use serde_json::json;
use tokio::sync::OnceCell;
use tokio::try_join;

pub(crate) const DEFAULT_METHOD: &'static str = "GET";
pub(crate) const DEFAULT_STATUS_CODE: u16 = 200;
pub(crate) const DEFAULT_DELAY_MS: u64 = 0;

#[derive(Args, Debug, Clone)]
pub struct CreateParams {
    #[arg(long, default_value = DEFAULT_METHOD, help = "Mock HTTP method")]
    method: String,

    #[arg(long, short, default_value_t = DEFAULT_STATUS_CODE, help = "HTTP status code")]
    status_code: u16,

    #[arg(long, short, help = "Delay in milliseconds before respond")]
    delay_ms: Option<u64>,

    #[arg(long = "header", help = "Custom header, example: 'X-Server: Mockers'")]
    headers: Vec<String>,

    #[arg(long, short, help = "Contents of the mock")]
    contents: Option<String>,

    #[arg(long, help = "Should the server response be cached")]
    cache_mode: Option<CacheMode>,

    #[arg(long, short, help = "Path to the directory containing mock files")]
    mocks: Option<String>,

    #[arg(long, default_value_t = false, help = "Should mock be disabled or not")]
    disabled: bool,

    /// Pathname to create mock for
    route: String,

    #[clap(skip)]
    global_config: OnceCell<GlobalConfigAPI>,
}

impl CreateParams {
    fn method(&self) -> &str {
        &self.method
    }

    fn route(&self) -> &str {
        &self.route
    }

    fn status_code(&self) -> u16 {
        self.status_code
    }

    fn cache_mode(&self) -> Option<CacheMode> {
        self.cache_mode.clone()
    }

    fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.iter().filter_map(|v| {
            v.split_once(':')
                .map(|(k, v)| (k.trim(), v.trim()))
                .filter(|(k, v)| !k.is_empty() && !v.is_empty())
        })
    }

    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn contents(&self) -> Option<&str> {
        if let Some(ref c) = self.contents {
            return Some(c);
        }
        None
    }

    async fn expect_mocks_path_to_exist(&self) -> Result<(), Box<dyn StdError + Sync + Send>> {
        let Some(ref path) = self.get_absolute_mocks_path().await else {
            return Err("Mocks dir not set".into());
        };
        let path_metadata = tokio::fs::metadata(path).await;

        let is_dir = path_metadata
            .as_ref()
            .map(Metadata::is_dir)
            .unwrap_or(false);

        if is_dir {
            return Ok(());
        }

        let wrong_target = path_metadata
            .map(|m| m.is_file() || m.is_symlink())
            .unwrap_or(false);

        if wrong_target {
            let err_msg = format!(
                "The specified path '{}' is not a directory",
                &path.display()
            );
            return Err(err_msg.into());
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

    pub async fn test(&self) -> Result<(), Box<dyn StdError + Sync + Send>> {
        if let Err(e) = self.expect_mocks_path_to_exist().await {
            return Err(e);
        }

        Ok(())
    }

    pub async fn create_mock(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let Some(mocks_absolute_path) = self.get_absolute_mocks_path().await else {
            return Err("Mocks dir not set".into());
        };
        let full_mock_path = mocks_absolute_path.extend_with_url_path(self.route());
        let destination_directory = full_mock_path.with_last_removed();
        let last_path_part = full_mock_path.file_name().map(|s| s.to_str()).flatten();
        let method_lower = &self.method().to_lowercase();
        let Some(last_path_part) = last_path_part else {
            let dst_dir = destination_directory.display();
            let mock = format!("{}.{}", dst_dir, &method_lower);
            let err_message = format!("Can't create mock \"{}\"", mock);
            let error = IoError::new(ErrorKind::NotFound, err_message.as_str());
            return Err(error.into());
        };

        tokio::fs::create_dir_all(&destination_directory).await?;

        let file_name = format!("{}.{}", last_path_part, &method_lower);
        let full_destination_file_path = destination_directory.join(&file_name);
        let full_destination_config_path = destination_directory.join(CONFIG_FILE_NAME);
        let contents = self.contents().map(String::from);

        let mock_config: MockConfig = self.into();
        let write_contents_fut = Self::write_default_mock(&full_destination_file_path, contents);
        let dump_config_fut =
            mock_config.try_write_to_file(&full_destination_config_path, &file_name);

        try_join!(write_contents_fut, dump_config_fut)?;

        Ok(())
    }

    async fn write_default_mock(
        absolute_path: impl AsRef<Path>,
        contents: Option<String>,
    ) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let contents = contents.unwrap_or_else(|| json!({ "hello": "world" }).to_string());
        tokio::fs::write(&absolute_path, &contents)
            .await
            .map_err(Box::from)
    }
}

impl From<&CreateParams> for MockConfig {
    fn from(value: &CreateParams) -> Self {
        let owned_headers = value.headers().fold(HashMap::new(), |mut map, (key, val)| {
            map.insert(key.to_string(), val.to_string());
            map
        });
        MockConfig::new()
            .with_headers(owned_headers)
            .with_delay_ms(value.delay_ms.unwrap_or(DEFAULT_DELAY_MS))
            .with_status_code(value.status_code())
            .with_cache_mode(value.cache_mode().unwrap_or(CacheMode::NoCache))
            .with_disabled_status(value.is_disabled())
    }
}

impl WithMocks for CreateParams {
    fn get_mocks(&self) -> Option<&str> {
        self.mocks.as_ref().map(|x| x.as_str())
    }
}

impl WithGlobalConfigAPI for CreateParams {
    async fn get_global_config(&self) -> &GlobalConfigAPI {
        self.global_config.get_or_init(GlobalConfigAPI::new).await
    }
}
