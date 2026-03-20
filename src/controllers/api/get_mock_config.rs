use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use http_body_util::Full;
use hyper::{Request, Response, body::Bytes};
use routerify_ng::ext::RequestExt;

use crate::controllers::entities::mock_info::MockInfo;
use crate::libs::absolute_mocks_path::AbsoluteMocksPath;
use crate::libs::fs_walker;
use crate::libs::mock_config::MockConfig;
use crate::libs::mockers_errors::MockersErrors;
use crate::libs::mockers_request_ext::BodyReader;
use crate::libs::path_buf_ext::PathBufExt;
use crate::libs::protocol_result::ProtocolResultConverter;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::response_ext::WellKnownResponses;
use crate::server::mockers_context::MockersContext;
use crate::server::params::CONFIG_FILE_NAME;
use crate::server::route_error::MockersRouteError;

pub async fn get_mock_config(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let Some(body) = req.body().read().await else {
        return Ok(INCORRECT_REQUEST.clone());
    };
    let request_params = serde_json::from_str::<MockInfo>(&body);

    let Ok(request_params) = request_params else {
        return Ok(INCORRECT_REQUEST.clone());
    };

    let Some(context) = req.data::<Arc<MockersContext>>() else {
        return Ok(GET_MOCK_CONFIG_FAILED.clone());
    };

    let Some(ref absolute_mocks_path) = context.server_params.get_absolute_mocks_path().await
    else {
        return Ok(GET_MOCK_CONFIG_FAILED.clone());
    };

    let fs_walker = fs_walker::FSWalker::new(absolute_mocks_path);
    let Some((config_absolute_path, config_entry_name)) = fs_walker
        .into_iter()
        .await
        .filter_map(|(md, pb)| match (md.is_file(), pb.is_mock_file()) {
            (true, true) => Some(pb),
            _ => None,
        })
        .find_map(|ref current_mock_absolute_path| {
            check_match(current_mock_absolute_path, &request_params)
        })
    else {
        return Ok(NOT_FOUND.clone());
    };

    let Ok(configs_by_path) = MockConfig::try_read_from_file(config_absolute_path).await else {
        let default_mock_config = MockConfig::default();
        let result = Result::<MockConfig, String>::Ok(default_mock_config)
            .to_protocol()
            .to_string();
        let response = Response::ok()
            .content_type_json()
            .body(result.into())
            .unwrap_or_default();
        return Ok(response);
    };

    let Some(config_by_entry) = configs_by_path.get(&config_entry_name) else {
        let default_mock_config = MockConfig::default();
        let result = Result::<MockConfig, String>::Ok(default_mock_config)
            .to_protocol()
            .to_string();
        let response = Response::ok()
            .content_type_json()
            .body(result.into())
            .unwrap_or_default();
        return Ok(response);
    };

    let result = Result::<MockConfig, String>::Ok(config_by_entry.clone())
        .to_protocol()
        .to_string();
    let response = Response::ok()
        .content_type_json()
        .body(result.into())
        .unwrap_or_default();

    return Ok(response);
}

fn check_match(
    current_mock_absolute_path: impl AsRef<Path>,
    request_params: &MockInfo,
) -> Option<(PathBuf, String)> {
    // Example: mOcK.get
    let current_mock_file_name = current_mock_absolute_path
        .as_ref()
        .file_name()
        .map(|v| v.to_string_lossy().to_string());
    // Example: get
    let current_mock_extension_lower = current_mock_absolute_path
        .as_ref()
        .extension()
        .map(|v| v.to_string_lossy().to_lowercase());

    let Some((current_mock_file_name, current_mock_extension_lower)) =
        current_mock_file_name.zip(current_mock_extension_lower)
    else {
        return None;
    };

    // "get".to_uppercase() == "GET"
    if current_mock_extension_lower.to_uppercase() != request_params.method().to_uppercase() {
        return None;
    }

    let file_name_probe = format!("{}.{}", request_params.path(), current_mock_extension_lower);

    // Checking if request path matches the mock file name
    if !current_mock_absolute_path
        .as_ref()
        .to_string_lossy()
        .to_string()
        .ends_with(&file_name_probe)
    {
        return None;
    }

    let Some((current_mock_file_name_without_extension, _)) =
        current_mock_file_name.rsplit_once('.')
    else {
        return None;
    };

    let entry_name = format!(
        "{}.{}",
        current_mock_file_name_without_extension, current_mock_extension_lower
    );

    let config_path = current_mock_absolute_path
        .as_ref()
        .to_path_buf()
        .with_last_removed()
        .join(CONFIG_FILE_NAME);

    return Some((config_path, entry_name));
}

static NOT_FOUND: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::NotFound.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::not_found()
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

static GET_MOCK_CONFIG_FAILED: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    let body = Err::<&str, String>(MockersErrors::InternalServerError.to_string())
        .to_protocol()
        .to_string()
        .into();
    Response::internal_server_error()
        .content_type_json()
        .body(body)
        .unwrap_or_default()
});

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_happy_pass() {
        let current_mock_absolute_path = Path::new("/mocks/foo/bar.get");
        let request_params = MockInfo::new("GET".to_string(), "/foo/bar".to_string());
        let expected = Some((
            Path::new("/mocks/foo/config.json").to_path_buf(),
            "bar.get".to_string(),
        ));
        let actual = check_match(&current_mock_absolute_path, &request_params);
        assert_eq!(
            &actual, &expected,
            "{:#?} is not equal to {:#?}",
            &actual, &expected
        );
    }

    #[test]
    fn test_check_match_not_found() {
        let current_mock_absolute_path = Path::new("/mocks/foo/bar.get");
        let request_params = MockInfo::new("GET".to_string(), "/foo/baz".to_string());
        let actual = check_match(&current_mock_absolute_path, &request_params);
        assert!(actual.is_none(), "Expected None, got {:#?}", &actual);
    }

    #[test]
    fn extension_uppercase() {
        let current_mock_absolute_path = Path::new("/mocks/foo/bar.GeT");
        let request_params = MockInfo::new("GeT".to_string(), "/foo/bar".to_string());
        let actual = check_match(&current_mock_absolute_path, &request_params);
        assert!(actual.is_none(), "Expected None, got {:#?}", &actual);
    }

    #[test]
    fn path_uppercase() {
        let current_mock_absolute_path = Path::new("/mocks/foo/Bar.get");
        let request_params = MockInfo::new("GET".to_string(), "/foo/Bar".to_string());
        let actual = check_match(&current_mock_absolute_path, &request_params);
        let expected = Some((
            Path::new("/mocks/foo/config.json").to_path_buf(),
            "Bar.get".to_string(),
        ));
        assert_eq!(
            &actual, &expected,
            "{:#?} is not equal to {:#?}",
            &actual, &expected
        );
    }
}
