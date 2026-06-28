use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc},
};

use axum::{
    extract,
    extract::{State, Query, Path},
    http::{StatusCode},
};
use chrono::Utc;
use tokio::sync::RwLock;

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
pub struct AppState {
    pub state: RwLock<config::State>,
    pub environment: env::EnvOptions, // readonly
    pub downloads: RwLock<HashMap<u32, Arc<download::DownloadInfo> /* readonly */>>,
    pub scheduler: download::DownloadScheduler,
}

impl AppState {
    pub fn new(environment: env::EnvOptions, state: config::State) -> Self {
        Self {
            state: RwLock::new(state),
            environment: environment,
            downloads: RwLock::new(HashMap::new()),
            scheduler: download::DownloadScheduler::new(),
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
        indexers: state.state.read().await.indexers.clone(),
    })
}

pub async fn post_create_indexer(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<CreateIndexerRequest>,
) -> Result<CreateIndexerResponse, ErrorResponse> {
    trace!("Received post_create_indexer for {:?}", payload);

    config::write_indexer_to_file(&payload.indexer, config::indexer_name_to_path(payload.indexer.name.as_str()).as_path())
        .await
        .map_err(|error| {
            error!("Unable to write indexer to file due to error: {}", error);
            ErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: format!("Unable to write indexer to file due to error: {}", error),
            }
        })?;

    state.state.write().await.refresh_indexers().await.map_err(|error| {
        error!("Failed to refresh indexers due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Failed to refresh indexers due to error: {}", error),
        }
    })?;

    Ok(CreateIndexerResponse { status: StatusCode::OK })
}

pub async fn post_update_indexer(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<UpdateIndexerRequest>,
) -> Result<UpdateIndexersResponse, ErrorResponse> {
    trace!("Received post_update_indexer for {:?}", payload);

    tokio::fs::remove_file(config::indexer_name_to_path(payload.old_name.as_str()).as_path())
        .await
        .map_err(|error| {
            error!("Unable to delete indexer \"{}\" due to error: {}", payload.old_name, error);
            ErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: format!("Unable to delete indexer \"{}\" due to error: {}", payload.old_name, error),
            }
        })?;

    config::write_indexer_to_file(&payload.indexer, config::indexer_name_to_path(payload.indexer.name.as_str()).as_path())
        .await
        .map_err(|error| {
            error!("Unable to write indexer to file due to error: {}", error);
            ErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: format!("Unable to write indexer to file due to error: {}", error),
            }
        })?;

    state.state.write().await.refresh_indexers().await.map_err(|error| {
        error!("Failed to refresh indexers due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Failed to refresh indexers due to error: {}", error),
        }
    })?;

    Ok(UpdateIndexersResponse { status: StatusCode::OK })
}

pub async fn post_delete_indexer(
    State(state): State<Arc<AppState>>, extract::Json(payload): extract::Json<DeleteIndexerRequest>,
) -> Result<DeleteIndexerResponse, ErrorResponse> {
    trace!("Received post_delete_indexer for \"{}\".", payload.name);

    tokio::fs::remove_file(config::indexer_name_to_path(payload.name.as_str()).as_path())
        .await
        .map_err(|error| {
            error!("Unable to delete indexer \"{}\" due to error: {}", payload.name, error);
            ErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: format!("Unable to delete indexer \"{}\" due to error: {}", payload.name, error),
            }
        })?;

    state.state.write().await.refresh_indexers().await.map_err(|error| {
        error!("Failed to refresh indexers due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Failed to refresh indexers due to error: {}", error),
        }
    })?;

    Ok(DeleteIndexerResponse { status: StatusCode::OK })
}

pub async fn post_refresh_indexers(State(state): State<Arc<AppState>>) -> Result<RefreshIndexersResponse, ErrorResponse> {
    trace!("Received post_refresh_indexers");

    state.state.write().await.refresh_indexers().await.map_err(|error| {
        error!("Failed to refresh indexers due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Failed to refresh indexers due to error: {}", error),
        }
    })?;

    Ok(RefreshIndexersResponse { status: StatusCode::OK })
}

pub async fn get_indexer_specifications(State(state): State<Arc<AppState>>) -> Result<IndexerSpecificationsResponse, ErrorResponse> {
    trace!("Received get_indexer_specifications");

    Ok(IndexerSpecificationsResponse {
        status: StatusCode::OK,
        indexers: state.state.read().await.indexer_specifications.clone(),
    })
}

pub async fn post_refresh_indexer_specifications(State(state): State<Arc<AppState>>) -> Result<RefreshIndexerSpecificationsResponse, ErrorResponse> {
    trace!("Received post_refresh_indexer_specifications");

    state.state.write().await.refresh_indexer_specifications().await.map_err(|error| {
        error!("Failed to refresh indexer specifications due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Failed to refresh indexer specifictions due to error: {}", error),
        }
    })?;

    Ok(RefreshIndexerSpecificationsResponse { status: StatusCode::OK })
}

pub async fn post_refetch_indexer_specifications(State(state): State<Arc<AppState>>) -> Result<RefetchIndexerSpecificationsResponse, ErrorResponse> {
    trace!("Received post_refetch_indexer_specifications");

    state.state.write().await.refetch_indexer_specifications().await.map_err(|error| {
        error!("Failed to refetch indexer specifications due to error: {}", error);
        ErrorResponse {
            status: StatusCode::BAD_GATEWAY,
            error: format!("Failed to refetch indexer specifictions due to error: {}", error),
        }
    })?;

    Ok(RefetchIndexerSpecificationsResponse { status: StatusCode::OK })
}

pub async fn get_search_movie(
    State(_state): State<Arc<AppState>>, Query(query): Query<SearchMovieQuery>,
) -> Result<SearchMovieResponse, ErrorResponse> {
    trace!("Received post_search_movie for \"{}\" on page {}.", query.name, query.page);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
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

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
    })?;

    let response = search::info::tmdb_search_series(query.name.as_str(), Some(query.page), &requester).await;

    Ok(SearchSeriesResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_movie(State(_state): State<Arc<AppState>>, Path(movie_id): Path<u32>) -> Result<GetMovieResponse, ErrorResponse> {
    trace!("Received get_movie for {}.", movie_id);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
    })?;

    let response = search::info::tmdb_get_movie(movie_id, &requester).await;

    Ok(GetMovieResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_series(State(_state): State<Arc<AppState>>, Path(series_id): Path<u32>) -> Result<GetSeriesResponse, ErrorResponse> {
    trace!("Received get_series for {}.", series_id);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
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

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
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

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
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

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
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

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
    })?;

    let response = search::info::tmdb_get_series_external_ids(series_id, &requester).await;

    Ok(GetSeriesExternalIDsResponse {
        status: StatusCode::OK,
        response: response,
    })
}

pub async fn get_streams_movie(
    State(state): State<Arc<AppState>>, Path((indexer_name, movie_id)): Path<(String, u32)>,
) -> Result<StreamsResponse, ErrorResponse> {
    trace!("Received get_streams_movie for movie {}", movie_id);

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
    })?;

    let indexer = {
        let guard = state.state.read().await;
        let Some(indexer) = guard.get_indexer_by_name(indexer_name.as_str()) else {
            error!("Failed to find indexer by name: \"{}\".", indexer_name);
            return Err(ErrorResponse {
                status: StatusCode::NOT_FOUND,
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
    .map_err(|error| {
        error!("Failed to find stream due to error: {}", error);
        ErrorResponse {
            status: StatusCode::BAD_GATEWAY,
            error: format!("Failed to find streams due to error: {}", error),
        }
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

    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
    })?;

    let indexer = {
        let guard = state.state.read().await;
        let Some(indexer) = guard.get_indexer_by_name(indexer_name.as_str()) else {
            error!("Failed to find indexer by name: \"{}\".", indexer_name);
            return Err(ErrorResponse {
                status: StatusCode::NOT_FOUND,
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
    .map_err(|error| {
        error!("Failed to find stream due to error: {}", error);
        ErrorResponse {
            status: StatusCode::BAD_GATEWAY,
            error: format!("Failed to find streams due to error: {}", error),
        }
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
        let guard = state.state.read().await;
        let Some(indexer) = guard.get_indexer_by_name(indexer_name.as_str()) else {
            error!("Indexer by name \"{}\" doesn't exist.", indexer_name);
            return Err(ErrorResponse {
                status: StatusCode::NOT_FOUND,
                error: format!("Indexer by name \"{}\" doesn't exist.", indexer_name),
            });
        };
        indexer.clone()
    };

    // TODO: Handle cloudflare
    let requester = request::Requester::get_curl(request::RequesterSpecification::default()).map_err(|error| {
        error!("Unable to create requester object due to error: {}", error);
        ErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: format!("Unable to create requester object due to error: {}", error),
        }
    })?;

    let output_file = PathBuf::from(OUTPUT_DIRECTORY).join(PathBuf::from(payload.output_file));
    let download_info = Arc::new(download::DownloadInfo::new(output_file.as_path()));
    let download_info_clone = Arc::clone(&download_info);
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        // Convert stream to downloadable stream
        *download_info_clone.status.write().await = download::DownloadStatus::Converting;
        let downloadable = match streams::convert_search_stream_to_downloadable(&indexer, &requester, payload.stream).await {
            Ok(downloadable) => downloadable,
            Err(error) => {
                error!("Failed to convert stream to a downloadable object, error: {}", error);
                *download_info_clone.status.write().await = download::DownloadStatus::Failed(error.to_string());
                *download_info_clone.end_time.write().await = Some(Utc::now());
                return;
            },
        };

        // Wait for a download slot for this host
        let host = downloadable.rate_limit_host();
        *download_info_clone.status.write().await = download::DownloadStatus::Queued;
        let _permit = state_clone.scheduler.acquire(host.as_str(), &indexer.download.order).await;

        // Start the actual download
        *download_info_clone.status.write().await = download::DownloadStatus::Starting;
        let download_result = download::download_stream(&indexer, downloadable, &requester, output_file.as_path(), &download_info_clone.status).await;
        *download_info_clone.end_time.write().await = Some(Utc::now());

        if let Err(error) = download_result {
            error!("Download failed due to error: {}", error);
        }

        // _permit drops here, letting the next queued download for this host proceed.
    });

    let id = rand::random::<u32>();
    trace!("Adding download by id {} to active downloads...", id);
    {
        let mut guard = state.downloads.write().await;
        guard.insert(id, download_info);
    }

    Ok(StartDownloadResponse {
        status: StatusCode::OK,
        id: id,
    })
}

pub async fn get_download_info(State(state): State<Arc<AppState>>, Path(id): Path<u32>) -> Result<DownloadInfoResponse, ErrorResponse> {
    trace!("Received get_download_info for download id {}", id);

    let guard = state.downloads.read().await;
    let Some(download_info) = guard.get(&id) else {
        error!("Trying to retrieve download information for download by id {}, id not found.", id);
        return Err(ErrorResponse {
            status: StatusCode::NOT_FOUND,
            error: format!("Trying to retrieve download information for download by id {}, id not found.", id),
        });
    };

    let start_time = download_info.start_time.timestamp() as u64;
    let end_time = download_info.end_time.read().await.map(|time| time.timestamp() as u64);

    Ok(DownloadInfoResponse {
        status: StatusCode::OK,
        start_time: start_time,
        end_time: end_time,
        output_file: download_info.output_file.clone(),
        download_status: download_info.status.read().await.clone(),
    })
}
