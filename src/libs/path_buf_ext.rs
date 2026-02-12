use std::path::PathBuf;

pub(crate) trait PathBufExt {
    fn with_last_removed(&self) -> Self;

    fn extend_with_url_path(&self, parts: impl Into<String>) -> PathBuf;
}

impl PathBufExt for PathBuf {
    /// Removes last part of PathBuf
    /// Does nothing if It cannot do that: returns cloned self
    fn with_last_removed(&self) -> Self {
        let mut clone = self.clone();
        clone.pop();
        clone
    }

    /// Concatenates `PathBuf` with Path-like parts:
    /// - `/foo/bar/baz`
    /// - `foo/bar`
    /// The result will be always fresh `PathBuf`
    /// with desired parts at the end
    fn extend_with_url_path(&self, parts: impl Into<String>) -> PathBuf {
        let parts = parts.into();
        if parts.starts_with("/") {
            self.join(parts.trim_start_matches("/"))
        } else {
            self.join(parts)
        }
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

    #[test]
    fn extend_with_relative() {
        let path = PathBuf::from("/foo/bar");
        let actual = path.extend_with_url_path("baz");
        let expected = PathBuf::from("/foo/bar/baz");
        assert_eq!(actual, expected);
    }

    #[test]
    fn extend_with_absolute() {
        let path = PathBuf::from("/foo/bar");
        let actual = path.extend_with_url_path("/baz");
        let expected = PathBuf::from("/foo/bar/baz");
        assert_eq!(actual, expected);
    }

    #[test]
    fn extend_with_empty_url_path() {
        let path = PathBuf::from("/foo/bar");
        let actual = path.extend_with_url_path("");
        let expected = PathBuf::from("/foo/bar");
        assert_eq!(actual, expected);
    }

    #[test]
    fn extend_with_trailing_slash() {
        let path = PathBuf::from("/foo/bar");
        let actual = path.extend_with_url_path("baz/");
        let expected = PathBuf::from("/foo/bar/baz");
        assert_eq!(actual, expected);
    }

    #[test]
    fn extend_with_nested_url_path() {
        let path = PathBuf::from("/foo/bar");
        let actual = path.extend_with_url_path("baz/qux/quux");
        let expected = PathBuf::from("/foo/bar/baz/qux/quux");
        assert_eq!(actual, expected);
    }

    #[test]
    fn extend_with_multiple_leading_slashes() {
        let path = PathBuf::from("/foo/bar");
        let actual = path.extend_with_url_path("///baz");
        let expected = PathBuf::from("/foo/bar/baz");
        assert_eq!(actual, expected);
    }

    #[test]
    fn extend_with_parent_dir_is_not_normalized() {
        let path = PathBuf::from("/foo/bar");
        let actual = path.extend_with_url_path("baz/../qux");
        let expected = PathBuf::from("/foo/bar/baz/../qux");
        assert_eq!(actual, expected);
    }

    #[test]
    fn extend_with_dot_url_path() {
        let path = PathBuf::from("/foo/bar");
        let actual = path.extend_with_url_path(".");
        let expected = PathBuf::from("/foo/bar");
        assert_eq!(actual, expected);
    }
}
