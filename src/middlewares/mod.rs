//! Handler for middlewares
//! Middleware is generally content transformation and/or inference upon it
use crate::{config::Config, content::Content};
use std::{error::Error, sync::Arc};
use tokio::sync::broadcast::{Receiver, Sender};

pub trait TaryMiddleware {
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>>;

    async fn listen(self, rx: Receiver<Content>);
}

pub struct Middlewares {}

impl Middlewares {
    pub fn new(cfg: Arc<Config>) -> Self {
        Self {}
    }

    pub async fn start(self, tx: Sender<Content>) {
        // let mut handles = Vec::new();

        // for handle in handles {
        //     handle.await.unwrap();
        // }
    }

    pub fn count(&self) -> usize {
        0
    }
}
