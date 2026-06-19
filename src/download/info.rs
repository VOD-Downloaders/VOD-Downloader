use std::{path::PathBuf};

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/////////////////////////////////////////////////////
// DownloadStatus
/////////////////////////////////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Converting,
    DownloadingSegments { amount: u32, total: u32 },
    // TODO: MP4
    Finished,
    Failed(String),
}

/////////////////////////////////////////////////////
// DownloadInfo
/////////////////////////////////////////////////////
#[derive(Debug)]
pub struct DownloadInfo {
    pub start_time: DateTime<Utc>, // readonly
    pub end_time: RwLock<Option<DateTime<Utc>>>,
    pub output_file: PathBuf, // readonly
    pub status: RwLock<DownloadStatus>,
}

impl DownloadInfo {
    pub fn new(output_file: &std::path::Path) -> Self {
        Self {
            start_time: Utc::now(),
            end_time: RwLock::new(None),
            output_file: output_file.to_path_buf(),
            status: RwLock::new(DownloadStatus::Pending),
        }
    }
}
