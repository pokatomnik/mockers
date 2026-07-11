use std::collections::HashMap;
use std::path::Path;

use crate::entities::cache_mode::CacheMode;
use crate::entities::global_config::{GlobalConfigAPI, WithGlobalConfigAPI};
use crate::entities::mock_config::MockConfig;
use crate::entities::mock_defaults::{DEFAULT_DELAY_MS, DEFAULT_STATUS_CODE};
use crate::libs::http_method::HyperHTTPMethodExt;
use crate::libs::path_buf_ext::PathBufExt;
use crate::libs::status_code_ext::StatusCode;
use crate::server::params::CONFIG_FILE_NAME;
use crate::server::params::DEFAULT_PORT;
use crate::use_cases::absolute_mocks_path::{AbsoluteMocksPath, WithMocks};
use clap::Args;
use serde_json::json;
use tokio::sync::OnceCell;
use tokio::try_join;

pub(crate) const DEFAULT_METHOD: &'static str = "GET";

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

    #[arg(long, short, default_value_t = false, help = "Run in interactive mode")]
    interactive: bool,

    /// Pathname to create mock for
    route: Option<String>,

    #[clap(skip)]
    global_config: OnceCell<GlobalConfigAPI>,
}

impl CreateParams {
    fn method(&self) -> String {
        let all_http_methods = hyper::Method::get_all_methods()
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();
        let method = self.method.to_owned();
        match self.interactive {
            true => dialoguer::FuzzySelect::new()
                .with_prompt("Select HTTP method")
                .items(&all_http_methods)
                .default(0)
                .interact()
                .ok()
                .and_then(|idx| all_http_methods.get(idx))
                .cloned()
                .unwrap_or(DEFAULT_METHOD.to_string()),
            false => method,
        }
    }

    fn route(&self) -> String {
        let dialog = dialoguer::Input::new().with_prompt("Specify URL path");
        match (self.interactive, &self.route) {
            (true, Some(route)) => dialog
                .with_initial_text(route)
                .interact()
                .unwrap_or_default(),
            (false, Some(route)) => route.clone(),
            (true, None) | (false, None) => dialog.interact().unwrap_or_default(),
        }
    }

    fn status_code(&self) -> u16 {
        let all_codes = u16::get_all_status_codes();
        let codes: Vec<String> = all_codes.iter().map(StatusCode::pretty_print).collect();
        match &self.interactive {
            true => dialoguer::FuzzySelect::new()
                .with_prompt("Specify http Response code")
                .items(&codes)
                .default(0)
                .interact()
                .ok()
                .and_then(|idx| all_codes.get(idx).cloned())
                .unwrap_or(DEFAULT_PORT),
            false => self.status_code,
        }
    }

    fn delay_ms(&self) -> Option<u64> {
        match &self.interactive {
            false => self.delay_ms,
            true => {
                let res = dialoguer::Input::new()
                    .with_prompt("Specify delay in milliseconds before response")
                    .validate_with(|v: &String| {
                        v.parse::<u64>()
                            .map(|_| ())
                            .map_err(|_| "Invalid delay time")
                    })
                    .interact()
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_DELAY_MS);
                Some(res)
            }
        }
    }

    fn cache_mode(&self) -> Option<CacheMode> {
        let all_cache_modes = CacheMode::all_values();
        let prompt = format!(
            "Select cache mode. Prefer \"{}\" to cache non-existing responses and \"{}\" not to cache them at all.",
            CacheMode::Overwrite,
            CacheMode::NoCache
        );
        match &self.interactive {
            true => {
                let res = dialoguer::Select::new()
                    .with_prompt(prompt)
                    .items(&all_cache_modes)
                    .default(0)
                    .interact()
                    .ok()
                    .and_then(|idx| all_cache_modes.get(idx))
                    .cloned()
                    .unwrap_or(CacheMode::NoCache);
                return Some(res);
            }
            false => self.cache_mode.clone(),
        }
    }

    fn headers(&self) -> Vec<(String, String)> {
        if !&self.interactive {
            return self
                .headers
                .iter()
                .filter_map(|v| {
                    v.split_once(':')
                        .map(|(k, v)| (k.trim(), v.trim()))
                        .filter(|(k, v)| !k.is_empty() && !v.is_empty())
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                })
                .collect();
        }

        let mut acc = Vec::new();

        while dialoguer::Confirm::new()
            .with_prompt(match acc.is_empty() {
                true => "Add headers?",
                false => "Add more headers?",
            })
            .interact()
            .unwrap_or(false)
        {
            let header_key = dialoguer::Input::new()
                .with_prompt("Specify header key")
                .interact()
                .ok()
                .unwrap_or("".to_string())
                .trim()
                .to_string();
            let header_value = dialoguer::Input::new()
                .with_prompt("Specify header value")
                .interact()
                .ok()
                .unwrap_or("".to_string())
                .trim()
                .to_string();
            acc.push((header_key, header_value));
        }

        return acc;
    }

    fn is_disabled(&self) -> bool {
        match self.interactive {
            true => dialoguer::Confirm::new()
                .with_prompt("Create this mock disabled?")
                .default(false)
                .show_default(true)
                .interact()
                .unwrap_or(false),
            false => self.disabled,
        }
    }

    fn contents(&self) -> Option<String> {
        let initial_text = self.contents.as_ref().cloned().unwrap_or_default();
        match &self.interactive {
            false => self.contents.to_owned(),
            true => {
                let res = dialoguer::Input::new()
                    .with_prompt("Write mock contents")
                    .allow_empty(false)
                    .with_initial_text(initial_text)
                    .interact()
                    .ok()
                    .unwrap_or_default();
                return Some(res);
            }
        }
    }

    pub async fn test(&self) -> anyhow::Result<()> {
        if let Err(e) = self.expect_mocks_path_to_exist().await {
            return Err(e.into());
        }

        Ok(())
    }

    pub async fn create_mock(&self) -> anyhow::Result<()> {
        let Some(mocks_absolute_path) = self.get_absolute_mocks_path().await else {
            return Err(anyhow::Error::msg("Mocks dir not set"));
        };
        let full_mock_path = mocks_absolute_path.extend_with_url_path(self.route());
        let destination_directory = full_mock_path.with_last_removed();
        let last_path_part = full_mock_path.file_name().map(|s| s.to_str()).flatten();
        let method_lower = &self.method().to_lowercase();
        let Some(last_path_part) = last_path_part else {
            let dst_dir = destination_directory.display();
            let mock = format!("{}.{}", dst_dir, &method_lower);
            let err_message = format!("Can't create mock \"{}\"", mock);
            return Err(anyhow::Error::msg(err_message));
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
    ) -> anyhow::Result<()> {
        let contents = contents.unwrap_or_else(|| json!({ "hello": "world" }).to_string());
        tokio::fs::write(&absolute_path, &contents).await?;
        Ok(())
    }
}

impl From<&CreateParams> for MockConfig {
    fn from(value: &CreateParams) -> Self {
        let owned_headers =
            value
                .headers()
                .into_iter()
                .fold(HashMap::new(), |mut map, (key, val)| {
                    map.insert(key, val);
                    map
                });
        MockConfig::new()
            .with_headers(owned_headers)
            .with_delay_ms(value.delay_ms().unwrap_or(DEFAULT_DELAY_MS))
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
