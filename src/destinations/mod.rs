//! Handler for outputs
mod pos_printer;
use pos_printer::PosPrinter;

mod console;
use console::Console;

use crate::config::Config;
use crate::content::Content;
use crate::storage::Storage;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};

macro_rules! spawn_into_vec {
    ($obj:expr, $vec:ident, $tx:ident) => {
        if let Some(t) = $obj {
            $vec.push(tokio::spawn(t.listen($tx.subscribe())));
        }
    };
}

pub trait TaryDestination {
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Result<Option<Box<Self>>, Box<dyn Error>>;

    async fn listen(self, rx: Receiver<Content>);
}

pub struct Destinations {
    pos_printer: Option<Box<PosPrinter>>,
    console: Option<Box<Console>>,
}

impl Destinations {
    pub fn new(cfg: Arc<Config>, storage: Arc<Storage>) -> Self {
        Self {
            pos_printer: PosPrinter::init(cfg.clone(), storage.clone())
                .expect("Failed to init POS printer destinagion"),
            console: Console::init(cfg.clone(), storage.clone())
                .expect("Failed to init console destination"),
        }
    }

    pub async fn start(self, tx: Sender<Content>) {
        let mut handles = Vec::new();

        spawn_into_vec!(self.pos_printer, handles, tx);
        spawn_into_vec!(self.console, handles, tx);

        for handle in handles {
            handle.await.unwrap();
        }
    }

    pub fn count(&self) -> u32 {
        0 + self.pos_printer.is_some() as u32 + self.console.is_some() as u32
    }
}
