use std::fs::{File, create_dir_all};
use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::libs::create_params::CreateParams;

pub async fn create_mock(
    params: CreateParams,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let verbose = params.verbose;
    let mocks_absolute_path = params.get_absolute_mocks_path().inspect_err(|_| {
        if verbose {
            println!("Failed to get mocks path")
        }
    })?;
    let mut destination_directory = mocks_absolute_path.join(params.route);
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
                    params.method.to_lowercase()
                )
            ),
        )));
    }
    create_dir_all(&destination_directory).inspect_err(|_| {
        if verbose {
            println!(
                "Failed to create dir '{}'",
                &destination_directory.display()
            )
        }
    })?;

    let last_path_part = last_path_part.unwrap();
    let file_name = format!("{}.{}", last_path_part, params.method.to_lowercase());
    let full_destination_file_path = destination_directory.join(file_name);
    let mut file = File::create(&full_destination_file_path).inspect_err(|_| {
        if verbose {
            println!(
                "Failed to create file: '{}'",
                &full_destination_file_path.display()
            )
        }
    })?;
    file.write(
        serde_json::to_string(&DefaultMockContents {
            hello: "world".to_string(),
        })
        .unwrap_or("".to_string())
        .as_bytes(),
    )
    .inspect_err(|_| {
        if verbose {
            println!(
                "Failed to write file contents to file: '{}'",
                &full_destination_file_path.display()
            )
        }
    })?;
    file.flush().inspect_err(|_| {
        if verbose {
            println!(
                "Failed to create file: '{}'",
                &full_destination_file_path.display()
            )
        }
    })?;

    return Ok(());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DefaultMockContents {
    hello: String,
}
