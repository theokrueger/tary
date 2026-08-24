//! Handler for input sources
use crate::content::Content;

use crate::config::Config;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub trait TarySource {
    async fn listen(self, tx: Sender<Content>);
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Option<Box<Self>>;
}

pub struct Sources {}

impl Sources {
    pub fn new(cfg: Arc<Config>) -> Self {
        Self {}
    }

    pub async fn start(self, tx: Sender<Content>) {
        let mut handles = Vec::new();

        for handle in handles {
            handle.await.unwrap();
        }
    }

    pub fn count(&self) -> u32 {
        0
    }
}
