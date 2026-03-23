use std::sync::{Arc, LazyLock};

use http_body_util::Full;
use hyper::{Request, Response, body::Bytes};
use routerify_ng::ext::RequestExt;
use tokio::join;

use crate::{
    controllers::entities::{empty::Empty, mock_create_params::MockCreateParams},
    libs::{
        absolute_mocks_path::AbsoluteMocksPath, mockers_errors::MockersErrors,
        mockers_request_ext::BodyReader, path_buf_ext::PathBufExt,
        protocol_result::ProtocolResultConverter, response_builder_ext::ResponseBuilderExt,
        response_ext::WellKnownResponses,
    },
    server::{
        mockers_context::MockersContext, params::CONFIG_FILE_NAME, route_error::MockersRouteError,
    },
};

pub async fn create_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let Some(body) = req.body().read().await else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    let Ok(request_params) = serde_json::from_str::<MockCreateParams>(&body) else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    let Some(context) = req.data::<Arc<MockersContext>>() else {
        return Ok(WRITE_MOCK_FAILED.clone());
    };

    // /home/username/mocks
    let Some(ref absolute_mocks_path) = context.server_params.get_absolute_mocks_path().await
    else {
        return Ok(WRITE_MOCK_FAILED.clone());
    };

    // foo/bar/baz.post
    let relative_actual_pathname = format!(
        "{}.{}",
        &request_params.mock_key().path(),
        &request_params.mock_key().method().to_lowercase()
    )
    .trim_start_matches('/')
    .to_owned();

    // /home/username/mocks/foo/bar/baz.post
    let absolute_actual_pathname = absolute_mocks_path.join(&relative_actual_pathname);

    // /home/username/mocks/foo/bar
    let absolute_mock_dir = absolute_actual_pathname.with_last_removed();

    // /home/username/mocks/foo/config.json
    let absolute_config_path = absolute_mock_dir.join(CONFIG_FILE_NAME);

    // baz.post
    let Some(ref entry_name) = absolute_actual_pathname
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
    else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    let dir_create_result = tokio::fs::create_dir_all(absolute_mock_dir).await;

    if dir_create_result.is_err() {
        return Ok(WRITE_MOCK_FAILED.clone());
    }

    let body_file_create_fut =
        tokio::fs::write(&absolute_actual_pathname, request_params.body().as_bytes());

    let config_create_fut = request_params
        .mock_config()
        .try_write_to_file(&absolute_config_path, entry_name);

    let (body_write_result, config_write_result) = join!(body_file_create_fut, config_create_fut);

    let write_ok = body_write_result.is_ok() && config_write_result.is_ok();
    if !write_ok {
        return Ok(WRITE_MOCK_FAILED.clone());
    }

    let body = Ok::<Empty, &str>(Empty::new())
        .to_protocol()
        .to_string()
        .into();
    let response_ok = Response::ok()
        .content_type_json()
        .body(body)
        .unwrap_or_default();

    return Ok(response_ok);
}

static WRITE_MOCK_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::InternalServerError.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

/// A static response for "Bad Request" errors.
static INCORRECT_REQUEST: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::BadRequest.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::bad_request()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});
