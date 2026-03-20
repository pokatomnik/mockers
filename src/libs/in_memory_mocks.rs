use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use std::error::Error as StdError;

pub trait PathDecoder {
    fn decode_path(&self) -> Result<String, Box<dyn StdError>>;
}

impl PathDecoder for String {
    fn decode_path(&self) -> Result<String, Box<dyn StdError>> {
        let bytes = BASE64_STANDARD.decode(self)?;
        let str = String::from_utf8(bytes)?;
        Ok(str)
    }
}

pub trait PathNormalizer {
    fn normalize_path(&self) -> String;
}

impl PathNormalizer for String {
    fn normalize_path(&self) -> String {
        self.trim_matches('/').to_string()
    }
}

pub trait MethodNormalizer {
    fn normalize_method(&self) -> String;
}

impl MethodNormalizer for String {
    fn normalize_method(&self) -> String {
        self.to_lowercase()
    }
}
