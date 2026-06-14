use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// PagedMovieBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PagedMovieBody {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub genre_ids: Vec<u32>,
    pub id: u32,
    pub title: String,
    // pub original_language: String,
    // pub original_title: String,
    pub overview: String,
    // pub popularity: f32,
    pub poster_path: Option<String>,
    pub release_date: String, // Datetime<chrono::Utc>
}

/////////////////////////////////////////////////////
// MoviePageResultBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MoviePageResultBody {
    pub page: u32,
    pub results: Vec<PagedMovieBody>,
    pub total_pages: u32,
    pub total_results: u32,
}

/////////////////////////////////////////////////////
// PagedSeriesBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PagedSeriesBody {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub genre_ids: Vec<u32>,
    pub id: u32,
    // pub origin_country: Vec<String>,
    // pub original_language: String,
    // pub original_name: String,
    pub overview: String,
    // pub popularity: f32,
    pub poster_path: Option<String>,
    pub first_air_date: Option<String>, // Datetime<chrono::Utc>
    pub name: String,
}

/////////////////////////////////////////////////////
// SeriesPageResultBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SeriesPageResultBody {
    pub page: u32,
    pub results: Vec<PagedSeriesBody>,
    pub total_pages: u32,
    pub total_results: u32,
}

/////////////////////////////////////////////////////
// CollectionBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CollectionBody {
    pub id: u32,
    pub name: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

/////////////////////////////////////////////////////
// GenreBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GenreBody {
    pub id: u32,
    pub name: String,
}

/////////////////////////////////////////////////////
// ProductionCompanyBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProductionCompanyBody {
    pub id: u32,
    pub logo_path: Option<String>,
    pub name: String,
    pub origin_country: String,
}

/////////////////////////////////////////////////////
// ProductionCountryBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProductionCountryBody {
    pub iso_3166_1: String,
    pub name: String,
}

/////////////////////////////////////////////////////
// SpokenLanguageBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SpokenLanguageBody {
    pub english_name: String,
    pub iso_639_1: String,
    // pub name: String,
}

/////////////////////////////////////////////////////
// FullMovieBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullMovieBody {
    pub adult: bool,
    pub backdrop_path: String,
    pub belongs_to_collection: Option<CollectionBody>,
    pub genres: Vec<GenreBody>,
    // pub homepage: String,
    pub id: u32,
    pub imdb_id: String,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub original_title: String,
    pub overview: String,
    // pub popularity: f32,
    pub poster_path: Option<String>,
    // pub production_companies: Vec<ProductionCompanyBody>,
    // pub production_countries: Vec<ProductionCountryBody>,
    pub release_date: String,
    // pub revenue: u32,
    pub runtime: Option<u32>,
    pub spoken_languages: Vec<SpokenLanguageBody>,
    // pub status: String,
    pub tagline: String,
    pub title: String,
    // pub video: bool,
    // pub vote_average: f32,
    // pub vote_count: u32,
}

/////////////////////////////////////////////////////
// CreatedByBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreatedByBody {
    pub id: u32,
    pub credit_id: String,
    pub name: String,
    pub gender: u32,
    pub profile_path: Option<String>,
}

/////////////////////////////////////////////////////
// LastEpisodeToAirBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LastEpisodeToAirBody {
    pub id: u32,
    pub name: String,
    pub overview: String,
    // pub vote_average: f32,
    // pub vote_count: u32,
    pub air_date: Option<String>,
    pub episode_number: u32,
    pub production_code: String,
    pub runtime: Option<u32>,
    pub season_number: u32,
    pub show_id: u32,
    pub still_path: Option<String>,
}

/////////////////////////////////////////////////////
// NetworkBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NetworkBody {
    pub id: u32,
    pub logo_path: Option<String>,
    pub name: String,
    pub origin_country: String,
}

/////////////////////////////////////////////////////
// SeasonBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SeasonBody {
    pub air_date: Option<String>,
    pub episode_count: u32,
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub poster_path: Option<String>,
    pub season_number: u32,
    // pub vote_average: f32,
}

/////////////////////////////////////////////////////
// FullSeriesBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullSeriesBody {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub created_by: Vec<CreatedByBody>,
    pub episode_run_time: Vec<u32>,
    pub first_air_date: Option<String>,
    pub genres: Vec<GenreBody>,
    // pub homepage: String,
    pub id: u32,
    pub in_production: bool,
    pub languages: Vec<String>,
    pub last_air_date: Option<String>,
    pub last_episode_to_air: Option<LastEpisodeToAirBody>,
    pub name: String,
    pub next_episode_to_air: Option<String>,
    pub networks: Vec<NetworkBody>,
    pub number_of_episodes: u32,
    pub number_of_seasons: u32,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub original_name: String,
    pub overview: String,
    // pub popularity: f32,
    pub poster_path: Option<String>,
    // pub production_companies: Vec<ProductionCompanyBody>,
    // pub production_countries: Vec<ProductionCountryBody>,
    pub seasons: Vec<SeasonBody>,
    pub spoken_languages: Vec<SpokenLanguageBody>,
    // pub status: String,
    // pub tagline: String,
    // #[serde(rename = "type")]
    // pub series_type: String,
    // pub vote_average: f32,
    // pub vote_count: u32,
}

/////////////////////////////////////////////////////
// CrewBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrewBody {
    pub department: String,
    pub job: String,
    pub credit_id: String,
    pub adult: bool,
    // pub gender: u32,
    pub id: u32,
    pub known_for_department: String,
    pub name: String,
    pub original_name: String,
    // pub popularity: f32,
    pub profile_path: Option<String>,
}

/////////////////////////////////////////////////////
// GuestStarBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GuestStarBody {
    pub character: String,
    // pub credit_id: String,
    // pub order: u32,
    pub adult: bool,
    // pub gender: u32,
    pub id: u32,
    pub known_for_department: String,
    pub name: String,
    pub original_name: String,
    // pub popularity: f32,
    pub profile_path: Option<String>,
}

/////////////////////////////////////////////////////
// EpisodeBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EpisodeBody {
    pub air_date: Option<String>,
    pub episode_number: u32,
    pub episode_type: String,
    pub id: u32,
    pub name: String,
    pub overview: String,
    // pub production_code: String,
    pub runtime: Option<u32>,
    pub season_number: u32,
    pub show_id: u32,
    pub still_path: String,
    // pub vote_average: f32,
    // pub vote_count: u32,
    pub crew: Vec<CrewBody>,
    pub guest_stars: Vec<GuestStarBody>,
}

/////////////////////////////////////////////////////
// FullSeasonBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullSeasonBody {
    // pub _id: String,
    pub air_date: Option<String>,
    pub episodes: Vec<EpisodeBody>,
    pub name: String,
    pub networks: NetworkBody,
    pub overview: String,
    pub id: u32,
    pub poster_path: String,
    pub season_number: u32,
    // pub vote_average: f32,
}

/////////////////////////////////////////////////////
// FullEpisodeBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullEpisodeBody {
    pub air_date: Option<String>,
    pub crew: Vec<CrewBody>,
    pub episode_number: u32,
    pub guest_stars: Vec<GuestStarBody>,
    pub name: String,
    pub overview: String,
    pub id: u32,
    // pub production_code: String,
    pub runtime: Option<u32>,
    pub season_number: u32,
    pub still_path: Option<String>,
    // pub vote_average: f32,
    // pub vote_count: u32,
}

/////////////////////////////////////////////////////
// MovieExternalIDsBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MovieExternalIDsBody {
    pub id: u32,
    pub imdb_id: String,
    // pub wikidata_id: String,
    // pub facebook_id: String,
    // pub instagram_id: String,
    // pub twitter_id: String,
}

/////////////////////////////////////////////////////
// SeriesExternalIDsBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SeriesExternalIDsBody {
    pub id: u32,
    pub imdb_id: String,
    // pub freebase_mid: String,
    // pub freebase_id: String,
    // pub tvdb_id: u32,
    // pub tvrage_id: u32,
    // pub wikidata_id: String,
    // pub facebook_id: String,
    // pub instagram_id: String,
    // pub twitter_id: String,
}

/////////////////////////////////////////////////////
// SeasonExternalIDsBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SeasonExternalIDsBody {
    pub id: u32,
    pub imdb_id: String,
    // pub freebase_mid: String,
    // pub freebase_id: String,
    // pub tvdb_id: u32,
    // pub tvrage_id: u32,
    // pub wikidata_id: String,
}

/////////////////////////////////////////////////////
// EpisodeExternalIDsBody
/////////////////////////////////////////////////////
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EpisodeExternalIDsBody {
    pub id: u32,
    pub imdb_id: String,
    // pub freebase_mid: String,
    // pub freebase_id: String,
    // pub tvdb_id: u32,
    // pub tvrage_id: u32,
    // pub wikidata_id: String,
}
