//! Handler for middlewares
//! Middleware is generally content transformation and/or inference upon it
mod i2c_display;
use i2c_display::I2cDisplayMiddleware;

use crate::{config::Config, content::Content};
use std::{error::Error, sync::Arc};
use tokio::sync::broadcast::{Receiver, Sender};

pub trait TaryMiddleware {
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>>;

    async fn transform(&mut self, ct: Content) -> Content;
}

pub struct Middlewares {
    i2c: I2cDisplayMiddleware,
}

impl Middlewares {
    pub fn new(cfg: Arc<Config>) -> Self {
        Self {
            i2c: *I2cDisplayMiddleware::init(cfg.clone()).unwrap().unwrap(),
        }
    }

    pub async fn start(mut self, mut rx: Receiver<Content>, tx: Sender<Content>) {
        loop {
            let mut content = rx.recv().await.unwrap();
            content = self.i2c.transform(content).await;
            tx.send(content).unwrap();
        }
    }

    pub fn count(&self) -> usize {
        0
    }
}
