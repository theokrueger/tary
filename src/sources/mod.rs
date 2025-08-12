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

pub trait SourceListener {
    async fn listen(&self);
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> impl SourceListener;
}

pub struct Sources {
    telegram: Telegram,
}

impl Sources {
    pub fn new(cfg: Arc<Config>, storage: Arc<Storage>) -> Self {
        Self {
            telegram: Telegram::init(cfg.clone(), storage.clone()),
        }
    }

    pub async fn start(&self) {
        tokio::join!(self.telegram.listen());
    }
}
