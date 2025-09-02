//! Handler for input sources
mod telegram;
use telegram::Telegram;

use crate::config::Config;
use crate::storage::Storage;
use std::sync::Arc;
use time::Date;

pub struct Content {
    source: String,
    dest: Option<String>,
    date: Date,
    due: Option<Date>,
}

pub trait TarySource {
    async fn listen(self);
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Option<Box<Self>>;
}

#[derive(Default)]
pub struct Sources {
    telegram: Option<Box<Telegram>>,
}

impl Sources {
    pub fn new(cfg: Arc<Config>, storage: Arc<Storage>) -> Self {
        if let Some(_) = cfg.sources {
            Self {
                telegram: Telegram::init(cfg.clone(), storage.clone()),
            }
        } else {
            Self::default()
        }
    }

    pub async fn start(self) {
        let mut handles = Vec::new();
        if let Some(t) = self.telegram {
            handles.push(tokio::spawn(t.listen()));
        }
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
