use std::sync::{Arc, LazyLock};

use http_body_util::Full;
use hyper::{Request, Response, body::Bytes};
use routerify_ng::ext::RequestExt;
use tokio::join;

use crate::controllers::entities::empty::Empty;
use crate::controllers::entities::mock_create_params::MockCreateParams;
use crate::libs::absolute_mocks_path::AbsoluteMocksPath;
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::mockers_request_ext::BodyReader;
use crate::libs::path_buf_ext::PathBufExt;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::mockers_context::MockersContext;
use crate::server::params::CONFIG_FILE_NAME;
use crate::server::route_error::MockersRouteError;

/// Handles the creation of a new mock endpoint based on the incoming request parameters.
///
/// This function parses the request body to determine the mock configuration,
/// constructs the necessary file paths, and persists both the mock body and
/// its configuration to the file system.
///
/// # Arguments
///
/// * `req` - A hyper `Request` containing the `MockCreateParams` in the body.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(Response)`: An empty JSON response on success.
/// - `Ok(Response)`: A static error response (`INCORRECT_REQUEST` or `WRITE_MOCK_FAILED`) on failure.
/// - `Err(MockersRouteError)`: A route error if processing fails (though currently unused).
pub async fn create_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    // Attempt to read the request body. If the body is missing, return a "Bad Request" response.
    let Some(body) = req.body().read().await else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    // Attempt to deserialize the body into `MockCreateParams`. If deserialization fails, return a "Bad Request" response.
    let Ok(request_params) = serde_json::from_str::<MockCreateParams>(&body) else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    // Retrieve the `MockersContext` from the request extensions. If missing, return an internal error.
    let Some(context) = req.data::<Arc<MockersContext>>() else {
        return Ok(WRITE_MOCK_FAILED.clone());
    };

    // Retrieve the absolute path to the mocks directory (e.g., /home/username/mocks).
    // If it cannot be determined, return an internal error.
    let Some(ref absolute_mocks_path) = context.server_params.get_absolute_mocks_path().await
    else {
        return Ok(WRITE_MOCK_FAILED.clone());
    };

    // Construct the relative path for the mock file using the path and method (e.g., foo/bar/baz.post).
    let relative_actual_pathname = format!(
        "{}.{}",
        &request_params.mock_key().path(),
        &request_params.mock_key().method().to_lowercase()
    )
    .trim_start_matches('/')
    .to_owned();

    // Join the mocks directory with the relative path to get the full path for the mock file.
    // e.g., /home/username/mocks/foo/bar/baz.post
    let absolute_actual_pathname = absolute_mocks_path.join(&relative_actual_pathname);

    // Determine the directory containing the mock file (e.g., /home/username/mocks/foo/bar).
    let absolute_mock_dir = absolute_actual_pathname.with_last_removed();

    // Determine the path for the configuration file associated with this mock (e.g., /home/username/mocks/foo/config.json).
    let absolute_config_path = absolute_mock_dir.join(CONFIG_FILE_NAME);

    // Extract the file name (entry name) from the absolute path (e.g., baz.post).
    let Some(ref entry_name) = absolute_actual_pathname
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
    else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    // Create the directory structure asynchronously. Return an error if creation fails.
    let dir_create_result = tokio::fs::create_dir_all(absolute_mock_dir).await;

    if dir_create_result.is_err() {
        return Ok(WRITE_MOCK_FAILED.clone());
    }

    // Prepare the future for writing the mock body to the file.
    let body_file_create_fut =
        tokio::fs::write(&absolute_actual_pathname, request_params.body().as_bytes());

    // Prepare the future for writing the mock configuration to the config file.
    let config_create_fut = request_params
        .mock_config()
        .try_write_to_file(&absolute_config_path, entry_name);

    // Execute both file write operations concurrently.
    let (body_write_result, config_write_result) = join!(body_file_create_fut, config_create_fut);

    // Check if both write operations succeeded. If either failed, return an internal error.
    let write_ok = body_write_result.is_ok() && config_write_result.is_ok();
    if !write_ok {
        return Ok(WRITE_MOCK_FAILED.clone());
    }

    // Construct a success response containing an empty JSON body.
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
