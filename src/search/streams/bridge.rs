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
// SearchMovieParameters
/////////////////////////////////////////////////////
#[derive(Debug, Serialize)]
pub struct SearchMovieParameters {
    pub name: String,
    pub year: u16,
    #[serde(rename = "tmdbId")]
    pub tmdb_id: u32,
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
    #[serde(rename = "serverName")]
    pub server_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchEpisodeParameters {
    pub name: String,
    pub year: u16,
    pub season: u32,
    pub episode: u32,
    #[serde(rename = "tmdbId")]
    pub tmdb_id: u32,
    #[serde(rename = "imdbId")]
    pub imdb_id: String,
    #[serde(rename = "serverName")]
    pub server_name: Option<String>,
}

/////////////////////////////////////////////////////
// Bridge interface
/////////////////////////////////////////////////////
pub async fn bridge_search_movie_streams(
    indexer: &config::Indexer, name: &str, year: u16, tmdb_id: u32, imdb_id: String, bridge_url: &Url, requester: &request::Requester,
) -> Result<Streams, SearchError> {
    let mut url = bridge_url.clone().join(format!("/api/{}/movie", indexer.based_on).as_str()).unwrap();
    let parameters = serde_url_params::to_string(&SearchMovieParameters {
        name: name.to_string(),
        year: year,
        tmdb_id: tmdb_id,
        imdb_id: imdb_id,
        server_name: Some(indexer.server.clone()),
    })
    .unwrap();
    url.set_query(Some(parameters.as_str()));

    let json_str = requester.get_string(&url, None).await?;
    let streams: Streams = serde_json::from_str(json_str.as_str())?;

    Ok(streams)
}

pub async fn bridge_search_episode_streams(
    indexer: &config::Indexer, name: &str, year: u16, tmdb_id: u32, imdb_id: String, season: u32, episode: u32, bridge_url: &Url,
    requester: &request::Requester,
) -> Result<Streams, SearchError> {
    let mut url = bridge_url.clone().join(format!("/api/{}/series", indexer.based_on).as_str()).unwrap();
    let parameters = serde_url_params::to_string(&SearchEpisodeParameters {
        name: name.to_string(),
        year: year,
        season: season,
        episode: episode,
        tmdb_id: tmdb_id,
        imdb_id: imdb_id,
        server_name: Some(indexer.server.clone()),
    })
    .unwrap();
    url.set_query(Some(parameters.as_str()));

    let json_str = requester.get_string(&url, None).await?;
    let streams: Streams = serde_json::from_str(json_str.as_str())?;

    Ok(streams)
}
