use path_absolutize::Absolutize;
use std::error::Error as StdError;
use std::path::{Path, PathBuf};

pub(crate) trait AbsoluteMocksPath {
    fn get_absolute_mocks_path(&self) -> Result<PathBuf, Box<dyn StdError + Sync + Send>>;
}

/// The function is necessary in order to get the full path of the mock directory.
/// Since the mock directory can be specified as either an absolute or relative path,
/// the function normalizes the path to an absolute path.
/// If a relative path is specified, the current directory and the relative path are glued together.
/// If an absolute path is specified, the function simply returns it.
pub(crate) fn generic_get_absolute_mocks_path<MP, CWD>(
    mocks_path: MP,
    get_current_dir: CWD,
) -> Result<PathBuf, Box<dyn StdError + Sync + Send>>
where
    MP: AsRef<Path>,
    CWD: FnOnce() -> Result<PathBuf, Box<dyn StdError + Sync + Send>>,
{
    let mocks_path = mocks_path.as_ref();
    let path = Path::new(&mocks_path).absolutize()?;
    match PathBuf::from(&mocks_path).is_absolute() {
        true => Ok(path.into()),
        false => Ok(get_current_dir()?.join(&mocks_path).absolutize()?.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_home_dir() -> Result<PathBuf, Box<dyn StdError + Sync + Send>> {
        Ok("/home/john_doe".into())
    }

    #[test]
    fn unix_absolute() {
        let actual = generic_get_absolute_mocks_path("/foo/bar/baz", get_home_dir).unwrap();
        let expected = PathBuf::from("/foo/bar/baz");
        assert_eq!(actual, expected);
    }

    #[test]
    fn unix_relative() {
        let actual = generic_get_absolute_mocks_path("foo/bar/baz", get_home_dir).unwrap();
        let expected = PathBuf::from("/home/john_doe/foo/bar/baz");
        assert_eq!(actual, expected);
    }

    #[test]
    fn relative_path_with_parent_dir_is_normalized() {
        let actual = generic_get_absolute_mocks_path("foo/bar/../baz", get_home_dir).unwrap();
        let expected = PathBuf::from("/home/john_doe/foo/baz");

        assert_eq!(actual, expected);
    }
}
