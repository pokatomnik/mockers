use mimetype_detector::{APPLICATION_JSON, detect};
use serde::de::IgnoredAny;

pub fn get_mime(source: &[u8]) -> String {
    if serde_json::from_slice::<IgnoredAny>(&source).is_ok() {
        APPLICATION_JSON.to_owned()
    } else {
        detect(source).mime().to_string()
    }
}
