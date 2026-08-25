use crate::middlewares::TaryMiddleware;
use crate::{config::Config, content::Content};
use std::{error::Error, sync::Arc};

pub struct StubMiddleware {}

impl TaryMiddleware for StubMiddleware {
    fn init(_cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        Ok(Some(Box::new(StubMiddleware {})))
    }

    async fn transform(&mut self, ct: Content) -> Content {
        ct
    }
}
