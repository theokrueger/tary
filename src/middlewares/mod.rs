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
    Self: 'static + Send + Sync,
{
    fn init(cfg: Arc<Config>) -> Result<Option<Arc<Self>>, Box<dyn Error>>
    where
        Self: Sized;

    /// Get order of this middleware
    fn order(&self) -> u32 {
        0
    }

    /// Prepare to do a transformation
    async fn pre_transform(&self) {}

    /// Apply some transformation to content
    async fn transform(&self, ct: Content) -> Content {
        ct
    }

    /// Listener to content, cannot modify it
    /// TODO listen actually
    async fn listen(&self) {}
}

pub struct Middlewares {
    pipeline: Vec<Arc<dyn TaryMiddleware>>,
    listeners: Vec<JoinHandle<()>>,
}

impl Middlewares {
    pub fn new(cfg: Arc<Config>) -> Self {
        let mut mw = Self {
            pipeline: Vec::new(),
            listeners: Vec::new(),
        };

        mw.pipeline
            .push(RegexMiddleware::init(cfg).unwrap().unwrap());

        mw
    }

    pub async fn start(mut self, mut rx: Receiver<Content>, tx: Sender<Content>) {
        for mw in &self.pipeline {
            let m = mw.clone();
            self.listeners.push(tokio::spawn(async move {
                m.listen().await;
            }));
        }

        // self.pipeline.sort_by(|a, b| a.order().cmp(b.order()));

        loop {
            let mut content = rx.recv().await.unwrap();
            for mw in &self.pipeline {
                mw.pre_transform();
                content = mw.transform(content).await;
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
