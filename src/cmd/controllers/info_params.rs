use crate::entities::cache_mode::CacheMode;
use crate::entities::global_config::{GlobalConfigAPI, WithGlobalConfigAPI};
use crate::entities::mock_config::MockConfig;
use crate::entities::mock_defaults::{DEFAULT_DELAY_MS, DEFAULT_STATUS_CODE};
use crate::libs::fs_walker::FSWalker;
use crate::libs::get_mime::get_mime;
use crate::libs::http_method::StandardMethodValidator;
use crate::libs::path_buf_ext::PathBufExt;
use crate::server::params::CONFIG_FILE_NAME;
use crate::use_cases::absolute_mocks_path::{AbsoluteMocksPath, WithMocks};
use clap::Args;
use hyper::Method;
use std::fs::Metadata;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::str::FromStr;
use tokio::join;
use tokio::sync::OnceCell;

#[derive(Args, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct InfoParams {
    #[arg(long, short, help = "Path to the directory containing mock files")]
    mocks: Option<String>,

    #[arg(
        long,
        short,
        default_value_t = false,
        help = "Display mock body or not"
    )]
    show_body: bool,

    /// Name or full path to mock file or both
    name: String,

    #[clap(skip)]
    global_config: OnceCell<GlobalConfigAPI>,
}

impl InfoParams {
    async fn find_matching_mocks(
        &self,
        filter_fn: impl Fn((&Metadata, &PathBuf)) -> bool,
    ) -> anyhow::Result<Vec<(Metadata, PathBuf)>> {
        let Some(absolute_mocks_path) = self.get_absolute_mocks_path().await else {
            return Err(anyhow::Error::msg("No mocks path"));
        };
        let mocks = FSWalker::new(&absolute_mocks_path)
            .into_iter()
            .await
            .filter(|(md, pathbuf)| md.is_file() && filter_fn((&md, &pathbuf)))
            .collect::<Vec<_>>();
        Ok(mocks)
    }

    fn show_if_empty(&self) {
        println!("No mocks found");
    }

    fn show_if_multiple(
        &self,
        absolute_mocks_path: impl AsRef<Path>,
        matching_mocks: &Vec<(Metadata, PathBuf)>,
    ) {
        let mut correct_mocks = Vec::with_capacity(matching_mocks.iter().len());
        for (_, pathbuf) in matching_mocks {
            let method = pathbuf
                .extension()
                .and_then(|m| Method::from_str(&m.to_string_lossy().to_uppercase()).ok());
            let Some(method) = method else {
                continue;
            };

            let method_validation_result = method
                .validate(|| Box::new("Incorrect method name"))
                .is_ok();
            if !method_validation_result {
                continue;
            }

            let sep = MAIN_SEPARATOR.to_string();
            let mock_path = pathbuf.to_string_lossy().replace(
                &absolute_mocks_path.as_ref().to_string_lossy().to_string(),
                "",
            );
            let mock_path = mock_path.trim_start_matches(&sep).to_string();

            correct_mocks.push(format!("{}{}", &sep, &mock_path));
        }
        println!(
            "There are {}, mocks matching your query:",
            correct_mocks.len()
        );
        for (idx, mock) in correct_mocks.iter().enumerate() {
            println!("{}. {}", idx + 1, mock);
        }
        println!("Specify which are you interested in more precisely");
    }

    fn show_if_error(&self) {
        eprintln!("Failed to show mocks info");
    }

    async fn print_config_and_body(
        &self,
        full_mock_body_path: impl AsRef<Path>,
        config: &MockConfig,
        body: &[u8],
    ) -> anyhow::Result<()> {
        let absolute_mocks_path = self.get_absolute_mocks_path().await;
        let Some(absolute_mocks_path) = absolute_mocks_path else {
            return Err(anyhow::Error::msg("No mocks path"));
        };
        let full_mock_body_path = full_mock_body_path
            .as_ref()
            .display()
            .to_string()
            .replace(&absolute_mocks_path.to_string_lossy().to_string(), "");
        println!(
            "Mock: {}{}",
            &MAIN_SEPARATOR,
            full_mock_body_path.trim_start_matches(&MAIN_SEPARATOR.to_string())
        );
        println!(
            "Response status code: {}",
            config.status_code().unwrap_or(DEFAULT_STATUS_CODE)
        );
        let headers = config.headers();
        if let Some(headers) = headers
            && !headers.is_empty()
        {
            println!("Response headers:");
            for (key, val) in headers {
                println!("{}: {}", key, val);
            }
        }
        println!(
            "Delay timeout (ms): {}",
            config.delay_ms().unwrap_or(DEFAULT_DELAY_MS)
        );
        println!(
            "Cache mode: {}",
            config.cache_mode().unwrap_or(CacheMode::NoCache)
        );
        let detected_mime = get_mime(&body).await;
        println!("Detected mime: {}", detected_mime);

        if self.show_body {
            println!("Body:");
            println!("{}", String::from_utf8_lossy(&body));
        }

        Ok(())
    }

    async fn show_mock_info(&self, full_mock_body_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let entry_name = full_mock_body_path
            .as_ref()
            .file_name()
            .map(|s| s.to_string_lossy().to_string());

        let Some(entry_name) = entry_name else {
            return Err(anyhow::Error::msg("Unknown file name"));
        };

        let full_dir_path = full_mock_body_path.as_ref().to_owned().with_last_removed();
        let full_config_path = full_dir_path.clone().join(CONFIG_FILE_NAME);

        let read_body_fut = async { tokio::fs::read(&full_mock_body_path).await };

        let read_config_fut = async {
            MockConfig::try_read_from_file(&full_config_path)
                .await
                .ok()
                .and_then(|m| m.get(&entry_name).cloned())
        };

        let (config, body) = join!(read_config_fut, read_body_fut);
        let body = body?;

        self.print_config_and_body(
            full_mock_body_path,
            &config.unwrap_or_else(MockConfig::default),
            body.as_ref(),
        )
        .await
    }

    pub async fn show_info(&self) -> anyhow::Result<()> {
        let name_lower = self.name.to_lowercase();
        let mocks = self
            .find_matching_mocks(|(_, pathbuf)| {
                let found = pathbuf
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&name_lower);
                let is_ext_correct_method = pathbuf
                    .extension()
                    .map(|e| e.to_string_lossy())
                    .map(|e| e.to_uppercase())
                    .map(|e| e.to_string())
                    .and_then(|e| e.parse::<Method>().ok())
                    .and_then(|m| m.validate(|| Box::new("")).ok())
                    .is_some();
                found && is_ext_correct_method
            })
            .await;

        let Ok(matching_mocks) = mocks else {
            return Ok(self.show_if_error());
        };

        let Some((_, first_matching_mock)) = matching_mocks.first() else {
            return Ok(self.show_if_empty());
        };

        let Some(absolute_mocks_path) = self.get_absolute_mocks_path().await else {
            return Ok(());
        };

        if matching_mocks.len() > 1 {
            return Ok(self.show_if_multiple(&absolute_mocks_path, &matching_mocks));
        }

        self.show_mock_info(first_matching_mock).await
    }
}

impl WithMocks for InfoParams {
    fn get_mocks(&self) -> Option<&str> {
        self.mocks.as_ref().map(|x| x.as_str())
    }
}

impl WithGlobalConfigAPI for InfoParams {
    async fn get_global_config(&self) -> &GlobalConfigAPI {
        self.global_config.get_or_init(GlobalConfigAPI::new).await
    }
}
