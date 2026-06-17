use url::Url;

use crate::config;
use crate::request::HeaderMapExt;
use crate::search::streams::Stream;
use crate::request::Requester;
use crate::streams::ConversionError;
use crate::streams::DownloadableStream;
use crate::streams::DownloadableStreamType;
use crate::streams::M3UResult;
use crate::streams::parse_m3u_contents;

/////////////////////////////////////////////////////
// Converter
/////////////////////////////////////////////////////
pub async fn create_stream(indexer: &config::Indexer, requester: &Requester, stream: Stream) -> Result<DownloadableStream, ConversionError> {
    let headers = {
        let mut headers = HeaderMapExt::new();
        for (key, value) in &indexer.stream.headers {
            headers.insert(reqwest::header::HeaderName::from_bytes(&key.clone().into_bytes()).unwrap(), value.parse().unwrap());
        }
        headers
    };

    let index_m3u_str = requester
        .get_string(&stream.url, Some(headers))
        .await
        .map_err(|error| ConversionError::UnableToRetrieveStream(stream.url.clone(), error))?;

    let parse_result = parse_m3u_contents(index_m3u_str.as_str()).map_err(|error| ConversionError::FailedToParseResult(error.to_string()))?;
    let M3UResult::Index(segments) = parse_result else {
        error!("Expected the M3U result to be an index.m3u(8) file, so unable to create stream.");
        return Err(ConversionError::FailedToParseResult("Expected the M3U result to be an index.m3u(8).".to_string()));
    };

    create_stream_from_segments(segments, &stream.url, stream.quality.as_str())
}

fn create_stream_from_segments(segments: Vec<String>, request_url: &Url, resolution: &str) -> Result<DownloadableStream, ConversionError> {
    let mut url_segments = Vec::with_capacity(segments.len());

    for segment in segments {
        let segment_url = Url::parse(segment.as_str());
        let full_url = {
            match segment_url {
                Ok(url) => url,
                Err(_error) => request_url.join(".").unwrap().join(segment.as_str()).unwrap(),
            }
        };
        url_segments.push(full_url);
    }

    Ok(DownloadableStream {
        resolution: resolution.to_string(),
        stream_type: DownloadableStreamType::Segments(url_segments),
    })
}
