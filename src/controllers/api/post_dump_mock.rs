use crate::libs::absolute_mocks_path::AbsoluteMocksPath;
use crate::libs::in_memory_mocks::{MethodNormalizer, PathDecoder, PathNormalizer};
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::path_buf_ext::PathBufExt;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::mockers_context::MockersContext;
use crate::server::params::CONFIG_FILE_NAME;
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use path_absolutize::Absolutize;
use routerify_ng::ext::RequestExt;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use tokio::join;

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
        return Ok(GET_MOCKS_BY_PATH_AND_METHOD_FAILED.clone());
    };

    let Some(cached) = cache.get_mock_by_path_and_method(&path, &method).await else {
        return Ok(NO_SUCH_MOCK.clone());
    };

    let absolute_mocks_dir = match req
        .data::<Arc<MockersContext>>()
        .map(|ctx| &ctx.server_params)
    {
        None => None,
        Some(sp) => sp.get_absolute_mocks_path().await,
    };

    let Some(absolute_mocks_dir) = absolute_mocks_dir else {
        return Ok(MISSING_MOCKS_DIRECTORY.clone());
    };

    let target_file_path_without_extension =
        absolute_mocks_dir.join(path).to_string_lossy().to_string();

    let absolute_target_file_name: PathBuf =
        format!("{}.{}", &target_file_path_without_extension, &method).into();

    let Ok(absolute_target_file_name) = absolute_target_file_name.absolutize() else {
        return Ok(MISSING_MOCKS_DIRECTORY.clone());
    };

    let absolute_target_file_name = absolute_target_file_name.to_path_buf().clone();
    let absolute_target_directory = absolute_target_file_name.with_last_removed();
    let absolute_target_config_file_name = absolute_target_file_name
        .with_last_removed()
        .join(CONFIG_FILE_NAME);
    let config_entry_name = absolute_target_file_name
        .file_name()
        .map(|s| s.to_string_lossy().to_string());

    let is_dir = tokio::fs::metadata(&absolute_target_directory)
        .await
        .map(|md| md.is_dir())
        .unwrap_or(false);

    // If directory does not exist or failed to create
    // Make an attempt to create It
    if !is_dir && let Err(_) = tokio::fs::create_dir_all(&absolute_target_directory).await {
        return Ok(MISSING_MOCKS_DIRECTORY.clone());
    }

    if let Some(ref config_entry_name) = config_entry_name {
        let config_dump_fut =
            cached.dump_config(config_entry_name, &absolute_target_config_file_name);
        let response_dump_fut = cached.dump_response(&absolute_target_file_name);
        let (config_dump_result, response_dump_result) = join!(config_dump_fut, response_dump_fut);
        let response: Response<Full<Bytes>> = match (config_dump_result, response_dump_result) {
            (Ok(_), Ok(_)) => MOCK_AND_CONFIG_WRITE_OK.clone(),
            (Err(_), Err(_)) => MOCK_AND_CONFIG_WRITE_FAILED.clone(),
            (Err(_), Ok(_)) => MOCK_CONFIG_WRITE_FAILED.clone(),
            (Ok(_), Err(_)) => MOCK_WRITE_FAILED.clone(),
        };

        return Ok(response);
    }

    Ok(MOCK_CONFIG_WRITE_FAILED.clone())
}

static GET_MOCKS_BY_PATH_AND_METHOD_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::GetMocksByPathAndMethodFailed.to_string())
        .to_protocol()
        .to_string()
        .into();
    return Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default();
});

static NO_SUCH_MOCK: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::NoSuchMock.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::not_found()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

static MISSING_MOCKS_DIRECTORY: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::MissingMocksDirectory.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

static MOCK_AND_CONFIG_WRITE_OK: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Ok::<&str, &str>("Mock and config written")
        .to_protocol()
        .to_string()
        .into();
    Response::ok()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

static MOCK_AND_CONFIG_WRITE_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::DumpMockAndMockConfigFailed.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

static MOCK_CONFIG_WRITE_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::DumpMockConfigFailed.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

static MOCK_WRITE_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::DumpMockBodyFailed.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});
