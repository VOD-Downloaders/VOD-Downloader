use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant, sleep_until};
use url::Url;

use crate::config;
use crate::request;
use crate::search::streams::Stream;

use super::DownloadStatus;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Error)]
pub enum DownloadError {
    #[error("Failed to open output file \"{file}\" with error: {error}", file = file.display())]
    FailedToOpenOutputFile { file: PathBuf, error: String },
    #[error("Request error: {0}")]
    RequestFailed(#[from] request::RequestError),
    #[error("Failed to write bytes to disk due to error: {0}")]
    FailedToWriteBytes(String),
}

/////////////////////////////////////////////////////
// Downloader
/////////////////////////////////////////////////////
pub async fn download_stream(
    indexer: &config::Indexer, stream: Stream, requester: &request::Requester, output_file: &Path, status: &RwLock<DownloadStatus>,
) -> Result<(), DownloadError> {
    trace!("Downloading stream of quality: {}.", stream.quality);

    // Open output
    trace!("Opening file \"{}\" for writing...", output_file.display());

    let mut file = OpenOptions::new().create(true).append(true).open(output_file).await.map_err(|error| {
        trace!("Failed to open \"{}\", error: {:?}, source: {:?}", output_file.display(), error, error.source());

        DownloadError::FailedToOpenOutputFile {
            file: output_file.to_path_buf(),
            error: error.to_string(),
        }
    })?;

    trace!("File \"{}\" successfully opened.", output_file.display());

    // Download based of of stream_type
    let result = download_segments(indexer, stream.video_segments, requester, &mut file, status).await;

    // Set status
    match result.clone() {
        Ok(_) => {
            trace!("Download finished successfully.");
            *status.write().await = DownloadStatus::Finished;
        },
        Err(error) => {
            trace!("Download finished unsuccessfully, error: {}.", error);
            *status.write().await = DownloadStatus::Failed(error.to_string());
        },
    }

    result
}

async fn download_segments(
    indexer: &config::Indexer, segments: Vec<Url>, requester: &request::Requester, output_file: &mut File, status: &RwLock<DownloadStatus>,
) -> Result<(), DownloadError> {
    trace!("Starting segments download...");

    // Per-second request throttle
    let throttle = {
        if matches!(indexer.download.order, config::DownloadOrder::Sequential)
            && let Some(max_requests) = indexer.download.max_requests
        {
            Some(Duration::from_secs_f64(1.0 / max_requests as f64))
        } else {
            None
        }
    };
    let mut next_allowed = Instant::now();

    for (i, segment) in segments.iter().enumerate() {
        trace!("Downloading segment #{} from: \"{}\"...", i, segment);

        // Set status
        *status.write().await = DownloadStatus::DownloadingSegments {
            amount: i as u32,
            total: segments.len() as u32,
        };

        let mut last_error: Option<DownloadError> = None;
        for attempt in 1..=indexer.download.segment_download.segment_attempts {
            // Throttle requests
            if let Some(gap) = throttle {
                sleep_until(next_allowed).await;
                next_allowed = Instant::now() + gap;
            }

            // Download
            match download_segment(indexer, segment, requester, output_file).await {
                Ok(_) => {
                    last_error = None;
                    break;
                },
                Err(error) => {
                    warning!(
                        "[Attempt {}/{}] For segment \"{}\" failed with error: {}.",
                        attempt,
                        indexer.download.segment_download.segment_attempts,
                        segment.as_str(),
                        error
                    );

                    last_error = Some(error);
                },
            }
        }

        if let Some(error) = last_error {
            return Err(error);
        }
    }

    Ok(())
}

async fn download_segment(indexer: &config::Indexer, url: &Url, requester: &request::Requester, output_file: &mut File) -> Result<(), DownloadError> {
    let contents = requester
        .get_bytes(url, Some(indexer.download.segment_download.headers.clone()))
        .await
        .map_err(DownloadError::RequestFailed)?;

    if contents.len()
        <= (indexer.download.segment_post_download.remove_front_bytes + indexer.download.segment_post_download.remove_back_bytes) as usize
    {
        return Err(DownloadError::FailedToWriteBytes(
            "Downloaded amount of bytes is less than the amount specified in postprocessing arguments.".to_string(),
        ));
    }

    let clean_bytes = &contents[indexer.download.segment_post_download.remove_front_bytes as usize
        ..(contents.len() - indexer.download.segment_post_download.remove_back_bytes as usize)];

    output_file
        .write_all(clean_bytes)
        .await
        .map_err(|error| DownloadError::FailedToWriteBytes(error.to_string()))?;

    Ok(())
}
