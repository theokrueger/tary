//! Handler for outputs
mod console;
mod pos_printer;
use console::Console;
use pos_printer::PosPrinter;

use crate::{config::Config, content::Content};
use std::{error::Error, sync::Arc};
use tokio::sync::broadcast::{Receiver, Sender};

pub trait TaryDestination {
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>>;

    async fn listen(self, rx: Receiver<Content>);
}

pub struct Destinations {
    pos_printer: Option<Box<PosPrinter>>,
    console: Option<Box<Console>>,
}

impl Destinations {
    pub fn new(cfg: Arc<Config>) -> Self {
        Self {
            pos_printer: PosPrinter::init(cfg.clone())
                .expect("Failed to init POS printer destination"),
            console: Console::init(cfg.clone()).expect("Failed to init console destination"),
        }
    }

    pub async fn start(self, tx: Sender<Content>) {
        let mut handles = Vec::new();

        if let Some(dest) = self.pos_printer {
            handles.push(tokio::spawn(dest.listen(tx.subscribe())));
        }
        if let Some(dest) = self.console {
            handles.push(tokio::spawn(dest.listen(tx.subscribe())));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    pub fn count(&self) -> u32 {
        self.pos_printer.is_some() as u32 + self.console.is_some() as u32
    }
}
