use std::collections::HashMap;

use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// StreamType
/////////////////////////////////////////////////////
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    #[serde(rename = "index")]
    Index,
    #[serde(rename = "master")]
    Master,
    #[serde(rename = "mp4")]
    Mp4,
    #[serde(rename = "video_audio")]
    VideoAudo,
}

/////////////////////////////////////////////////////
// StreamSpecification
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSpecification {
    #[serde(rename = "type")]
    pub stream_type: StreamType,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}
