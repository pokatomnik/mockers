use crate::libs::tap::Tap;
use reqwest::Url;
use std::error::Error as StdError;

pub(crate) trait UrlExt {
    fn from_parts(
        origin: impl Into<String>,
        path: impl Into<String>,
        query: Option<&str>,
    ) -> Result<Url, impl StdError>;
}

impl UrlExt for Url {
    fn from_parts(
        origin: impl Into<String>,
        path: impl Into<String>,
        query: Option<&str>,
    ) -> Result<Url, impl StdError> {
        let origin = origin.into().tap(|o| o.trim_end_matches("/").to_string());
        let path = path.into().tap(|p| p.trim_start_matches("/").to_string());
        let uri_str = match query {
            Some(params_str) => format!("{}/{}?{}", origin, path, params_str),
            None => format!("{}/{}", origin, path),
        };
        Url::parse(&uri_str)
    }
}
