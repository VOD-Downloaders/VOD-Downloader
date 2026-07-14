use url::Url;
use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// AudioChannel
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize, Serialize)]
pub struct AudioChannel {
    pub lang: String,
    pub segments: Vec<Url>,
}

/////////////////////////////////////////////////////
// Stream
/////////////////////////////////////////////////////
#[derive(Debug, Deserialize, Serialize)]
pub struct Stream {
    pub quality: String,
    pub video_segments: Vec<Url>,
    pub audio_channels: Vec<AudioChannel>, // lang, url // empty if original
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
