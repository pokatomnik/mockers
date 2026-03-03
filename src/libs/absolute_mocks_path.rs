use crate::libs::global_config::WithGlobalConfigAPI;
use crate::server::params::DEFAULT_MOCKS_DIR_NAME;
use path_absolutize::Absolutize;
use std::env::{current_dir, home_dir};
use std::error::Error as StdError;
use std::path::{Path, PathBuf};

pub(crate) trait WithMocks {
    fn get_mocks(&self) -> Option<&str>;
}

pub(crate) trait AbsoluteMocksPath {
    async fn get_absolute_mocks_path(&self) -> Option<PathBuf>;
}

impl<T> AbsoluteMocksPath for T
where
    T: WithGlobalConfigAPI + WithMocks,
{
    async fn get_absolute_mocks_path(&self) -> Option<PathBuf> {
        let mocks_from_args = self.get_mocks().and_then(|mocks| {
            generic_get_absolute_mocks_path(mocks, || current_dir().map_err(Box::from), home_dir)
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
                    generic_get_absolute_mocks_path(
                        mocks,
                        || current_dir().map_err(Box::from),
                        home_dir,
                    )
                });
        if let Some(mocks_from_global_config) = mocks_from_global_config {
            return Some(mocks_from_global_config);
        }

        current_dir()
            .map(|cwd| cwd.join(DEFAULT_MOCKS_DIR_NAME))
            .ok()
    }
}

/// The function is necessary in order to get the full path of the mock directory.
/// Since the mock directory can be specified as either an absolute or relative path,
/// the function normalizes the path to an absolute path.
/// If a relative path is specified, the current directory and the relative path are glued together.
/// If an absolute path is specified, the function simply returns it.
pub(crate) fn generic_get_absolute_mocks_path(
    mocks_path: impl AsRef<Path>,
    get_current_dir: impl FnOnce() -> Result<PathBuf, Box<dyn StdError + Sync + Send>>,
    get_home_dir: impl FnOnce() -> Option<PathBuf>,
) -> Option<PathBuf> {
    let mocks_path = mocks_path.as_ref();
    if mocks_path.starts_with("~") {
        let mocks_path_as_str = mocks_path.to_string_lossy().to_string();
        let mocks_path = mocks_path_as_str.trim_start_matches("~/").trim_start_matches("~\\");
        let Some(home_dir) = get_home_dir() else {
            return None;
        };
        return home_dir.join(mocks_path).absolutize().ok().map(PathBuf::from);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_current_dir() -> Result<PathBuf, Box<dyn StdError + Sync + Send>> {
        Ok("/home/john_doe".into())
    }

    fn get_home_dir() -> Option<PathBuf> {
        Some("/home/john_doe".into())
    }

    #[test]
    fn unix_absolute() {
        let actual =
            generic_get_absolute_mocks_path("/foo/bar/baz", get_current_dir, get_home_dir).unwrap();
        let expected = PathBuf::from("/foo/bar/baz");
        assert_eq!(actual, expected);
    }

    #[test]
    fn unix_relative() {
        let actual =
            generic_get_absolute_mocks_path("foo/bar/baz", get_current_dir, get_home_dir).unwrap();
        let expected = PathBuf::from("/home/john_doe/foo/bar/baz");
        assert_eq!(actual, expected);
    }

    #[test]
    fn relative_path_with_parent_dir_is_normalized() {
        let actual =
            generic_get_absolute_mocks_path("foo/bar/../baz", get_current_dir, get_home_dir)
                .unwrap();
        let expected = PathBuf::from("/home/john_doe/foo/baz");

        assert_eq!(actual, expected);
    }

    #[test]
    fn with_home_alias() {
        let actual =
            generic_get_absolute_mocks_path("~/foo/bar/..", get_current_dir, get_home_dir).unwrap();
        let expected = PathBuf::from("/home/john_doe/foo");

        assert_eq!(actual, expected);
    }
}
