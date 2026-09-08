//! Handler for middlewares
//! Middleware is generally content transformation and/or inference upon it
mod regex_mw;
use regex_mw::RegexMiddleware;

use crate::{config::Config, content::Content};
use async_trait::async_trait;
use log::error;
use std::{error::Error, sync::Arc};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    task::JoinHandle,
};

#[async_trait]
pub trait TaryMiddleware
where
    Self: 'static + Send + Sync,
{
    fn init(cfg: Arc<Config>) -> Result<Option<Arc<Self>>, Box<dyn Error>>
    where
        Self: Sized;

    /// Get order of this middleware
    fn order(&self) -> u32 {
        u32::MAX
    }

    /// Prepare to do a transformation
    /// It is up to the middleware implementation to handle cases where `pre_transform()`
    /// doesn't finish before `transform()` is called.
    async fn pre_transform(&self, _ct: Content) {}

    /// Apply some transformation to content
    async fn transform(&self, ct: Content) -> Content {
        ct
    }

    /// Listener to content, cannot modify it
    async fn listen(&self, _rx: Receiver<Content>) {}
}

pub struct Middlewares
where
    Self: 'static,
{
    pipeline: Vec<Arc<dyn TaryMiddleware>>,
    listeners: Vec<JoinHandle<()>>,
}

impl Middlewares {
    pub fn new(cfg: Arc<Config>) -> Self {
        let mut mw = Self {
            pipeline: Vec::new(),
            listeners: Vec::new(),
        };

        for x in [RegexMiddleware::init(cfg)] {
            match x {
                Ok(Some(a)) => mw.pipeline.push(a),
                Ok(None) => {}
                Err(e) => {
                    error!("{e}")
                }
            }
        }

        mw.pipeline.sort_by(|a, b| a.order().cmp(&b.order()));

        mw
    }

    pub async fn start(mut self, rx: Sender<Content>, tx: Sender<Content>) {
        // listeners
        for mw in &self.pipeline {
            let m = mw.clone();
            let rxr = rx.subscribe();
            self.listeners.push(tokio::spawn(async move {
                m.listen(rxr).await;
            }));
        }

        // transformers
        let mut rxr = rx.subscribe();
        loop {
            let mut content = rxr.recv().await.unwrap();

            for mw in &self.pipeline {
                let orig = content.clone();
                tokio::spawn(async move {
                    mw.pre_transform(orig).await;
                });
            }

            for mw in &self.pipeline {
                content = mw.transform(content).await;
            }

            tx.send(content).unwrap();
        }

        // idk why i have this here
        #[allow(unreachable_code)]
        for handle in self.listeners {
            handle.await.unwrap();
        }
    }

    pub fn count(&self) -> usize {
        self.pipeline.len()
    }
}
