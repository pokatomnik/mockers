use crate::entities::global_config::WithGlobalConfigAPI;
use crate::entities::mock_defaults::DEFAULT_MOCKS_DIR_NAME;
use path_absolutize::Absolutize;
use std::env::{current_dir, home_dir};
use std::fs::Metadata;
use std::path::{Path, PathBuf};

pub(crate) trait WithMocks {
    fn get_mocks(&self) -> Option<&str>;
}

pub(crate) trait AbsoluteMocksPath {
    async fn get_absolute_mocks_path(&self) -> Option<PathBuf>;

    async fn expect_mocks_path_to_exist(&self) -> anyhow::Result<()>;
}

impl<T> AbsoluteMocksPath for T
where
    T: WithGlobalConfigAPI + WithMocks,
{
    async fn get_absolute_mocks_path(&self) -> Option<PathBuf> {
        let mocks_from_args = self.get_mocks().and_then(|mocks| {
            generic_get_absolute_mocks_path(mocks, || Ok(current_dir()?), home_dir)
        });
        if let Some(mocks_from_args) = mocks_from_args {
            return Some(mocks_from_args);
        }
        let mocks_from_global_config =
            self.get_global_config()
                .await
                .get_mocks()
                .await
                .and_then(|mocks| {
                    generic_get_absolute_mocks_path(mocks, || Ok(current_dir()?), home_dir)
                });
        if let Some(mocks_from_global_config) = mocks_from_global_config {
            return Some(mocks_from_global_config);
        }

        current_dir()
            .map(|cwd| cwd.join(DEFAULT_MOCKS_DIR_NAME))
            .ok()
    }

    async fn expect_mocks_path_to_exist(&self) -> anyhow::Result<()> {
        let Some(ref path) = self.get_absolute_mocks_path().await else {
            return Err(anyhow::Error::msg("Mocks dir not set"));
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
            return Err(anyhow::Error::msg(err_msg));
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
}

/// The function is necessary in order to get the full path of the mock directory.
/// Since the mock directory can be specified as either an absolute or relative path,
/// the function normalizes the path to an absolute path.
/// If a relative path is specified, the current directory and the relative path are glued together.
/// If an absolute path is specified, the function simply returns it.
pub(crate) fn generic_get_absolute_mocks_path(
    mocks_path: impl AsRef<Path>,
    get_current_dir: impl FnOnce() -> anyhow::Result<PathBuf>,
    get_home_dir: impl FnOnce() -> Option<PathBuf>,
) -> Option<PathBuf> {
    let mocks_path = mocks_path.as_ref();
    if mocks_path.starts_with("~") {
        let mocks_path_as_str = mocks_path.to_string_lossy().to_string();
        let mocks_path = mocks_path_as_str
            .trim_start_matches("~/")
            .trim_start_matches("~\\");
        let Some(home_dir) = get_home_dir() else {
            return None;
        };
        return home_dir
            .join(mocks_path)
            .absolutize()
            .ok()
            .map(PathBuf::from);
    }
    let path = Path::new(&mocks_path).absolutize().ok()?;
    if PathBuf::from(&mocks_path).is_absolute() {
        return Some(path.into());
    }

    get_current_dir()
        .ok()
        .and_then(|cwd| {
            cwd.join(mocks_path)
                .absolutize()
                .ok()
                .map(|v| v.to_path_buf())
        })
        .map(|cwd| cwd.into())
}
