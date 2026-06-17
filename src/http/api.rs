use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use axum::{
    extract,
    extract::{State, Query, Path},
    http::{StatusCode},
};

use super::bodies::*;
use crate::{
    env,
    search::info::{tmdb_get_episode, tmdb_get_movie, tmdb_get_movie_external_ids, tmdb_get_series, tmdb_get_series_external_ids},
};
use crate::config;
use crate::request;
use crate::search;
use crate::download;
use crate::streams;

const OUTPUT_DIRECTORY: &str = "/output/";

/////////////////////////////////////////////////////
// State
/////////////////////////////////////////////////////
pub struct DownloadInfo {}

pub struct AppState {
    pub state: RwLock<config::State>,
    pub environment: env::EnvOptions, // readonly
    pub downloads: RwLock<HashMap<u32, DownloadInfo>>,
}

impl AppState {
    pub fn new(environment: env::EnvOptions, state: config::State) -> Self {
        Self {
            state: RwLock::new(state),
            environment: environment,
            downloads: RwLock::new(HashMap::new()),
        }
    }
}

/////////////////////////////////////////////////////
// API
/////////////////////////////////////////////////////
pub async fn get_indexers(State(state): State<Arc<AppState>>) -> Result<IndexersResponse, ErrorResponse> {
    trace!("Received get_indexers");

    Ok(IndexersResponse {
        status: StatusCode::OK,
        indexers: state.state.read().unwrap().indexers.clone(),
    })
}

pub async fn post_create_indexer(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<CreateIndexerRequest>,
) -> Result<CreateIndexerResponse, ErrorResponse> {
    trace!("Received post_create_indexer for {:?}", payload);

    config::write_indexer_to_file(&payload.indexer, config::indexer_name_to_path(payload.indexer.name.as_str()).as_path())
        .await
        .map_err(|error| ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            error: format!("Unable to write indexer to file due to error: {}", error),
        })?;

    // Update indexers in state
    state.state.write().unwrap().indexers = config::load_indexers().await;

    Ok(CreateIndexerResponse { status: StatusCode::OK })
}

pub async fn post_delete_indexer(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<DeleteIndexerRequest>,
) -> Result<DeleteIndexerResponse, ErrorResponse> {
    trace!("Received post_delete_indexer for \"{}\".", payload.name);

    tokio::fs::remove_file(config::indexer_name_to_path(payload.name.as_str()).as_path())
        .await
        .map_err(|error| ErrorResponse {
            status: StatusCode::BAD_REQUEST,
            error: format!("Unable to delete indexer \"{}\" due to error: {}", payload.name, error),
        })?;

    // Update indexers in state
    state.state.write().unwrap().indexers = config::load_indexers().await;

    Ok(DeleteIndexerResponse { status: StatusCode::OK })
}

pub async fn get_indexer_specifications(State(_state): State<Arc<AppState>>) -> Result<IndexerSpecificationsResponse, ErrorResponse> {
    trace!("Received get_indexer_specifications");

    Ok(IndexerSpecificationsResponse {
        status: StatusCode::OK,
        indexers: config::load_indexer_specifications().await,
    })
}

pub async fn post_refresh_indexer_specifications(State(_state): State<Arc<AppState>>) -> Result<IndexerSpecificationsResponse, ErrorResponse> {
    trace!("Received post_refresh_indexer_specifications");

    config::get_new_specifications().await.map_err(|error| ErrorResponse {
        status: StatusCode::BAD_GATEWAY,
        error: format!("Unable to retrieve latest indexer specifications due to error: {}", error),
    })?;

    Ok(IndexerSpecificationsResponse {
        status: StatusCode::OK,
        indexers: config::load_indexer_specifications().await,
    })
}

pub async fn get_search_movie(
    State(_state): State<Arc<AppState>>, Query(query): Query<SearchMovieQuery>,
) -> Result<SearchMovieResponse, ErrorResponse> {
    trace!("Received post_search_movie for \"{}\" on page {}.", query.name, query.page);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_search_movies(query.name.as_str(), Some(query.page), &requester).await;

    Ok(SearchMovieResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_search_series(
    State(_state): State<Arc<AppState>>, Query(query): Query<SearchSeriesQuery>,
) -> Result<SearchSeriesResponse, ErrorResponse> {
    trace!("Received post_search_series for \"{}\" on page {}.", query.name, query.page);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_search_series(query.name.as_str(), Some(query.page), &requester).await;

    Ok(SearchSeriesResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_movie(State(_state): State<Arc<AppState>>, Path(movie_id): Path<u32>) -> Result<GetMovieResponse, ErrorResponse> {
    trace!("Received get_movie for {}.", movie_id);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_get_movie(movie_id, &requester).await;

    Ok(GetMovieResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_series(State(_state): State<Arc<AppState>>, Path(series_id): Path<u32>) -> Result<GetSeriesResponse, ErrorResponse> {
    trace!("Received get_series for {}.", series_id);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_get_series(series_id, &requester).await;

    Ok(GetSeriesResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_season(
    State(_state): State<Arc<AppState>>, Path((series_id, season_number)): Path<(u32, u32)>,
) -> Result<GetSeasonResponse, ErrorResponse> {
    trace!("Received get_season for series {} season {}.", series_id, season_number);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_get_season(series_id, season_number, &requester).await;

    Ok(GetSeasonResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_episode(
    State(_state): State<Arc<AppState>>, Path((series_id, season_number, episode_number)): Path<(u32, u32, u32)>,
) -> Result<GetEpisodeResponse, ErrorResponse> {
    trace!("Received get_episode for series {} season {} episode {}.", series_id, season_number, episode_number);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_get_episode(series_id, season_number, episode_number, &requester).await;

    Ok(GetEpisodeResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_movie_external_ids(
    State(_state): State<Arc<AppState>>, Path(movie_id): Path<u32>,
) -> Result<GetMovieExternalIDsResponse, ErrorResponse> {
    trace!("Received get_movie_external_ids for {}.", movie_id);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_get_movie_external_ids(movie_id, &requester).await;

    Ok(GetMovieExternalIDsResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_series_external_ids(
    State(_state): State<Arc<AppState>>, Path(series_id): Path<u32>,
) -> Result<GetSeriesExternalIDsResponse, ErrorResponse> {
    trace!("Received get_series_external_ids for {}.", series_id);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_get_series_external_ids(series_id, &requester).await;

    Ok(GetSeriesExternalIDsResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_season_external_ids(
    State(_state): State<Arc<AppState>>, Path((series_id, season_number)): Path<(u32, u32)>,
) -> Result<GetSeasonExternalIDsResponse, ErrorResponse> {
    trace!("Received get_season_external_ids for series {} season {}.", series_id, season_number);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_get_season_external_ids(series_id, season_number, &requester).await;

    Ok(GetSeasonExternalIDsResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_episode_external_ids(
    State(_state): State<Arc<AppState>>, Path((series_id, season_number, episode_number)): Path<(u32, u32, u32)>,
) -> Result<GetEpisodeExternalIDsResponse, ErrorResponse> {
    trace!("Received get_episode_external_ids for series {} season {} episode {}.", series_id, season_number, episode_number);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let response = search::info::tmdb_get_episode_external_ids(series_id, season_number, episode_number, &requester).await;

    Ok(GetEpisodeExternalIDsResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_streams_movie(
    State(state): State<Arc<AppState>>, Path((indexer_name, movie_id)): Path<(String, u32)>,
) -> Result<StreamsResponse, ErrorResponse> {
    trace!("Received get_streams_movie for movie {}", movie_id);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let indexer = {
        let guard = state.state.read().unwrap();
        let Some(indexer) = guard.get_indexer_by_name(indexer_name.as_str()) else {
            return Err(ErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: format!("Failed to find indexer by name: \"{}\".", indexer_name),
            });
        };
        indexer.clone()
    };

    let movie_info = tmdb_get_movie(movie_id, &requester).await;
    let external_ids = tmdb_get_movie_external_ids(movie_id, &requester).await;

    let streams = search::streams::bridge_search_movie_streams(
        &indexer,
        movie_info.title.as_str(),
        movie_info.id,
        external_ids.imdb_id,
        &state.environment.bridge_url,
        &requester,
    )
    .await
    .map_err(|error| ErrorResponse {
        status: StatusCode::FAILED_DEPENDENCY,
        error: format!("Failed to find streams due to error: {}", error),
    })?;

    Ok(StreamsResponse {
        status: StatusCode::OK,
        streams: streams,
    })
}

pub async fn get_streams_episode(
    State(state): State<Arc<AppState>>, Path((indexer_name, series_id, season_number, episode_number)): Path<(String, u32, u32, u32)>,
) -> Result<StreamsResponse, ErrorResponse> {
    trace!("Received get_streams_series for series {} season {} episode {}.", series_id, season_number, episode_number);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let indexer = {
        let guard = state.state.read().unwrap();
        let Some(indexer) = guard.get_indexer_by_name(indexer_name.as_str()) else {
            return Err(ErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: format!("Failed to find indexer by name: \"{}\".", indexer_name),
            });
        };
        indexer.clone()
    };

    let series_info = tmdb_get_series(series_id, &requester).await;
    let episode_info = tmdb_get_episode(series_id, season_number, episode_number, &requester).await;
    let external_ids = tmdb_get_series_external_ids(series_id, &requester).await;

    let streams = search::streams::bridge_search_episode_streams(
        &indexer,
        episode_info.name.as_str(),
        series_info.id,
        external_ids.imdb_id,
        season_number,
        episode_number,
        &state.environment.bridge_url,
        &requester,
    )
    .await
    .map_err(|error| ErrorResponse {
        status: StatusCode::FAILED_DEPENDENCY,
        error: format!("Failed to find streams due to error: {}", error),
    })?;

    Ok(StreamsResponse {
        status: StatusCode::OK,
        streams: streams,
    })
}

pub async fn post_start_download(
    State(state): State<Arc<AppState>>, Path(indexer_name): Path<String>, extract::Json(payload): extract::Json<StartDownloadRequest>,
) -> Result<StartDownloadResponse, ErrorResponse> {
    trace!("Received post_start_download for \"{:?}\".", payload);

    let indexer = {
        let guard = state.state.read().unwrap();
        let Some(indexer) = guard.get_indexer_by_name(indexer_name.as_str()) else {
            return Err(ErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: format!("Indexer by name \"{}\" doesn't exist.", indexer_name),
            });
        };
        indexer.clone()
    };

    // TODO: Handle cloudflare
    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| ErrorResponse {
        status: StatusCode::PRECONDITION_FAILED,
        error: format!("Unable to create requester object due to error: {}", error),
    })?;

    let output_file = PathBuf::from(OUTPUT_DIRECTORY).join(PathBuf::from(payload.output_file));
    tokio::spawn(async move {
        let stream_result = streams::convert_search_stream_to_downloadable(&indexer, &requester, payload.stream).await;

        if let Err(error) = stream_result {
            error!("Failed to convert stream to a downloadable object, error: {}", error);
            return;
        }

        let download_result = download::download_stream(&indexer, stream_result.unwrap(), &requester, output_file.as_path()).await;

        if let Err(error) = download_result {
            error!("Download failed due to error: {}", error);
        }
    });

    let id = rand::random::<u32>();
    trace!("Adding download by id {} to active downloads...", id);
    {
        let mut guard = state.downloads.write().unwrap();
        guard.insert(id, DownloadInfo {});
    }

    Ok(StartDownloadResponse {
        status: StatusCode::OK,
        id: id,
    })
}
