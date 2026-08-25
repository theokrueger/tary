use crate::middlewares::TaryMiddleware;
use crate::{config::Config, content::Content};
use std::{error::Error, sync::Arc};

pub struct I2cDisplayMiddleware {}

impl TaryMiddleware for I2cDisplayMiddleware {
    fn init(_cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        Ok(Some(Box::new(I2cDisplayMiddleware {})))
    }

    async fn transform(&mut self, ct: Content) -> Content {
        ct
    }
}
