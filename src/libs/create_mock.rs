use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use tokio::io::AsyncWriteExt;

use crate::libs::cache_mode::CacheMode;
use crate::libs::create_params::CreateParams;
use crate::libs::mock_config::{MockConfig, read_config};
use crate::server::params::CONFIG_FILE_NAME;

pub async fn create_mock(params: CreateParams) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let verbose = params.verbose();
    let mocks_absolute_path = params.get_absolute_mocks_path().inspect_err(|_| {
        if verbose {
            println!("Failed to get mocks path")
        }
    })?;
    let mut destination_directory = {
        let route = params.route();
        let route_as_path: PathBuf = params.route().into();
        if route_as_path.is_absolute() && route_as_path.starts_with("/") {
            mocks_absolute_path.join(route.trim_start_matches("/"))
        } else {
            mocks_absolute_path.join(&route)
        }
    };
    let last_path_part = destination_directory
        .iter()
        .last()
        .map(|p| p.to_string_lossy().to_string());
    let success = destination_directory.pop();
    if !success || last_path_part.is_none() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            format!(
                "Can't create mock \"{}\"",
                format!(
                    "{}.{}",
                    destination_directory.display(),
                    params.method().to_lowercase()
                )
            ),
        )));
    }
    tokio::fs::create_dir_all(&destination_directory)
        .await
        .inspect_err(|_| {
            if verbose {
                println!(
                    "Failed to create dir '{}'",
                    &destination_directory.display()
                )
            }
        })?;

    let last_path_part = last_path_part.unwrap_or_default();
    let file_name = format!("{}.{}", last_path_part, params.method().to_lowercase());
    let full_destination_file_path = destination_directory.join(&file_name);
    let full_destination_config_path = destination_directory.join(CONFIG_FILE_NAME);

    write_default_mock(
        &full_destination_file_path.display().to_string(),
        params.contents(),
        verbose,
    )
    .await?;
    write_default_config(
        &full_destination_config_path.display().to_string(),
        &file_name,
        &params,
    )
    .await?;

    return Ok(());
}

async fn write_default_config(
    absolute_config_path: &str,
    entry_name: &str,
    params: &CreateParams,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let get_default_config = || {
        MockConfig::new()
            .with_cache_mode(params.cache_mode().unwrap_or(CacheMode::NoCache))
            .with_delay_ms(params.delay_ms())
            .with_headers({
                let mut headers_map = HashMap::new();
                headers_map.insert("X-Server".to_string(), "Mockers".to_string());
                for (header_key, header_val) in params.headers() {
                    headers_map.insert(header_key.to_owned(), header_val.to_owned());
                }
                headers_map
            })
            .with_status_code(params.status_code())
    };
    let existing_config = read_config(absolute_config_path)
        .await
        .map(|mut config| {
            config.insert(entry_name.to_string(), get_default_config());
            config
        })
        .unwrap_or_else(|| {
            let mut new_config = HashMap::new();
            new_config.insert(entry_name.to_string(), get_default_config());
            new_config
        });

    let json_value = serde_json::to_value(&existing_config)?;
    let mut file = tokio::fs::File::create(&absolute_config_path).await?;
    file.write(serde_json::to_string_pretty(&json_value)?.as_bytes())
        .await?;
    file.flush().await?;

    Ok(())
}

async fn write_default_mock(
    absolute_path: &str,
    contents: Option<&str>,
    verbose: bool,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let mut file = tokio::fs::File::create(absolute_path)
        .await
        .inspect_err(|_| {
            if verbose {
                println!("Failed to create file: '{}'", absolute_path)
            }
        })?;

    let get_default_contents = || {
        serde_json::to_string(&DefaultMockContents {
            hello: "world".to_string(),
        })
        .unwrap_or("".to_string())
    };

    let contents_to_write = match contents {
        Some(v) => v,
        None => &get_default_contents(),
    };
    file.write(contents_to_write.as_bytes())
        .await
        .inspect_err(|_| {
            if verbose {
                println!("Failed to write file contents to file: '{}'", absolute_path)
            }
        })?;
    file.flush().await.inspect_err(|_| {
        if verbose {
            println!("Failed to create file: '{}'", absolute_path)
        }
    })?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DefaultMockContents {
    hello: String,
}
