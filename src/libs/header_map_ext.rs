use crate::libs::headers::{CORS_HEADER_KEYS, CORS_HEADER_VALUE};
use hyper::header::HeaderValue;
use hyper::HeaderMap;
use std::collections::HashMap;

pub(crate) trait HeaderMapSanitizer {
    fn remove_host_header(self) -> Self;

    fn remove_content_length_header(self) -> Self;
}

impl HeaderMapSanitizer for HeaderMap {
    fn remove_host_header(mut self) -> Self {
        self.remove(hyper::header::HOST);
        self
    }

    fn remove_content_length_header(mut self) -> Self {
        self.remove(hyper::header::CONTENT_LENGTH);
        self
    }
}

pub(crate) trait HeaderMapConverter {
    fn kv_iter(&self) -> impl Iterator<Item = (String, String)>;

    fn to_hash_map(&self) -> HashMap<String, String>;
}

impl HeaderMapConverter for HeaderMap {
    fn kv_iter(&self) -> impl Iterator<Item = (String, String)> {
        self.iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
    }

    fn to_hash_map(&self) -> HashMap<String, String> {
        let mut headers_map = HashMap::with_capacity(self.keys_len());
        for (header_name, header_value) in self {
            if let Ok(header_value) = header_value.to_str() {
                headers_map.insert(header_name.to_string(), header_value.to_string());
            }
        }
        headers_map
    }
}

pub(crate) trait HeaderMapExt {
    fn add_cors(&mut self);
}

impl HeaderMapExt for HeaderMap {
    fn add_cors(&mut self) {
        let anything_header: Result<HeaderValue, _> = CORS_HEADER_VALUE.parse();
        if let Ok(header_val) = anything_header {
            for header in CORS_HEADER_KEYS.iter() {
                self.insert(*header, header_val.clone());
            }
        }
    }
}
