use std::{path::PathBuf, sync::Arc, time::Duration};

use crate::libs::absolute_mocks_path::AbsoluteMocksPath;
use crate::server::frontmatter_parser::frontmatter_parser::FrontmatterParser;
use crate::server::frontmatter_parser::mockers_frontmatter::MockersPromptParams;
use crate::libs::header_map_ext::{HeaderMapConverter, HeaderMapSanitizer};
use crate::libs::hyper_response_ext::HyperWellKnownResponses;
use crate::server::llm::client::LLMClient;
use crate::libs::mock_config::MockConfig;
use crate::libs::mockers_request_ext::{MockersRequestExt, Prompt};
use crate::libs::reqwest_response_ext::ReqwestResponseExt;
use crate::libs::response_builder_ext::ResponseBuilderExt;
use crate::libs::tap::Tap;
use crate::libs::url_ext::UrlExt;
use crate::libs::{cache_mode::CacheMode, get_mime::get_mime};
use crate::server::mockers_context::MockersContext;
use crate::server::params::CONFIG_FILE_NAME;
use crate::server::params::DEFAULT_CORS_ENABLED;
use crate::server::params::DEFAULT_MOCKS_RESPONSE_DELAY;
use crate::server::params::HARD_MAX_PROXY_RESPONSE_BODY_BYTES;
use crate::server::route_error::MockersRouteError;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, StatusCode, body::Bytes};
use reqwest::Url;
use routerify_ng::ext::RequestExt;
use tokio::join;

pub async fn mock_handler(
    req: Request<Full<Bytes>>,
) -> Result<Response<Full<Bytes>>, MockersRouteError> {
    let context = req.data::<Arc<MockersContext>>();
    let cors = match context.map(|context| &context.server_params) {
        None => DEFAULT_CORS_ENABLED,
        Some(sp) => sp.cors().await,
    };
    let preflight = match context.map(|ctx| &ctx.server_params) {
        None => None,
        Some(sp) => sp.preflight().await,
    };
    let global_delay_ms = match context.map(|ctx| &ctx.server_params) {
        None => DEFAULT_MOCKS_RESPONSE_DELAY,
        Some(sp) => sp.delay_ms().await,
    };
    let absolute_mocks_dir = match context.map(|ctx| &ctx.server_params) {
        None => None,
        Some(sp) => sp.get_absolute_mocks_path().await,
    };
    let origin = match context.map(|ctx| &ctx.server_params) {
        None => None,
        Some(sp) => sp.origin().await,
    };
    let proxy_max_body_bytes = match context.map(|ctx| &ctx.server_params) {
        None => None,
        Some(sp) => Some(sp.proxy_body_max_bytes().await),
    }
    .unwrap_or(HARD_MAX_PROXY_RESPONSE_BODY_BYTES);

    let client = context.map(|context| context.clone().client.clone());

    let llm_client = context.map(|context| context.clone().llm_client.clone());

    let method = req.method().to_string().to_lowercase();
    let uri_pathname = req.uri().path().to_string();

    if absolute_mocks_dir.is_none() {
        let response = Response::forbidden().empty_body().unwrap_or_default();
        return Ok(response);
    }
    /*
     * Example:
     * `/home/username/.cargo/bin/mocks`
     */
    let absolute_mocks_dir = absolute_mocks_dir.expect("absolute_mocks_dir is missing");
    /*
     * Example:
     * `/foo/bar/baz.get`
     */
    let relative_mock_file_name = format!("{}.{}", uri_pathname[1..].to_string(), method);
    /*
     * Example:
     * `/home/username/.cargo/bin/mocks/foo/bar/baz.get`
     */
    let absolute_mock_file_name = absolute_mocks_dir.join(&relative_mock_file_name);
    /*
     * Example:
     * `baz.get`
     */
    let mock_config_entry_name = relative_mock_file_name
        .split("/")
        .last()
        .unwrap_or("")
        .to_string();
    /*
     * Example:
     * `/foo/bar/`
     */
    let relative_current_mock_config_dir = {
        let mut pathbuf = PathBuf::from(uri_pathname[1..].to_string());
        pathbuf.pop();
        pathbuf
    };
    /*
     * Example:
     * `/home/username/.cargo/bin/mocks/foo/baz/config.json`
     */
    let config_file_path = absolute_mocks_dir
        .join(relative_current_mock_config_dir)
        .join(CONFIG_FILE_NAME);
    let config = MockConfig::try_read_from_file(&config_file_path).await.ok();

    let delay = config
        .as_ref()
        .and_then(|c| c.get(&mock_config_entry_name).and_then(|x| x.delay_ms()))
        .unwrap_or(global_delay_ms);
    let custom_headers = config
        .as_ref()
        .and_then(|c| {
            c.get(&mock_config_entry_name)
                .and_then(|x| x.headers().cloned())
        })
        .unwrap_or_default();
    let status_if_file_found = config
        .as_ref()
        .and_then(|c| c.get(&mock_config_entry_name).and_then(|x| x.status_code()))
        .map(|status_code| StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK))
        .unwrap_or(StatusCode::OK);
    let cache_mode = config
        .as_ref()
        .and_then(|c| c.get(&mock_config_entry_name))
        .and_then(|x| x.cache_mode())
        .unwrap_or(CacheMode::NoCache);
    let is_disabled_by_config = config
        .as_ref()
        .and_then(|c| c.get(&mock_config_entry_name))
        .map(|x| x.is_disabled())
        .unwrap_or(false);

    if is_disabled_by_config {
        let response = Response::not_found()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();

        return Ok(response);
    }

    let handle_preflight = match (preflight, req.is_preflight()) {
        (Some(preflight), true) => Some(preflight),
        _ => None,
    };

    // Check if this request is a special "preflight" browser request and user enforced handing It
    if let Some(handle_preflight) = handle_preflight {
        let response = Response::no_content()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(req.preflight_response_headers(handle_preflight).into_iter())
            .empty_body()
            .unwrap_or_default();
        return Ok(response);
    }

    let mock_bytes_result = tokio::fs::read(&absolute_mock_file_name).await;
    let mock_str = mock_bytes_result
        .as_ref()
        .ok()
        .and_then(|v| std::str::from_utf8(v).ok());

    // Try treat mock as LLM prompt
    if let Some(data) = mock_str {
        let frontmatter_parser = FrontmatterParser::from(data);
        let frontmatter_params = frontmatter_parser.frontmatter().and_then(|f| f.mockers());
        let suffix = req.to_markdown().await;
        let llm_response = match llm_client.zip(frontmatter_params) {
            Some((client, params)) => {
                client
                    .as_ref()
                    .process_prompt(params, data, suffix.as_str())
                    .await
            }
            None => None,
        };
        if let Some(Ok(llm_response)) = llm_response {
            let mime = get_mime(llm_response.as_bytes()).await;
            let response = Response::builder()
                .status(status_if_file_found)
                .add_content_type_header(&mime)
                .tap(|builder| if cors { builder.add_cors() } else { builder })
                .add_custom_headers(custom_headers.into_iter())
                .body(llm_response.into())
                .unwrap_or_default();
            return Ok(response);
        }
        if let Some(Err(_)) = llm_response {
            let response = Response::bad_gateway()
                .tap(|builder| if cors { builder.add_cors() } else { builder })
                .add_custom_headers(custom_headers.into_iter())
                .empty_body()
                .unwrap_or_default();
            return Ok(response);
        }
    }

    // Try respond from file-based mock
    if let Ok(data) = mock_bytes_result {
        let mime = get_mime(data.as_slice()).await;
        let response = Response::builder()
            .status(status_if_file_found)
            .add_content_type_header(&mime)
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .body(data.into())
            .unwrap_or_default();

        tokio::time::sleep(Duration::from_millis(delay)).await;

        return Ok(response);
    }

    // return 404 if no remote server specified
    let Some((origin, client)) = origin.zip(client) else {
        let response = Response::not_found()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();

        return Ok(response);
    };

    // make request if remote server is specified
    let Ok(target_url) = Url::from_parts(&origin, &uri_pathname, req.uri().query()) else {
        let response = Response::bad_request()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();

        return Ok(response);
    };

    let request_headers = req
        .headers()
        .to_owned()
        .remove_host_header()
        .remove_content_length_header();

    let request_builder = client
        .request(req.method().to_owned(), target_url)
        .headers(request_headers)
        .body(reqwest::Body::wrap_stream(
            req.body().clone().into_data_stream(),
        ));

    // Send response if remote server is specified
    let Ok(response) = request_builder.send().await else {
        let response = Response::bad_gateway()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();
        return Ok(response);
    };

    let resp_status = response.status().clone();

    let resp_headers = response.headers().clone();
    let save_headers = resp_headers.clone();
    let response_content_length = response.content_length();

    if let Some(response_content_length) = response_content_length
        && response_content_length > proxy_max_body_bytes as u64
    {
        let response = Response::bad_gateway()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();
        return Ok(response);
    }

    let response_bytes = response
        .read_until_cap(
            proxy_max_body_bytes,
            response_content_length.map(|l| l as usize),
        )
        .await;

    let Ok(response_bytes) = response_bytes else {
        let response = Response::bad_gateway()
            .tap(|builder| if cors { builder.add_cors() } else { builder })
            .add_custom_headers(custom_headers.into_iter())
            .empty_body()
            .unwrap_or_default();
        return Ok(response);
    };

    let save_bytes = response_bytes.clone();

    let response = Response::builder()
        .status(resp_status)
        .add_custom_headers(resp_headers.kv_iter())
        .body(response_bytes.into())
        .unwrap_or_default();

    if cache_mode == CacheMode::Overwrite {
        tokio::spawn(async move {
            let mock_config = MockConfig::new()
                .with_delay_ms(0)
                .with_cache_mode(CacheMode::Overwrite)
                .with_headers(save_headers.to_hash_map())
                .with_status_code(resp_status.into())
                .with_disabled_status(false);

            let write_config_fut =
                mock_config.try_write_to_file(&config_file_path, mock_config_entry_name);
            let write_mock_fut = tokio::fs::write(absolute_mock_file_name, &save_bytes);

            // TODO unused, should be logged maybe
            let _ = join!(write_config_fut, write_mock_fut);
        });
    }

    Ok(response)
}

trait AskLLM {
    async fn process_prompt(
        &self,
        frontmatter: &MockersPromptParams,
        prompt: &str,
        suffix: &str,
    ) -> Option<anyhow::Result<String>>;
}

impl AskLLM for &LLMClient {
    async fn process_prompt(
        &self,
        frontmatter: &MockersPromptParams,
        prompt: &str,
        suffix: &str,
    ) -> Option<anyhow::Result<String>> {
        let treat_as_prompt = frontmatter.prompt().unwrap_or(false);
        if !treat_as_prompt {
            return None;
        }
        Some(self.ask(frontmatter, prompt, suffix).await)
    }
}
