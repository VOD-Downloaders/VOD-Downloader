use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::config::DownloadOrder;

/////////////////////////////////////////////////////
// DownloadScheduler
/////////////////////////////////////////////////////
#[derive(Debug)]
pub struct DownloadScheduler {
    sequential: Mutex<HashMap<String, Arc<Semaphore>>>,
}

impl DownloadScheduler {
    pub fn new() -> Self {
        Self {
            sequential: Mutex::new(HashMap::new()),
        }
    }

    // Acquire a download slot for a host. For Sequential this blocks until the
    // previous download for that host drops its permit.
    pub async fn acquire(&self, host: &str, order: &DownloadOrder) -> Option<OwnedSemaphorePermit> {
        trace!("Acquiring semaphore for host: \"{}\", order: {:?}", host, order);

        match order {
            DownloadOrder::Parallel => None,
            DownloadOrder::Sequential => {
                let semaphore = {
                    let mut map = self.sequential.lock().unwrap();
                    Arc::clone(map.entry(host.to_string()).or_insert_with(|| Arc::new(Semaphore::new(1))))
                };

                trace!("Got semaphore./");

                Some(semaphore.acquire_owned().await.expect("Semaphore is never closed."))
            },
        }
    }
}
