use std::collections::HashMap;
use crate::controllers::swagger::static_files::SWAGGER_JSON;
use crate::libs::absolute_mocks_path::AbsoluteMocksPath;
use crate::libs::create_params::DEFAULT_STATUS_CODE;
use crate::libs::fs_cached_reader::FSCachedReader;
use crate::libs::fs_walker::FSWalker;
use crate::libs::get_mime::get_mime;
use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::libs::mock_config::MockConfig;
use crate::libs::path_buf_ext::PathBufExt;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::server::mockers_context::MockersContext;
use crate::server::params::{CONFIG_FILE_NAME, ServerParams};
use crate::server::route_error::MockersRouteError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, Response};
use routerify_ng::ext::RequestExt;
use serde_json::json;
use std::sync::{Arc, LazyLock};

pub async fn get_swagger_json(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let Some(server_params) = (match req.data::<Arc<MockersContext>>() {
        None => None,
        Some(ctx) => Some(ctx.server_params.clone()),
    }) else {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    };

    let Ok(mut openapi_specs) = serde_json::from_str::<serde_json::Value>(SWAGGER_JSON) else {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    };

    if add_servers(&mut openapi_specs, &server_params)
        .await
        .is_err()
    {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    }

    if add_mocks(&mut openapi_specs, &server_params).await.is_err() {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    }

    let Ok(result_json) = serde_json::to_string_pretty(&openapi_specs) else {
        return Ok(OPENAPI_FULFILL_JSON_ERROR.clone());
    };

    let response = Response::ok()
        .content_type_json()
        .body(result_json.into())
        .unwrap_or_default();

    Ok(response)
}

async fn add_servers(
    specs: &mut serde_json::Value,
    server_params: &ServerParams,
) -> anyhow::Result<()> {
    let specs = specs
        .as_object_mut()
        .ok_or(anyhow::Error::msg("Invalid json structure"))?;
    let admin_base_url = server_params
        .admin_base_url()
        .await
        .ok_or(anyhow::Error::msg("Admin base URL not specified"))?;

    specs.insert("servers".to_string(), json!([{ "url": admin_base_url }]));

    Ok(())
}

async fn add_mocks(
    specs: &mut serde_json::Value,
    server_params: &ServerParams,
) -> anyhow::Result<()> {
    let specs = specs
        .as_object_mut()
        .map(|s| s.get_mut("paths"))
        .flatten()
        .map(|v| v.as_object_mut())
        .flatten()
        .ok_or(anyhow::Error::msg("Invalid json structure"))?;

    let absolute_mocks_path = server_params
        .get_absolute_mocks_path()
        .await
        .ok_or(anyhow::Error::msg("Mocks absolute path not specified"))?;

    let fs_cached_reader = FSCachedReader::default();
    let walker = FSWalker::new(&absolute_mocks_path);

    for (metadata, path) in walker.into_iter().await {
        if metadata.is_dir() || metadata.is_symlink() || !path.is_mock_file() {
            continue;
        }

        let mock_pathname = path
            .to_string_lossy()
            .to_string()
            .replace(&absolute_mocks_path.to_string_lossy().to_string(), "")
            .rsplit_once('.')
            .map(|(path, _)| path.to_string());
        let Some(mock_pathname) = mock_pathname else {
            continue;
        };

        let Some((file_name, http_method)) = path
            .file_name()
            .map(|f| {
                f.to_string_lossy()
                    .to_string()
                    .rsplit_once('.')
                    .map(|(a, b)| (a.to_string(), b.to_string()))
            })
            .flatten()
            .map(|(file_name, method)| (file_name.to_string(), method.to_lowercase()))
        else {
            continue;
        };
        let entry_name = format!("{file_name}.{http_method}");

        let full_config_path = path.with_last_removed().join(CONFIG_FILE_NAME);
        let mock_config = fs_cached_reader
            .read(full_config_path)
            .await
            .as_ref()
            .as_ref()
            .ok()
            .map(|v| String::from_utf8(v.clone()).ok())
            .flatten()
            .map(|v| MockConfig::try_from_str(v).ok())
            .flatten()
            .unwrap_or_default()
            .get(&entry_name)
            .cloned()
            .unwrap_or_default();

        if mock_config.is_disabled() {
            continue;
        }

        let contents = tokio::fs::read(&path).await.unwrap_or_default();

        let content_type_header_value = mock_config
            .headers()
            .map(|h| h.into_iter().map(|(k, v)| (k.to_lowercase(), v.to_string())).collect::<HashMap<String, String>>())
            .map(|h| h.get(&CONTENT_TYPE.to_string()).cloned())
            .flatten();
        let mime = match content_type_header_value {
            Some(ct) => ct.to_owned(),
            None => get_mime(&contents).await,
        };

        specs
            .entry(mock_pathname)
            .or_insert(json!({
                // Mocks are being served from the root URL
                "servers": [{ "url": "/" }]
            }))
            .as_object_mut()
            .ok_or(anyhow::Error::msg("Failed to insert"))?
            .entry(http_method)
            .or_insert(json!({ "tags": ["mocks"] }))
            .as_object_mut()
            .ok_or(anyhow::Error::msg("Failed to insert"))?
            .entry("responses")
            .or_insert(json!({}))
            .as_object_mut()
            .ok_or(anyhow::Error::msg("Failed to insert"))?
            .entry(
                mock_config
                    .status_code()
                    .unwrap_or(DEFAULT_STATUS_CODE)
                    .to_string(),
            )
            .or_insert(json!({}))
            .as_object_mut()
            .ok_or(anyhow::Error::msg("Failed to insert"))?
            .entry("content")
            .or_insert(json!({}))
            .as_object_mut()
            .ok_or(anyhow::Error::msg("Failed to insert"))?
            .entry(mime.as_str())
            .or_insert(json!({}))
            .as_object_mut()
            .ok_or(anyhow::Error::msg("Failed to insert"))?
            .entry("schema")
            .or_insert(json!({}));
    }

    Ok(())
}

static OPENAPI_FULFILL_JSON_ERROR: LazyLock<Response<Full<Bytes>>> = LazyLock::new(|| {
    Response::internal_server_error()
        .content_type_octet_stream()
        .body(Full::default())
        .unwrap_or_default()
});
