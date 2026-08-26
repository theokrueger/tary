use crate::middlewares::TaryMiddleware;
use crate::{config::Config, content::Content};
use async_trait::async_trait;
use log::info;
use std::{error::Error, sync::Arc};
use tokio::sync::broadcast::Receiver;

pub struct RegexMiddleware {}

#[async_trait]
impl TaryMiddleware for RegexMiddleware {
    fn init(_cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        Ok(Some(Box::new(Self {})))
    }

    fn transform(&mut self, ct: Content) -> Content {
        info!("aaaaaaa");
        ct
    }

    async fn listen(&mut self) {
        info!("bbbbbb");
    }

    fn order(&self) -> u32 {
        0
    }
}
