use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::{MethodNormalizer, PathDecoder, PathNormalizer};
use crate::server::mockers_context::MockersContext;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use path_absolutize::Absolutize;
use routerify_ng::ext::RequestExt;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{create_dir_all, metadata, write};

pub async fn post_dump_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req
        .params()
        .get("path_encoded")
        .and_then(|str| str.decode_path().ok())
        .unwrap_or(String::new())
        .normalize_path();
    let method = req
        .params()
        .get("method")
        .map(|v| v.to_owned())
        .unwrap_or(String::new())
        .normalize_method();

    let response_cache = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().response_cache.clone());

    let Some(cache) = response_cache else {
        let json = serde_json::to_string(
            &Err::<String, String>(MockersErrors::GetMocksByPathAndMethodFailed.to_string())
                .to_protocol(),
        );
        let bytes = json.map(|s| Bytes::from(s)).unwrap_or(Bytes::new());
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(Full::new(Bytes::from(bytes)))
            .unwrap_or_default());
    };

    let Some(cached) = cache.get_mock_by_path_and_method(&path, &method).await else {
        let json = serde_json::to_string(
            &Err::<String, String>(MockersErrors::NoSuchMock.to_string()).to_protocol(),
        );
        let status_code = json
            .as_ref()
            .map(|_| StatusCode::NOT_FOUND)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let bytes = json.map(|str| Bytes::from(str)).unwrap_or(Bytes::new());
        return Ok(Response::builder()
            .status(status_code)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(Full::new(bytes))
            .unwrap_or_default());
    };

    let Some(absolute_mocks_dir) = req
        .data::<Arc<MockersContext>>()
        .map(|c| c.clone().server_params.get_absolute_mocks_path())
        .unwrap_or_else(|| Err(Box::from("mocks directory not provided")))
        .ok()
    else {
        let json = serde_json::to_string(
            &Err::<String, String>(MockersErrors::MissingMocksDirectory.to_string()).to_protocol(),
        );
        let bytes = json.map(|str| Bytes::from(str)).unwrap_or(Bytes::new());
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(Full::new(bytes))
            .unwrap_or_default());
    };

    let target_file_path_without_extension =
        absolute_mocks_dir.join(path).to_string_lossy().to_string();

    let absolute_target_file_name = PathBuf::from(format!(
        "{}.{}",
        &target_file_path_without_extension, &method
    ));
    let Ok(absolute_target_file_name) = absolute_target_file_name.absolutize() else {
        let json = serde_json::to_string(
            &Err::<String, String>(MockersErrors::MissingMocksDirectory.to_string()).to_protocol(),
        );
        let bytes = json.map(|s| Bytes::from(s)).unwrap_or(Bytes::new());
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(Full::new(Bytes::from(bytes)))
            .unwrap_or_default());
    };

    let (absolute_target_file_name, absolute_target_directory) =
        (absolute_target_file_name.to_path_buf().clone(), {
            let mut result = absolute_target_file_name.to_path_buf().clone();
            result.pop();
            result
        });

    if let Ok(md) = metadata(&absolute_target_directory).await
        && md.is_dir()
    {
        return write_file_contents_and_get_response(&absolute_target_file_name, &cached.body)
            .await;
    }

    if let Err(_) = create_dir_all(&absolute_target_directory).await {
        let json = serde_json::to_string(
            &Err::<String, String>(MockersErrors::MissingMocksDirectory.to_string()).to_protocol(),
        );
        let bytes = json.map(|s| Bytes::from(s)).unwrap_or(Bytes::new());
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(Full::new(Bytes::from(bytes)))
            .unwrap_or_default());
    }

    write_file_contents_and_get_response(&absolute_target_file_name, &cached.body).await
}

async fn write_file_contents_and_get_response(
    to: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let result_write = write(&to, &contents).await.ok();
    let result_json = serde_json::to_string(
        &Ok::<String, String>(format!(
            "Mock written to: '{}'",
            to.as_ref().to_path_buf().to_string_lossy().to_string()
        ))
        .to_protocol(),
    )
    .ok();
    let result = result_write.zip(result_json.clone());
    let status_code = result
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = result_json.map(|str| Bytes::from(str)).unwrap_or_default();

    Ok(Response::builder()
        .status(status_code)
        .header(CONTENT_TYPE, APPLICATION_JSON)
        .body(Full::from(Bytes::from(bytes)))
        .unwrap_or_default())
}
