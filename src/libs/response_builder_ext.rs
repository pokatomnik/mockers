use crate::libs::get_mime::TEXT_CSS;
use crate::libs::headers::{CORS_HEADER_KEYS, CORS_HEADER_VALUE};
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;
use hyper::header::{CONTENT_TYPE, HeaderName, HeaderValue};
use hyper::http::response::Builder;
use mimetype_detector::APPLICATION_JAVASCRIPT;
use mimetype_detector::APPLICATION_JSON;
use mimetype_detector::APPLICATION_OCTET_STREAM;
use mimetype_detector::IMAGE_PNG;
use mimetype_detector::TEXT_HTML;
use mimetype_detector::TEXT_PLAIN;

pub(crate) trait ResponseBuilderExt {
    fn add_cors(self) -> Self;

    fn add_custom_headers(self, headers: impl Iterator<Item = (String, String)>) -> Self;

    fn add_content_type_header(self, content_type: &str) -> Self;

    fn content_type_json(self) -> Self;

    fn content_type_octet_stream(self) -> Self;

    fn content_type_html(self) -> Self;

    fn content_type_png(self) -> Self;

    fn content_type_css(self) -> Self;

    fn content_type_js(self) -> Self;

    fn content_type_text_plain(self) -> Self;

    fn empty_body(self) -> Result<Response<Full<Bytes>>, hyper::http::Error>;
}

impl ResponseBuilderExt for Builder {
    fn add_cors(mut self) -> Self {
        if let Some(headers) = self.headers_mut() {
            let anything_header: Result<HeaderValue, _> = CORS_HEADER_VALUE.parse();
            if let Ok(header_val) = anything_header {
                for header in CORS_HEADER_KEYS.iter() {
                    headers.insert(*header, header_val.clone());
                }
            }
        }

        self
    }

    fn add_custom_headers(mut self, headers_iter: impl Iterator<Item = (String, String)>) -> Self {
        if let Some(req_headers) = self.headers_mut() {
            for (header_key, header_value) in headers_iter {
                if let Some((header_key, header_val)) = header_key
                    .parse::<HeaderName>()
                    .ok()
                    .zip(header_value.parse().ok())
                {
                    req_headers.insert(header_key, header_val);
                }
            }
        }

        self
    }

    fn add_content_type_header(mut self, content_type: &str) -> Self {
        let checked = self
            .headers_mut()
            .zip(HeaderValue::from_str(content_type.into()).ok());

        if let Some((req_headers, content_type)) = checked {
            req_headers.insert(CONTENT_TYPE, content_type);
        }

        self
    }

    fn content_type_json(mut self) -> Self {
        if let Some(headers) = self.headers_mut() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON));
        }
        self
    }

    fn content_type_octet_stream(mut self) -> Self {
        if let Some(headers) = self.headers_mut() {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static(APPLICATION_OCTET_STREAM),
            );
        }
        self
    }

    fn content_type_html(mut self) -> Self {
        if let Some(headers) = self.headers_mut() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static(TEXT_HTML));
        }
        self
    }

    fn content_type_png(mut self) -> Self {
        if let Some(headers) = self.headers_mut() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static(IMAGE_PNG));
        }
        self
    }

    fn content_type_css(mut self) -> Self {
        if let Some(headers) = self.headers_mut() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static(TEXT_CSS));
        }
        self
    }

    fn content_type_js(mut self) -> Self {
        if let Some(headers) = self.headers_mut() {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static(APPLICATION_JAVASCRIPT),
            );
        }
        self
    }

    fn content_type_text_plain(mut self) -> Self {
        if let Some(headers) = self.headers_mut() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static(TEXT_PLAIN));
        }
        self
    }

    fn empty_body(self) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
        self.body(Full::default())
    }
}
