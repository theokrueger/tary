//! Handler for input sources
use crate::{config::Config, content::Content};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

mod http_server;
use http_server::HttpServerSource;

pub trait TarySource {
    async fn listen(self, tx: Sender<Content>);
    fn init(cfg: Arc<Config>) -> Option<Box<Self>>;
}

pub struct Sources {
    http_server: Option<Box<HttpServerSource>>,
}

impl Sources {
    pub fn new(cfg: Arc<Config>) -> Self {
        Self {
            http_server: HttpServerSource::init(cfg.clone()),
        }
    }

    pub async fn start(self, tx: Sender<Content>) {
        let mut handles = Vec::new();

        if let Some(source) = self.http_server {
            handles.push(tokio::spawn(source.listen(tx.clone())));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    pub fn count(&self) -> u32 {
        self.http_server.is_some() as u32
    }
}
