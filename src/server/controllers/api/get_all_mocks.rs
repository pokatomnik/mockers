use crate::entities::mockers_errors::MockersErrors;
use crate::libs::fs_walker::FSWalker;
use crate::libs::hyper_response_ext::HyperWellKnownResponses;

use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::server::controllers::dto::mock_info::MockInfo;
use crate::server::mock_file_checker::MockFileChecker;
use crate::server::mockers_context::MockersContext;
use crate::server::protocol_result::ProtocolResultConverter;
use crate::server::route_error::MockersRouteError;
use crate::use_cases::absolute_mocks_path::AbsoluteMocksPath;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use routerify_ng::ext::RequestExt;
use std::sync::{Arc, LazyLock};

/// Handles the `GET /mocks` request to list all available mocks.
///
/// This function traverses the mock directory specified in the server configuration,
/// identifies mock files based on their extension, and extracts the HTTP method
/// and pathname from the file name.
///
/// # Arguments
/// * `req` - The incoming HTTP request containing server context data.
///
/// # Returns
/// A `Result` containing a JSON response with a list of `MockInfo` objects
/// representing the discovered mocks, or a pre-defined error response.
pub async fn get_all_mocks(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    // Attempt to retrieve the shared MockersContext from request extensions.
    // Return a failure response if the context is missing.
    let Some(context) = req.data::<Arc<MockersContext>>() else {
        return Ok(GET_ALL_MOCKS_FAILED.clone());
    };

    // Asynchronously retrieve the absolute path to the mocks directory.
    // Return a failure response if the path cannot be determined.
    let Some(ref absolute_mocks_path) = context.server_params.get_absolute_mocks_path().await
    else {
        return Ok(GET_ALL_MOCKS_FAILED.clone());
    };

    // Initialize a filesystem walker for the mocks directory.
    let fs_walker = FSWalker::new(absolute_mocks_path);

    // Iterate over files in the directory to find and parse mocks.
    let all_mocks: Vec<MockInfo> = fs_walker
        .into_iter()
        .await
        .filter_map(
            move |(md, path)| match (md.is_file(), path.is_mock_file()) {
                // Process only files that match the mock file signature.
                (true, true) => {
                    // Strip the base mocks directory path to get the relative path + extension.
                    let pathname_with_extension = path.to_string_lossy().to_string().replace(
                        absolute_mocks_path.to_string_lossy().to_string().as_str(),
                        "",
                    );
                    // Split the remaining string into pathname and extension (method).
                    // e.g., "/api/users.get" -> pathname="/api/users", method="GET".
                    let Some((pathname, method)) = pathname_with_extension.rsplit_once('.') else {
                        return None;
                    };
                    Some((pathname.to_string(), method.to_uppercase()))
                }
                _ => None,
            },
        )
        // Map the extracted tuples into MockInfo structs.
        .map(|(pathname, method)| MockInfo::new(method, pathname))
        .collect();

    // Wrap the result in a protocol-compliant structure and serialize to JSON.
    let result = Result::<Vec<MockInfo>, String>::Ok(all_mocks).to_protocol();
    let result_json = serde_json::to_string(&result).unwrap();

    // Build the successful HTTP response.
    let response = Response::ok()
        .content_type_json()
        .body(result_json.into())
        .unwrap_or_default();

    Ok(response)
}

static GET_ALL_MOCKS_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::InternalServerError.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});
