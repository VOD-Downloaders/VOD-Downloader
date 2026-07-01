use url::Url;
use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// Stream
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize, Serialize)]
pub struct Stream {
    pub quality: String,
    pub video_segments: Vec<Url>,
    pub audio_channels: Vec<(String, Url)>, // lang, url
}

impl Stream {
    pub fn rate_limit_host(&self) -> String {
        const INVALID_HOST: &str = "https://invalidurl.com";
        self.video_segments
            .first()
            .map(|segment| segment.to_string())
            .unwrap_or(INVALID_HOST.to_string())
    }
}

/////////////////////////////////////////////////////
// Subtitle
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize, Serialize)]
pub struct Subtitle {
    pub lang: String,
    pub url: Url,
}

/////////////////////////////////////////////////////
// Streams
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize, Serialize)]
pub struct Streams {
    pub streams: Vec<Stream>,
    pub subtitles: Vec<Subtitle>,
}
