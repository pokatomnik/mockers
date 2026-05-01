use mimetype_detector::APPLICATION_JSON;
use mimetype_detector::APPLICATION_OCTET_STREAM;
use mimetype_detector::TEXT_PLAIN;
use mimetype_detector::TEXT_UTF8;
use mimetype_detector::TEXT_UTF16_BE;
use mimetype_detector::TEXT_UTF16_LE;
use mimetype_detector::detect;

use lru::LruCache;
use raffia::ast::Stylesheet;
use raffia::{Parser, Syntax};
use serde::de::IgnoredAny;
use serde::ser::StdError;
use std::num::NonZeroUsize;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use xxhash_rust::xxh3::xxh3_128_with_seed;

static LRU_SIZE: usize = 1024;
static SEED: u64 = 128;
const CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(LRU_SIZE).unwrap();

pub(crate) static TEXT_CSS: &'static str = "text/css";

struct MimeCache {
    seed: u64,
    shared_data: Mutex<LruCache<u128, String>>,
}

impl MimeCache {
    fn with_capacity(cap: NonZeroUsize, seed: u64) -> Self {
        MimeCache {
            shared_data: Mutex::new(LruCache::new(cap)),
            seed,
        }
    }

    #[inline]
    fn hash(&self, bytes: &[u8]) -> u128 {
        xxh3_128_with_seed(bytes, self.seed)
    }

    #[inline]
    fn is_css(source: &[u8]) -> bool {
        let src_vec = source.to_owned();
        let Ok(slice) = str::from_utf8(&src_vec) else {
            return false;
        };
        let mut parser = Parser::new(slice, Syntax::Css);
        parser.parse::<Stylesheet>().is_ok()
    }

    #[inline]
    fn is_json(source: &[u8]) -> bool {
        serde_json::from_slice::<IgnoredAny>(&source).is_ok()
    }

    async fn get_cached_mime(&self, raw_data: &[u8], fallback: &'static str) -> String {
        if raw_data.is_empty() {
            return APPLICATION_OCTET_STREAM.to_string();
        }

        let data_hash = self.hash(raw_data);

        let mut cache = self.shared_data.lock().await;
        let cached_mime = cache.get(&data_hash);

        if let Some(mime) = cached_mime {
            return mime.to_owned();
        }

        let mime = match Self::get_mime(raw_data).await {
            Ok(mime) => mime.to_owned(),
            Err(_) => fallback.to_owned(),
        };

        cache.put(data_hash, mime.clone());

        mime
    }

    async fn get_mime(source: &[u8]) -> Result<String, Box<dyn StdError + Sync + Send>> {
        let source_vec = source.to_owned();
        tokio::task::spawn_blocking(move || {
            // Serde-based probe, the most common case
            if Self::is_json(source_vec.as_slice()) {
                return APPLICATION_JSON.to_string();
            }

            // A common signature-based probe, not precise
            let detected_mime = detect(source_vec.as_slice()).mime();
            match detected_mime {
                // Serde-based probe told us this is NOT JSON, do not trust signature based probe, skip
                APPLICATION_JSON => {}
                TEXT_PLAIN | TEXT_UTF8 | TEXT_UTF16_BE | TEXT_UTF16_LE => {}
                _ => return detected_mime.to_string(),
            }

            // CSS probe: not supported by mimetype-detector yet https://github.com/Asuan/mimetype-detector/issues/4
            let result = Self::is_css(source_vec.as_slice());

            match result {
                true => TEXT_CSS.to_string(),
                false => TEXT_UTF8.to_string(),
            }
        })
        .await
        .map_err(Box::from)
    }
}

static MIME_CACHE: LazyLock<MimeCache> =
    LazyLock::new(|| MimeCache::with_capacity(CACHE_SIZE, SEED));

pub(crate) async fn get_mime(raw_data: &[u8]) -> String {
    MIME_CACHE.get_cached_mime(raw_data, TEXT_UTF8).await
}

#[cfg(test)]
mod tests {
    use mimetype_detector::{IMAGE_JPEG, IMAGE_PNG, IMAGE_X_ICON, TEXT_HTML};

    use super::*;

    #[tokio::test]
    async fn test_get_mime_happy_pass() {
        let actual = get_mime(b"{\"key\":\"value\"}").await;
        let expected = APPLICATION_JSON;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_with_spaces() {
        let actual = get_mime(b"        {\"key\":\"value\"}").await;
        let expected = APPLICATION_JSON;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_of_null() {
        let actual = get_mime(b"        null").await;
        let expected = APPLICATION_JSON;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_plain_text() {
        let actual = get_mime(b"this is a plaintext string").await;
        let expected = TEXT_UTF8;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_with_newlines() {
        let actual = get_mime(b"\n\n{\"key\":\"value\"}").await;
        let expected = APPLICATION_JSON;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_with_incorrect_json() {
        let actual = get_mime(b"{\"key\":\"value\",\n}").await;
        let expected = TEXT_UTF8;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_png() {
        let actual = get_mime(b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR").await;
        let expected = IMAGE_PNG;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_jpg() {
        let actual = get_mime(b"\xFF\xD8\xFF\xE0\x00\x10JFIF").await;
        let expected = IMAGE_JPEG;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_ico() {
        let actual = get_mime(b"\x00\x00\x01\x00\x01\x00\x10\x10").await;
        let expected = IMAGE_X_ICON;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_js() {
        let actual = get_mime(b"function test() { return true; }").await;
        assert_ne!(actual, APPLICATION_JSON);
    }

    #[tokio::test]
    async fn test_get_mime_css() {
        let actual = get_mime(b".x{--a:0;display:grid;place-items:center;aspect-ratio:1;background:conic-gradient(from var(--a),red,blue);clip-path:polygon(50% 0,100% 50%,50% 100%,0 50%)}").await;
        let expected = TEXT_CSS;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_get_mime_html() {
        let actual = get_mime(b"<!doctype html><html><head><title>hello world</title></head><body>This is HTML file</body></html>").await;
        let expected = TEXT_HTML;

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn empty() {
        let actual = get_mime(b"").await;
        let expected = APPLICATION_OCTET_STREAM;

        assert_eq!(actual, expected);
    }
}
