use std::path::PathBuf;

pub(crate) trait PathBufExt {
    fn with_last_removed(&self) -> Self;
}

impl PathBufExt for PathBuf {
    /// Removes last part of PathBuf
    /// Does nothing if It cannot do that: returns cloned self
    fn with_last_removed(&self) -> Self {
        let mut clone = self.clone();
        clone.pop();
        clone
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_last_dir_success() {
        let path = PathBuf::from("/foo/bar/baz");
        let actual = path.with_last_removed();
        let expected = PathBuf::from("/foo/bar");
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_last_file_success() {
        let path = PathBuf::from("/foo/bar/baz.txt");
        let actual = path.with_last_removed();
        let expected = PathBuf::from("/foo/bar");
        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_last_part_impossible() {
        let path = PathBuf::from("/");
        let actual = path.with_last_removed();
        let expected = PathBuf::from("/");
        assert_eq!(actual, expected);
    }
}
