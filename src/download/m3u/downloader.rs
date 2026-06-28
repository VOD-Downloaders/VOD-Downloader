use url::Url;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant, sleep_until};

use crate::config;
use crate::request;
use crate::download::DownloadError;
use crate::download::DownloadStatus;

/////////////////////////////////////////////////////
// Download
/////////////////////////////////////////////////////
pub async fn download_segments(
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
