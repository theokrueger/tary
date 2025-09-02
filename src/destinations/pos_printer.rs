use crate::config::{Config, POSConnectionTypes as PCT};
use crate::destinations::TaryDestination;
use crate::storage::Storage;
use escpos::driver::*;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use log::trace;
use std::sync::Arc;

pub struct POSPrinter {
    driver: Box<dyn Driver>,
    printer: Printer,
}

impl TaryDestination for POSPrinter {
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Result<Option<Box<Self>>> {
        if let Some(p) = &cfg.destinations.as_ref().unwrap().pos_printer {
            if !p.enabled {
                return None;
            }
            info!("Setting up POS printer");
            let driver: Box<dyn Driver> = Box::new(match p.connection {
                PCT::USB => NativeUsbDriver::open(0x04b8, 0x0e15).unwrap(),
            });

            let printer = Printer::new(
                driver.clone,
                Protocol::default(),
                Some(PrinterOptions::default()),
            )
            .debug_mode(Some(DebugMode::Dec))
            .init()?
            .smoothing(true)?;

            Some(Box::new(Self {
                driver: driver,
                printer: printer,
            }))
        } else {
            None
        }
    }
}

impl POSPrinter {}
