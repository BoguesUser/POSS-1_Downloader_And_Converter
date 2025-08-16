use std::path::Path;
use tokio::io::AsyncWriteExt;
use reqwest::StatusCode;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::stream::StreamExt;

// src/downloader.rs

// Using smol for async operations.
// For actual implementation, consider adding dependencies like `reqwest` for downloading
// and `smol` for async runtime

/// Downloads a file from a given URL and saves it to a specified path.
///
/// # Arguments
///
/// * `url` - The URL of the file to download.
/// * `dest_path` - The local path where the file should be saved.
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub async fn download_file(url: &str, dest_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(dest_path);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Check if the destination file already exists using tokio::fs
    match tokio::fs::metadata(dest_path).await {
        Ok(metadata) => {
            if metadata.is_file() {
                println!("File already exists at {}. Skipping download.", dest_path);
                return Ok(());
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File does not exist, proceed with download
        }
        Err(e) => {
            // Other error checking metadata
            eprintln!("Error checking file metadata for {}: {}", dest_path, e);
            return Err(Box::new(e));
        }
    }

    let response = reqwest::get(url).await?;

    if response.status() == StatusCode::NOT_FOUND {
        return Err(format!("File not found at URL: {}", url).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", url));

    let mut file = tokio::fs::File::create(dest_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message(format!("Downloaded {}", url));

    Ok(())
}