use std::io::ErrorKind;
use std::sync::{Arc, LazyLock};

use http_body_util::Full;
use hyper::{Request, Response, body::Bytes};
use routerify_ng::ext::RequestExt;

use crate::entities::mock_config::MockConfig;
use crate::entities::mockers_errors::MockersErrors;
use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::libs::mockers_request_ext::BodyReader;
use crate::libs::path_buf_ext::PathBufExt;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::server::controllers::dto::empty::Empty;
use crate::server::controllers::dto::mock_info::MockInfo;
use crate::server::mockers_context::MockersContext;
use crate::server::params::CONFIG_FILE_NAME;
use crate::server::protocol_result::ProtocolResultConverter;
use crate::server::route_error::MockersRouteError;
use crate::use_cases::absolute_mocks_path::AbsoluteMocksPath;

/// Handles the deletion of a mock endpoint based on the request parameters.
///
/// This function extracts mock information from the request body, locates the
/// corresponding mock file and configuration entry, and removes them from the
/// filesystem.
///
/// # Arguments
///
/// * `req` - A `Request<Full<Bytes>>` containing the full HTTP request with a
///   body that is expected to deserialize into `MockInfo`.
///
/// # Returns
///
/// A `Result` which contains:
/// - `Ok(Response<Full<Bytes>>)` - A response indicating the result of the
///   deletion operation (e.g., success, bad request, or internal error).
/// - `Err(MockersRouteError)` - An error if route handling fails.
pub async fn delete_mock(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    // Read the request body. If the body cannot be read, return an incorrect request response.
    let Some(body) = req.body().read().await else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    // Parse the body content into MockInfo. If parsing fails, return an incorrect request response.
    let Ok(request_params) = serde_json::from_str::<MockInfo>(&body) else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    // Retrieve the shared MockersContext from the request extensions. If not found, return a failure response.
    let Some(context) = req.data::<Arc<MockersContext>>() else {
        return Ok(DELETE_MOCK_FAILED.clone());
    };

    // Retrieve the absolute path to the mocks directory (e.g., /home/username/mocks).
    // If the path cannot be retrieved, return a failure response.
    let Some(ref absolute_mocks_path) = context.server_params.get_absolute_mocks_path().await
    else {
        return Ok(DELETE_MOCK_FAILED.clone());
    };

    // Construct the relative path for the mock file based on the request path and method.
    // Example: foo/bar/baz.post
    let relative_actual_pathname = format!(
        "{}.{}",
        &request_params.path(),
        &request_params.method().to_lowercase()
    )
    .trim_start_matches('/')
    .to_owned();

    // Construct the absolute path to the mock file to be deleted.
    // Example: /home/username/mocks/foo/bar/baz.post
    let absolute_actual_pathname = absolute_mocks_path.join(&relative_actual_pathname);

    // Determine the directory containing the mock file.
    // Example: /home/username/mocks/foo/bar
    let absolute_mock_dir = absolute_actual_pathname.with_last_removed();

    // Construct the absolute path to the configuration file.
    // Example: /home/username/mocks/foo/config.json
    let absolute_config_path = absolute_mock_dir.join(CONFIG_FILE_NAME);

    // Extract the file name (entry name) from the absolute path.
    // Example: baz.post
    let Some(ref entry_name) = absolute_actual_pathname
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
    else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    // Create futures for removing the mock file and updating the configuration file.
    let mock_remove_fut = tokio::fs::remove_file(&absolute_actual_pathname);
    let config_remove_fut = MockConfig::try_remove_from_file(&absolute_config_path, entry_name);

    // Execute both removal operations concurrently.
    let (mock_remove_result, _) = tokio::join!(mock_remove_fut, config_remove_fut);

    if let Err(ref e) = mock_remove_result
        && e.kind() == ErrorKind::NotFound
    {
        return Ok(MOCK_NOT_FOUND.clone());
    }

    // Check if both operations succeeded. If either failed, return a failure response.
    if !mock_remove_result.is_ok() {
        return Ok(DELETE_MOCK_FAILED.clone());
    }

    // Construct a success response body containing an empty entity.
    let body = Ok::<Empty, &str>(Empty::new())
        .to_protocol()
        .to_string()
        .into();

    // Build and return a successful response with the body.
    let response_ok = Response::ok()
        .content_type_json()
        .body(body)
        .unwrap_or_default();

    return Ok(response_ok);
}

static MOCK_NOT_FOUND: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::NotFound.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::not_found()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

static DELETE_MOCK_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::InternalServerError.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

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
