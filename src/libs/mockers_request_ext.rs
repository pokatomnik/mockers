use http_body_util::{BodyExt, Full};
use hyper::{Request, body::Bytes};

pub(crate) trait BodyReader {
    async fn read(&self) -> Option<String>;
}

impl BodyReader for Full<Bytes> {
    async fn read(&self) -> Option<String> {
        let body = self
            .clone()
            .collect()
            .await
            .map(|b| b.to_bytes().to_vec())
            .ok()
            .and_then(|b| String::from_utf8(b).ok())?;

        let Some(json) = serde_json::from_str::<serde_json::Value>(&body).ok() else {
            return Some(body);
        };
        let Some(pretty) = serde_json::to_string_pretty(&json).ok() else {
            return Some(body);
        };

        Some(pretty)
    }
}

pub(crate) trait Prompt {
    /// Converts value to markdown
    async fn to_markdown(&self) -> String;
}

impl Prompt for Request<Full<Bytes>> {
    async fn to_markdown(&self) -> String {
        let mut buf = String::with_capacity(512);
        buf.push_str("\n## Additional information:");

        buf.push_str(format!("\n### Request Method:\n{}", self.method().to_string()).as_str());
        buf.push_str(format!("\n### Request URI:\n{}", self.uri()).as_str());

        buf.push_str("\n### Request Headers:");
        for (header_key, header_val) in self.headers().iter() {
            if let Ok(header_val) = header_val.to_str() {
                let str = format!("\n{}: {}", header_key.to_string(), header_val);
                buf.push_str(str.as_str());
            }
        }

        let body = self.body().read().await;
        if let Some(body) = body {
            buf.push_str("\n### Request Body:\n");
            buf.push_str(body.as_str());
            buf.push('\n');
        }

        buf
    }
}
