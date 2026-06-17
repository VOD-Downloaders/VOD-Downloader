use std::collections::HashMap;

use url::Url;
use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// SearchSpecification
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSpecification {
    pub emulate_url: Url,
    pub headers: HashMap<String, String>,
}
