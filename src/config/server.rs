use url::Url;
use serde::{Serialize, Deserialize};

use crate::config::{StreamSpecification, DownloadSpecification};

/////////////////////////////////////////////////////
// IndexerServer
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerServer {
    pub name: String,
    pub description: String,
    pub search_url: Url,

    pub stream: Option<StreamSpecification>,
    pub download: Option<DownloadSpecification>,
}
