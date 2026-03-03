use std::str::FromStr;

use crate::libs::absolute_mocks_path::{AbsoluteMocksPath, WithMocks};
use crate::libs::global_config::{GlobalConfigAPI, WithGlobalConfigAPI};
use crate::libs::{fs_walker::FSWalker, http_method::StandardMethodValidator};
use clap::Args;
use hyper::Method;
use tokio::sync::OnceCell;

#[derive(Args, Debug, Clone)]
pub(crate) struct LsParams {
    #[arg(long, short, help = "Path to the directory containing mock files")]
    mocks: Option<String>,

    #[clap(skip)]
    global_config: OnceCell<GlobalConfigAPI>,
}

impl LsParams {
    fn get_url_and_method(buf: &str) -> Option<(String, Method)> {
        buf.rsplit_once(".")
            .map(|(pathname, method)| match Method::from_str(method) {
                Err(_) => None,
                Ok(method) => Some((pathname.to_string(), method)),
            })
            .unwrap_or(None)
    }

    pub async fn ls_mocks(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let Some(absolute_mocks_path) = self.get_absolute_mocks_path().await else {
            return Err("No mocks path".into());
        };
        let mocks: Vec<(String, Method)> = FSWalker::new(&absolute_mocks_path)
            .into_iter()
            .await
            .filter(|(md, _)| md.is_file())
            .filter(|(_, pathbuf)| {
                pathbuf
                    .extension()
                    .map(|e| e.to_string_lossy().to_uppercase())
                    .and_then(|ext| Method::from_str(&ext).ok())
                    .and_then(|m| m.validate(|| Box::new("IncorrectMethod".to_string())).ok())
                    .is_some()
            })
            .filter_map(|(_, pb)| {
                let mock_url = pb
                    .to_string_lossy()
                    .replace(&absolute_mocks_path.to_string_lossy().to_string(), "");
                Self::get_url_and_method(&mock_url)
            })
            .collect();

        if mocks.len() == 0 {
            println!("There are no mocks");
            return Ok(());
        }

        println!("There are some mocks in {}", &absolute_mocks_path.display());
        for (pathname, method) in mocks {
            println!("* {} {}", method.to_string().to_uppercase(), pathname);
        }

        Ok(())
    }
}

impl WithMocks for LsParams {
    fn get_mocks(&self) -> Option<&str> {
        self.mocks.as_ref().map(|x| x.as_str())
    }
}

impl WithGlobalConfigAPI for LsParams {
    async fn get_global_config(&self) -> &GlobalConfigAPI {
        self.global_config.get_or_init(GlobalConfigAPI::new).await
    }
}
