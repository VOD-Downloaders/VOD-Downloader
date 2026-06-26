use std::collections::HashMap;

use url::Url;
use serde::{Serialize, Deserialize};

use crate::config::StreamType;

/////////////////////////////////////////////////////
// IndexerServer
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerServer {
    pub name: String,
    pub description: String,
    pub search_url: Url,
    #[serde(rename = "type")]
    pub stream_type: Option<StreamType>,
    pub headers: Option<HashMap<String, String>>,
}
