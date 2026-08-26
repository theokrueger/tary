//! Handler for middlewares
//! Middleware is generally content transformation and/or inference upon it
mod regex_mw;
use regex_mw::RegexMiddleware;

use crate::{config::Config, content::Content};
use async_trait::async_trait;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;
use tokio::{
    sync::broadcast::{Receiver, Sender},
    task::JoinHandle,
};

#[async_trait]
pub trait TaryMiddleware
where
    Self: 'static + Send,
{
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>>
    where
        Self: Sized;

    /// Get order of this middleware
    fn order(&self) -> u32;

    /// Apply some transformation to content
    fn transform(&mut self, ct: Content) -> Content;

    /// Listener to content, cannot modify
    async fn listen(&mut self);
}

pub struct Middlewares {
    pipeline: Vec<Arc<Mutex<Box<dyn TaryMiddleware>>>>,
    listeners: Vec<JoinHandle<()>>,
}

impl Middlewares {
    pub fn new(cfg: Arc<Config>) -> Self {
        let mut mw = Self {
            pipeline: Vec::new(),
            listeners: Vec::new(),
        };

        mw.pipeline.push(Arc::new(Mutex::new(
            RegexMiddleware::init(cfg).unwrap().unwrap(),
        )));

        // let mut a = I2cDisplayMiddleware::init(cfg).unwrap().unwrap();
        // mw.pipeline.push(Arc::new(Mutex::new(a)));

        mw
    }

    pub async fn start(mut self, mut rx: Receiver<Content>, tx: Sender<Content>) {
        for mw in &self.pipeline {
            let m = mw.clone();
            self.listeners.push(tokio::spawn(async move {
                m.lock().await.listen().await;
            }));
        }

        // self.pipeline.sort_by(|a, b| a.order().cmp(b.order()));

        loop {
            let mut content = rx.recv().await.unwrap();
            for mw in &self.pipeline {
                content = mw.lock().await.transform(content);
            }
            tx.send(content).unwrap();
        }

        #[allow(unreachable_code)]
        for handle in self.listeners {
            handle.await.unwrap();
        }
    }

    pub fn count(&self) -> usize {
        self.pipeline.len()
    }
}
