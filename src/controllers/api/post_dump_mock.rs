use crate::libs::mock_config::MockConfig;
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_cache::{MethodNormalizer, PathDecoder, PathNormalizer};
use crate::server::mockers_context::MockersContext;
use crate::server::params::CONFIG_FILE_NAME;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response, StatusCode};
use mimetype_detector::APPLICATION_JSON;
use path_absolutize::Absolutize;
use routerify_ng::ext::RequestExt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub async fn post_dump_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
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

    let (
        absolute_target_file_name,
        absolute_target_directory,
        absolute_target_config_file_name,
        config_entry_name,
    ) = (
        absolute_target_file_name.to_path_buf().clone(),
        {
            let mut result = absolute_target_file_name.to_path_buf().clone();
            result.pop();
            result
        },
        {
            let mut result = absolute_target_file_name.to_path_buf().clone();
            result.pop();
            result.join(CONFIG_FILE_NAME)
        },
        {
            absolute_target_file_name
                .to_path_buf()
                .clone()
                .iter()
                .last()
                .map(|s| s.to_string_lossy().to_string())
        },
    );

    if let Ok(md) = tokio::fs::metadata(&absolute_target_directory).await
        && md.is_dir()
    {
        if let Some(config_entry_name) = config_entry_name {
            // TODO: add logging here
            let _result = write_mock_config(
                absolute_target_config_file_name,
                config_entry_name,
                cached.delay_ms,
                cached.status_code,
                &cached.headers,
            )
            .await;
        }
        return write_file_contents_and_get_response(&absolute_target_file_name, &cached.body)
            .await;
    }

    if let Err(_) = tokio::fs::create_dir_all(&absolute_target_directory).await {
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

    if let Some(config_entry_name) = config_entry_name {
        // TODO: add logging here
        let _result = write_mock_config(
            absolute_target_config_file_name,
            config_entry_name,
            cached.delay_ms,
            cached.status_code,
            &cached.headers,
        )
        .await;
    }

    write_file_contents_and_get_response(&absolute_target_file_name, &cached.body).await
}

async fn write_mock_config(
    to: impl AsRef<Path>,
    entry_name: impl Into<String>,
    delay_ms: u64,
    status_code: u16,
    headers: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mock_config = {
        let mut mock_config = MockConfig::new();
        if !headers.is_empty() {
            mock_config = mock_config.with_headers(headers.to_owned())
        }
        if delay_ms != 0 {
            mock_config = mock_config.with_delay_ms(delay_ms)
        }
        mock_config.with_status_code(status_code)
    };
    let existing_config = tokio::fs::read(&to).await.ok().and_then(|contents| {
        serde_json::from_slice::<HashMap<String, MockConfig>>(&contents.as_ref()).ok()
    });

    if let Some(mut existing_config) = existing_config {
        existing_config.insert(entry_name.into(), mock_config);
        let updated = serde_json::to_string(&existing_config)?;
        tokio::fs::write(to, updated).await?;
    } else {
        let mut new_map = HashMap::with_capacity(1);
        new_map.insert(entry_name.into(), mock_config);
        let json = serde_json::to_string(&new_map)?;
        tokio::fs::write(to, json).await?;
    }

    Ok(())
}

async fn write_file_contents_and_get_response(
    to: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let result_write = tokio::fs::write(&to, &contents).await.ok();
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
