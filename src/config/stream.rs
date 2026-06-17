use std::collections::HashMap;

use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// StreamType
/////////////////////////////////////////////////////
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    Index,
    Mp4,
}

/////////////////////////////////////////////////////
// StreamSpecification
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSpecification {
    #[serde(rename = "type")]
    pub stream_type: StreamType,
    pub headers: HashMap<String, String>,
}
