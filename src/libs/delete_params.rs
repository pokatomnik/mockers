use crate::libs::absolute_mocks_path::generic_get_absolute_mocks_path;
use crate::libs::fs_walker::FSWalker;
use crate::libs::http_method::StandardMethodValidator;
use crate::libs::mock_config::MockConfig;
use crate::libs::path_buf_ext::PathBufExt;
use crate::server::params::{CONFIG_FILE_NAME, DEFAULT_MOCKS_DIR_NAME};
use clap::Args;
use hyper::Method;
use std::error::Error as StdError;
use std::fs::Metadata;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::pin::Pin;
use std::str::FromStr;
use tokio::try_join;

#[derive(Args, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct DeleteParams {
    #[arg(long, short, default_value = DEFAULT_MOCKS_DIR_NAME, help = "Path to the directory containing mock files")]
    mocks: String,

    /// Name or full path to mock file or both
    name: String,
}

impl DeleteParams {
    fn get_absolute_mocks_path(&self) -> Result<PathBuf, Box<dyn StdError + Sync + Send>> {
        generic_get_absolute_mocks_path(&self.mocks, || std::env::current_dir().map_err(Box::from))
    }

    async fn find_matching_mocks(
        &self,
        filter_fn: impl Fn((&Metadata, &PathBuf)) -> bool,
    ) -> Result<Vec<(Metadata, PathBuf)>, Box<dyn StdError + Sync + Send>> {
        let absolute_mocks_path = self.get_absolute_mocks_path()?;
        let mocks = FSWalker::new(&absolute_mocks_path)
            .into_iter()
            .await
            .filter(|(md, pathbuf)| md.is_file() && filter_fn((&md, &pathbuf)))
            .collect::<Vec<_>>();
        Ok(mocks)
    }

    fn show_if_empty(&self) {
        println!("There are no mocks matching your query");
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
        println!("Specify which one are you intended to delete more precisely");
    }

    fn show_if_error(&self) {
        eprintln!("Failed to delete mock");
    }

    async fn delete_mock_by_path(
        &self,
        absolute_path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn StdError>> {
        let absolute_mock_path = absolute_path.as_ref();
        let entry_name = (&absolute_mock_path)
            .file_name()
            .map(|en| en.to_string_lossy().to_string());

        let remove_mock_fut: Pin<Box<dyn Future<Output = Result<(), Box<dyn StdError>>>>> =
            Box::pin(async {
                tokio::fs::remove_file(&absolute_mock_path)
                    .await
                    .map_err(Box::from)
            });

        let remove_config_fut: Pin<Box<dyn Future<Output = Result<(), Box<dyn StdError>>>>> =
            Box::pin(async {
                let Some(entry_name) = entry_name else {
                    return Ok(());
                };
                let absolute_config_path = &absolute_mock_path
                    .to_owned()
                    .with_last_removed()
                    .join(CONFIG_FILE_NAME);
                let mut configs = MockConfig::try_read_from_file(&absolute_config_path)
                    .await
                    .map_err(|x| -> Box<dyn StdError> { x })?;
                configs.remove(&entry_name);
                if configs.is_empty() {
                    tokio::fs::remove_file(absolute_config_path).await?;
                } else {
                    let json = serde_json::to_string_pretty(&configs)?;
                    tokio::fs::write(&absolute_config_path, json).await?;
                }

                Ok(())
            });

        try_join!(remove_config_fut, remove_mock_fut).map(|_| ())
    }

    pub async fn delete_mock(&self) -> Result<(), Box<dyn StdError + Sync + Send>> {
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

        let Ok(absolute_mocks_path) = self.get_absolute_mocks_path() else {
            return Ok(());
        };

        if matching_mocks.len() > 1 {
            return Ok(self.show_if_multiple(&absolute_mocks_path, &matching_mocks));
        }

        self.delete_mock_by_path(first_matching_mock)
            .await
            .map_err(|e| -> Box<dyn StdError + Send + Sync> { e.to_string().into() })
    }
}
