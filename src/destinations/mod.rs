//! Handler for outputs
mod pos_printer;
use pos_printer::POSPrinter;

use crate::config::Config;
use crate::storage::Storage;
use std::error::Error;
use std::sync::Arc;

pub trait TaryDestination {
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Result<Option<Box<Self>>, Box<dyn Error>>;
}

#[derive(Default)]
pub struct Destinations {
    pos_printer: Option<Box<POSPrinter>>,
}

impl Destinations {
    pub fn new(cfg: Arc<Config>, storage: Arc<Storage>) -> Self {
        if let Some(_) = cfg.destinations {
            Self {
                pos_printer: POSPrinter::init(cfg.clone(), storage.clone())
                    .expect("Failed to init POS printer"),
            }
        } else {
            Self::default()
        }
    }
}
