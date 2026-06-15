use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;
use url::Url;
use tokio::fs::OpenOptions;

use crate::config;
use crate::download::m3u::M3UResult;
use crate::request;
use crate::request::HeaderMap;
use crate::search::streams::Stream;

use super::m3u;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Failed to get stream from \"{0}\" due to error: {1}")]
    FailedToGetStream(Url, request::RequestError),
    #[error("Failed to parse stream due to error: {0}")]
    FailedToParseStream(#[from] m3u::ParseError),
    #[error("M3U does not follow expectations: {0}")]
    InvalidM3U(String),
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
    indexer: &config::Indexer, stream: Stream, requester: &request::Requester, output_file: &Path,
) -> Result<(), DownloadError> {
    trace!("Downloading stream of resolution: {} from \"{}\".", stream.quality, stream.url);

    // Parse stream
    // TODO: Handle non-m3u
    let m3u = requester
        .get_string(&stream.url, Some(indexer.download.segment_download.headers.clone()))
        .await
        .map_err(|error| DownloadError::FailedToGetStream(stream.url.clone(), error))?;
    let result = m3u::parse_m3u_contents(m3u.as_str())?;
    let M3UResult::Index(segments) = result else {
        error!("Unable to download requested stream since the m3u is not a index.m3u(8) file.");
        return Err(DownloadError::InvalidM3U("The m3u file is not a index.m3u(8) file.".to_string()));
    };
    let segments: Vec<Url> = segments
        .iter()
        .map(|segment| Url::parse(segment))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_error| {
            error!("A url in the index.m3u(8) is not a valid url.");
            return DownloadError::InvalidM3U("One of the urls in the index.m3u(8) file is broken or does not follow the url standard.".to_string());
        })?;

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
    m3u::download_segments(indexer, segments, requester, &mut file).await
}
