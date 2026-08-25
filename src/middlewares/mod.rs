//! Handler for middlewares
//! Middleware is generally content transformation and/or inference upon it
mod stub;
use stub::StubMiddleware;

use crate::{config::Config, content::Content};
use std::{error::Error, sync::Arc};
use tokio::sync::broadcast::{Receiver, Sender};

pub trait TaryMiddleware {
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>>;

    async fn transform(&mut self, ct: Content) -> Content;
}

pub struct Middlewares {
    sm: StubMiddleware,
}

impl Middlewares {
    pub fn new(cfg: Arc<Config>) -> Self {
        Self {
            sm: *StubMiddleware::init(cfg.clone()).unwrap().unwrap(),
        }
    }

    pub async fn start(mut self, mut rx: Receiver<Content>, tx: Sender<Content>) {
        loop {
            let mut content = rx.recv().await.unwrap();
            content = self.sm.transform(content).await;
            tx.send(content).unwrap();
        }
    }

    pub fn count(&self) -> usize {
        0
    }
}
