use url::Url;
use serde::{Serialize, Deserialize};

use super::SeriesPageResultBody;

use crate::search::info::*;
use crate::request;

/////////////////////////////////////////////////////
// Config
/////////////////////////////////////////////////////
const TMDB_API_URL: &str = "https://tmdb-proxy-chi-ivory.vercel.app";

/////////////////////////////////////////////////////
// Parameters
/////////////////////////////////////////////////////
#[derive(Serialize)]
struct MovieSearchParameters {
    pub query: String,
    pub include_adult: bool,
    pub page: i32,
}

impl Default for MovieSearchParameters {
    fn default() -> Self {
        Self {
            query: "".to_string(),
            include_adult: false,
            page: 1,
        }
    }
}

#[derive(Serialize)]
struct SeriesSearchParameters {
    pub query: String,
    pub include_adult: bool,
    pub page: i32,
}

impl Default for SeriesSearchParameters {
    fn default() -> Self {
        Self {
            query: "".to_string(),
            include_adult: false,
            page: 1,
        }
    }
}

/////////////////////////////////////////////////////
// TMDB interface // TODO: Return Errors
/////////////////////////////////////////////////////
pub async fn tmdb_run_api_call<T>(api_call: &Url, requester: &request::Requester) -> T
where
    T: serde::de::DeserializeOwned + Default + std::fmt::Debug,
{
    trace!("Requesting API results from: \"{}\".", api_call);
    let result = requester.get_string(api_call, None).await;
    let Ok(json_str) = result else {
        error!("Failed to retrieve API results from \"{}\", error: {}", api_call, result.unwrap_err());
        return T::default();
    };

    trace!("{}", json_str);

    let result = serde_json::from_str::<T>(json_str.as_str());
    let Ok(body) = result else {
        error!("Failed to convert API results response to json, error: {}.", result.unwrap_err());
        return T::default();
    };

    trace!("Got these API results: {:?}", body);

    body
}

pub async fn tmdb_search_movies(movie_name: &str, page: Option<i32>, requester: &request::Requester) -> MoviePageResultBody {
    let page = page.unwrap_or(1);
    let parameters = serde_url_params::to_string(&MovieSearchParameters {
        query: movie_name.to_string(),
        page: page,
        ..MovieSearchParameters::default()
    })
    .unwrap();
    let api_call = format!("{}/search/movie?{}", TMDB_API_URL, parameters);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<MoviePageResultBody>(&url, requester).await
}

pub async fn tmdb_search_series(series_name: &str, page: Option<i32>, requester: &request::Requester) -> SeriesPageResultBody {
    let page = page.unwrap_or(1);
    let parameters = serde_url_params::to_string(&SeriesSearchParameters {
        query: series_name.to_string(),
        page: page,
        ..SeriesSearchParameters::default()
    })
    .unwrap();
    let api_call = format!("{}/search/tv?{}", TMDB_API_URL, parameters);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<SeriesPageResultBody>(&url, requester).await
}

pub async fn tmdb_get_movie(movie_id: u32, requester: &request::Requester) -> FullMovieBody {
    let api_call = format!("{}/movie/{}", TMDB_API_URL, movie_id);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<FullMovieBody>(&url, requester).await
}

pub async fn tmdb_get_series(series_id: u32, requester: &request::Requester) -> FullSeriesBody {
    let api_call = format!("{}/tv/{}", TMDB_API_URL, series_id);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<FullSeriesBody>(&url, requester).await
}

pub async fn tmdb_get_season(series_id: u32, season_number: u32, requester: &request::Requester) -> FullSeasonBody {
    let api_call = format!("{}/tv/{}/season/{}", TMDB_API_URL, series_id, season_number);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<FullSeasonBody>(&url, requester).await
}

pub async fn tmdb_get_episode(series_id: u32, season_number: u32, episode_number: u32, requester: &request::Requester) -> FullEpisodeBody {
    let api_call = format!("{}/tv/{}/season/{}/episode/{}", TMDB_API_URL, series_id, season_number, episode_number);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<FullEpisodeBody>(&url, requester).await
}

pub async fn tmdb_get_movie_external_ids(movie_id: u32, requester: &request::Requester) -> MovieExternalIDsBody {
    let api_call = format!("{}/movie/{}/external_ids", TMDB_API_URL, movie_id);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<MovieExternalIDsBody>(&url, requester).await
}

pub async fn tmdb_get_series_external_ids(series_id: u32, requester: &request::Requester) -> SeriesExternalIDsBody {
    let api_call = format!("{}/tv/{}/external_ids", TMDB_API_URL, series_id);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<SeriesExternalIDsBody>(&url, requester).await
}

pub async fn tmdb_get_season_external_ids(series_id: u32, season_number: u32, requester: &request::Requester) -> SeasonExternalIDsBody {
    let api_call = format!("{}/tv/{}/season/{}/external_ids", TMDB_API_URL, series_id, season_number);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<SeasonExternalIDsBody>(&url, requester).await
}

pub async fn tmdb_get_episode_external_ids(
    series_id: u32, season_number: u32, episode_number: u32, requester: &request::Requester,
) -> EpisodeExternalIDsBody {
    let api_call = format!("{}/tv/{}/season/{}/episode/{}/external_ids", TMDB_API_URL, series_id, season_number, episode_number);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<EpisodeExternalIDsBody>(&url, requester).await
}
