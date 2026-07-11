use crate::entities::global_config::{GlobalConfigAPI, WithGlobalConfigAPI};
use crate::entities::mock_config::MockConfig;
use crate::libs::fs_walker::FSWalker;
use crate::libs::http_method::StandardMethodValidator;
use crate::libs::path_buf_ext::PathBufExt;
use crate::server::params::CONFIG_FILE_NAME;
use crate::use_cases::absolute_mocks_path::{AbsoluteMocksPath, WithMocks};
use clap::Args;
use hyper::Method;
use std::fs::Metadata;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::pin::Pin;
use std::str::FromStr;
use tokio::sync::OnceCell;
use tokio::try_join;

#[derive(Args, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct DeleteParams {
    #[arg(long, short, help = "Path to the directory containing mock files")]
    mocks: Option<String>,

    /// Name or full path to mock file or both
    name: String,

    #[clap(skip)]
    global_config: OnceCell<GlobalConfigAPI>,
}

impl DeleteParams {
    async fn find_matching_mocks(
        &self,
        filter_fn: impl Fn((&Metadata, &PathBuf)) -> bool,
    ) -> anyhow::Result<Vec<(Metadata, PathBuf)>> {
        let Some(absolute_mocks_path) = self.get_absolute_mocks_path().await else {
            return Err(anyhow::Error::msg("Mocks dir not set"));
        };
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

    async fn delete_mock_by_path(&self, absolute_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let absolute_mock_path = absolute_path.as_ref();
        let entry_name = (&absolute_mock_path)
            .file_name()
            .map(|en| en.to_string_lossy().to_string());

        let remove_mock_fut: Pin<Box<dyn Future<Output = anyhow::Result<()>>>> = Box::pin(async {
            tokio::fs::remove_file(&absolute_mock_path).await?;
            Ok(())
        });

        let remove_config_fut: Pin<Box<dyn Future<Output = anyhow::Result<()>>>> =
            Box::pin(async {
                let Some(entry_name) = entry_name else {
                    return Ok(());
                };
                let absolute_config_path = &absolute_mock_path
                    .to_owned()
                    .with_last_removed()
                    .join(CONFIG_FILE_NAME);
                MockConfig::try_remove_from_file(absolute_config_path, entry_name).await?;

                Ok(())
            });

        try_join!(remove_config_fut, remove_mock_fut).map(|_| ())
    }

    pub async fn delete_mock(&self) -> anyhow::Result<()> {
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

        self.delete_mock_by_path(first_matching_mock).await?;
        Ok(())
    }
}

impl WithMocks for DeleteParams {
    fn get_mocks(&self) -> Option<&str> {
        self.mocks.as_ref().map(|x| x.as_str())
    }
}

impl WithGlobalConfigAPI for DeleteParams {
    async fn get_global_config(&self) -> &GlobalConfigAPI {
        self.global_config.get_or_init(GlobalConfigAPI::new).await
    }
}
