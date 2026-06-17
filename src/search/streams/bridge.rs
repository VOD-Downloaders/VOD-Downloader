use std::collections::HashMap;

use thiserror::Error;
use url::Url;
use serde::{Serialize};

use crate::config;
use crate::request;
use super::Streams;

/////////////////////////////////////////////////////
// SearchError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Failed to send request to bridge due to error: {0}")]
    RequestError(#[from] request::RequestError),
    #[error("Failed parse streams due to error: {0}")]
    FailedToParse(#[from] serde_json::Error),
}

/////////////////////////////////////////////////////
// BridgeSearchParameters
/////////////////////////////////////////////////////
#[derive(Debug, Serialize)]
pub struct BridgeSearchParameters {
    pub name: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    #[serde(rename = "tmdbId")]
    pub tmdb_id: u32,
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
    #[serde(rename = "serverUrl")]
    pub server_url: Url,
    pub emulate_url: Url,
    pub headers: HashMap<String, String>, // TODO: Make sure its encoded as headers[Referer]=https://example.com/}
}

/////////////////////////////////////////////////////
// Bridge interface
/////////////////////////////////////////////////////
pub async fn bridge_search_movie_streams(
    indexer: &config::Indexer, name: &str, tmdb_id: u32, imdb_id: String, bridge_url: &Url, requester: &request::Requester,
) -> Result<Streams, SearchError> {
    let mut url = bridge_url.clone().join(format!("/api/{}/movie", indexer.based_on).as_str()).unwrap();
    let headers = {
        let mut headers = indexer.search.headers.clone();
        if let Some(server_headers) = &indexer.server.headers {
            // Make the serverside headers override the global headers
            for (key, value) in server_headers {
                headers.insert(key.clone(), value.clone());
            }
        }
        headers
    };

    let parameters = serde_url_params::to_string(&BridgeSearchParameters {
        name: name.to_string(),
        season: None,
        episode: None,
        tmdb_id: tmdb_id,
        imdb_id: imdb_id,
        server_url: indexer.server.search_url.clone(),
        emulate_url: indexer.search.emulate_url.clone(),
        headers: headers,
    })
    .unwrap();
    url.set_query(Some(parameters.as_str()));

    let json_str = requester.get_string(&url, None).await?;
    let streams: Streams = serde_json::from_str(json_str.as_str())?;

    Ok(streams)
}

pub async fn bridge_search_episode_streams(
    indexer: &config::Indexer, name: &str, tmdb_id: u32, imdb_id: String, season: u32, episode: u32, bridge_url: &Url, requester: &request::Requester,
) -> Result<Streams, SearchError> {
    let mut url = bridge_url.clone().join(format!("/api/{}/series", indexer.based_on).as_str()).unwrap();
    let headers = {
        let mut headers = indexer.search.headers.clone();
        if let Some(server_headers) = &indexer.server.headers {
            // Make the serverside headers override the global headers
            for (key, value) in server_headers {
                headers.insert(key.clone(), value.clone());
            }
        }
        headers
    };

    let parameters = serde_url_params::to_string(&BridgeSearchParameters {
        name: name.to_string(),
        season: Some(season),
        episode: Some(episode),
        tmdb_id: tmdb_id,
        imdb_id: imdb_id,
        server_url: indexer.server.search_url.clone(),
        emulate_url: indexer.search.emulate_url.clone(),
        headers: headers,
    })
    .unwrap();
    url.set_query(Some(parameters.as_str()));

    let json_str = requester.get_string(&url, None).await?;
    let streams: Streams = serde_json::from_str(json_str.as_str())?;

    Ok(streams)
}
