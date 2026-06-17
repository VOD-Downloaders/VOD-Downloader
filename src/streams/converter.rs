use thiserror::Error;
use url::Url;

use super::m3u;
use super::DownloadableStream;
use crate::config;
use crate::search::streams::Stream;
use crate::request::Requester;
use crate::request::RequestError;

/////////////////////////////////////////////////////
// ConversionError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Unable to download stream file from \"{0}\".")]
    UnableToRetrieveStream(Url, RequestError),
    #[error("Failed to parse downloaded stream result, due to: {0}")]
    FailedToParseResult(String),
}

/////////////////////////////////////////////////////
// Converter
/////////////////////////////////////////////////////
pub async fn convert_search_stream_to_downloadable(
    indexer: &config::Indexer, requester: &Requester, stream: Stream,
) -> Result<DownloadableStream, ConversionError> {
    match indexer.stream.stream_type {
        config::StreamType::Index => m3u::create_stream(indexer, requester, stream).await,
        config::StreamType::Mp4 => {
            todo!()
        },
    }
}
