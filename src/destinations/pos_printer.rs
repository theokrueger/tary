use crate::config::{Config, POSConnectionTypes as PCT};
use crate::destinations::TaryDestination;
use crate::storage::Storage;
use escpos::driver::*;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use log::trace;
use std::error::Error;
use std::sync::Arc;

pub struct POSPrinter {
    driver: Box<dyn Driver>,
}

impl TaryDestination for POSPrinter {
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        if let Some(p) = &cfg.destinations.as_ref().unwrap().pos_printer {
            if !p.enabled {
                return Ok(None);
            }
            info!("Setting up POS printer");
            let driver = match p.connection {
                PCT::USB => {
                    let vid: u16 = p
                        .usb_vid
                        .expect("USB VID Not specified, but USB connection type is selected!");
                    let pid: u16 = p
                        .usb_pid
                        .expect("USB PID Not specified, but USB connection type is selected!");
                    NativeUsbDriver::open(vid, pid)
                        .expect(format!("Unable to open USB device {vid:x}:{pid:x}!").as_str())
                }
            };

            Ok(Some(Box::new(Self {
                driver: Box::new(driver),
            })))
        } else {
            Ok(None)
        }
    }
}

impl POSPrinter {}
