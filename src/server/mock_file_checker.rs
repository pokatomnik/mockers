use crate::libs::http_method::StandardMethodValidator;
use std::{path::PathBuf, str::FromStr};

pub(crate) trait MockFileChecker {
    fn is_mock_file(&self) -> bool;
}

impl MockFileChecker for PathBuf {
    fn is_mock_file(&self) -> bool {
        let Some(extension) = self.extension().map(|e| e.to_string_lossy().to_uppercase()) else {
            return false;
        };

        let Ok(method) = hyper::Method::from_str(&extension) else {
            return false;
        };

        method.validate(|| ()).is_ok()
    }
}
