use url::Url;
use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// DownloadableStreamType
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub enum DownloadableStreamType {
    Segments(Vec<Url>),
    Mp4(Url),
    // TODO: Video, Audio
}

impl Default for DownloadableStreamType {
    fn default() -> Self {
        DownloadableStreamType::Segments(Vec::new())
    }
}

/////////////////////////////////////////////////////
// Stream
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadableStream {
    pub resolution: String,
    pub stream_type: DownloadableStreamType,
}
