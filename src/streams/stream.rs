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

impl DownloadableStream {
    pub fn rate_limit_host(&self) -> String {
        const INVALID_HOST: &str = "invalidurl.com";
        let invalid_url = Url::parse(format!("http://{}", INVALID_HOST).as_str()).unwrap();

        let host: Option<&str> = match &self.stream_type {
            DownloadableStreamType::Segments(urls) => urls.first().unwrap_or(&invalid_url).host_str(),
            DownloadableStreamType::Mp4(url) => url.host_str(),
        };

        host.unwrap_or(INVALID_HOST).to_string()
    }
}
