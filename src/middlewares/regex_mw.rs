use crate::middlewares::TaryMiddleware;
use crate::{config::Config, content::Content};
use async_trait::async_trait;
use log::info;
use std::{error::Error, sync::Arc};
use tokio::sync::broadcast::Receiver;

pub struct RegexMiddleware {}

#[async_trait]
impl TaryMiddleware for RegexMiddleware {
    fn init(_cfg: Arc<Config>) -> Result<Option<Arc<Self>>, Box<dyn Error>> {
        Ok(Some(Arc::new(Self {})))
    }

    async fn transform(&self, ct: Content) -> Content {
        info!("transform");
        ct
    }

    async fn listen(&self) {
        info!("listen");
    }
}
