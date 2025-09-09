//! Handler for input sources
mod telegram;
use telegram::Telegram;

use crate::content::Content;

use crate::config::Config;
use crate::storage::Storage;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

pub trait TarySource {
    async fn listen(self, tx: Sender<Content>);
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Option<Box<Self>>;
}

pub struct Sources {
    telegram: Option<Box<Telegram>>,
}

impl Sources {
    pub fn new(cfg: Arc<Config>, storage: Arc<Storage>) -> Self {
        Self {
            telegram: Telegram::init(cfg.clone(), storage.clone()),
        }
    }

    pub async fn start(self, tx: Sender<Content>) {
        let mut handles = Vec::new();
        if let Some(t) = self.telegram {
            handles.push(tokio::spawn(t.listen(tx.clone())));
        }
        for handle in handles {
            handle.await.unwrap();
        }
    }

    pub fn count(&self) -> u32 {
        0 + self.telegram.is_some() as u32
    }
}
