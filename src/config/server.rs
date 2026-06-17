use std::collections::HashMap;

use url::Url;
use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// IndexerServer
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerServer {
    pub name: String,
    pub description: String,
    pub search_url: Url,
    pub headers: Option<HashMap<String, String>>,
}
