use std::error::Error as StdError;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Args;
use hyper::Method;

use crate::libs::absolute_mocks_path::{generic_get_absolute_mocks_path, AbsoluteMocksPath};
use crate::libs::{fs_walker::FSWalker, http_method::StandardMethodValidator};
use crate::server::params::DEFAULT_MOCKS_DIR_NAME;

#[derive(Args, Debug, Clone)]
pub(crate) struct LsParams {
    #[arg(long, short, default_value = DEFAULT_MOCKS_DIR_NAME, help = "Path to the directory containing mock files")]
    mocks: String,
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
        let absolute_mocks_path = self.get_absolute_mocks_path()?;
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

impl AbsoluteMocksPath for LsParams {
    fn get_absolute_mocks_path(&self) -> Result<PathBuf, Box<dyn StdError + Sync + Send>> {
        generic_get_absolute_mocks_path(&self.mocks, || std::env::current_dir().map_err(Box::from))
    }
}
