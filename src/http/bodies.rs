use serde::{Deserialize, Serialize};
use axum::{
    response::{self, IntoResponse},
    http::StatusCode,
};

use crate::config::Indexer;
use crate::config::IndexerSpecification;
use crate::search::streams::Stream;
use crate::search::streams::Streams;
use crate::search::info::*;

/////////////////////////////////////////////////////
// Requests
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct CreateIndexerRequest {
    pub indexer: Indexer,
}

#[derive(Debug, Deserialize)]
pub struct DeleteIndexerRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct StartDownloadRequest {
    pub stream: Stream,
    pub output_file: String,
}

/////////////////////////////////////////////////////
// Queries
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize)]
pub struct SearchMovieQuery {
    pub name: String,
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct SearchSeriesQuery {
    pub name: String,
    pub page: i32,
}

/////////////////////////////////////////////////////
// Paths
/////////////////////////////////////////////////////
#[derive(Deserialize)]
pub struct DownloadStatusPath {
    pub id: u32,
}

/////////////////////////////////////////////////////
// Responses
/////////////////////////////////////////////////////
#[derive(Serialize)]
pub struct ErrorResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub error: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct IndexersResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub indexers: Vec<Indexer>,
}

impl IntoResponse for IndexersResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct CreateIndexerResponse {
    #[serde(skip)]
    pub status: StatusCode,
}

impl IntoResponse for CreateIndexerResponse {
    fn into_response(self) -> response::Response {
        (self.status).into_response()
    }
}

#[derive(Serialize)]
pub struct DeleteIndexerResponse {
    #[serde(skip)]
    pub status: StatusCode,
}

impl IntoResponse for DeleteIndexerResponse {
    fn into_response(self) -> response::Response {
        (self.status).into_response()
    }
}

#[derive(Serialize)]
pub struct IndexerSpecificationsResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub indexers: Vec<IndexerSpecification>,
}

impl IntoResponse for IndexerSpecificationsResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}

#[derive(Serialize)]
pub struct SearchMovieResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: MoviePageResultBody,
}

impl IntoResponse for SearchMovieResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct SearchSeriesResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: SeriesPageResultBody,
}

impl IntoResponse for SearchSeriesResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct GetMovieResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: FullMovieBody,
}

impl IntoResponse for GetMovieResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct GetSeriesResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: FullSeriesBody,
}

impl IntoResponse for GetSeriesResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct GetSeasonResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: FullSeasonBody,
}

impl IntoResponse for GetSeasonResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct GetEpisodeResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: FullEpisodeBody,
}

impl IntoResponse for GetEpisodeResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct GetMovieExternalIDsResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: MovieExternalIDsBody,
}

impl IntoResponse for GetMovieExternalIDsResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct GetSeriesExternalIDsResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: SeriesExternalIDsBody,
}

impl IntoResponse for GetSeriesExternalIDsResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct GetSeasonExternalIDsResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: SeasonExternalIDsBody,
}

impl IntoResponse for GetSeasonExternalIDsResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct GetEpisodeExternalIDsResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub response: EpisodeExternalIDsBody,
}

impl IntoResponse for GetEpisodeExternalIDsResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.response)).into_response()
    }
}

#[derive(Serialize)]
pub struct StreamsResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub streams: Streams,
}

impl IntoResponse for StreamsResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self.streams)).into_response()
    }
}

#[derive(Serialize)]
pub struct StartDownloadResponse {
    #[serde(skip)]
    pub status: StatusCode,
    pub id: u32,
}

impl IntoResponse for StartDownloadResponse {
    fn into_response(self) -> response::Response {
        (self.status, response::Json(self)).into_response()
    }
}
