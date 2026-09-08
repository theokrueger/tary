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

    fn has_pre_transform(&self) -> bool {
        false
    }

    fn has_transform(&self) -> bool {
        false
    }

    fn has_sniff(&self) -> bool {
        false
    }

    /// Prepare to do a transformation. For high-latency tasks.
    /// It is up to the middleware implementation to handle cases where `pre_transform()`
    /// doesn't finish before `transform()` is called.
    async fn pre_transform(&self, _ct: Content) {}

    /// Apply some transformation to content
    async fn transform(&self, ct: Content) -> Content {
        ct
    }

    /// Sniff content, cannot modify it
    async fn sniff(&self, _rx: Receiver<Content>) {}
}

pub struct Middlewares {
    pipeline: Vec<Arc<dyn TaryMiddleware>>,
    sniffers: Vec<JoinHandle<()>>,
}

impl Middlewares {
    pub fn new(cfg: Arc<Config>) -> Self {
        let mut mw = Self {
            pipeline: Vec::new(),
            sniffers: Vec::new(),
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
        // sniffers
        for mw in &self.pipeline {
            if mw.has_sniff() {
                let m = mw.clone();
                let rxr = rx.subscribe();
                self.sniffers.push(tokio::spawn(async move {
                    m.sniff(rxr).await;
                }));
            }
        }

        // transformers
        let mut rxr = rx.subscribe();
        loop {
            let mut content = rxr.recv().await.unwrap();

            for mw in &self.pipeline {
                if mw.has_pre_transform() {
                    let m = mw.clone();
                    let orig = content.clone();
                    tokio::spawn(async move {
                        m.pre_transform(orig).await;
                    });
                }
            }

            for mw in &self.pipeline {
                if mw.has_transform() {
                    content = mw.transform(content).await;
                }
            }

            tx.send(content).unwrap();
        }

        // idk why i have this here
        #[allow(unreachable_code)]
        for handle in self.sniffers {
            handle.await.unwrap();
        }
    }

    pub fn count(&self) -> usize {
        self.pipeline.len()
    }
}
