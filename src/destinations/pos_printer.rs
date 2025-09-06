use crate::config::{Config, PosConnectionTypes as Pct};
use crate::content::Content;
use crate::destinations::TaryDestination;
use crate::storage::Storage;
use escpos::driver::*;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use log::trace;
use send_wrapper::SendWrapper;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};

pub struct PosPrinter {
    connection_type: Pct,
    usb_driver: Option<Box<NativeUsbDriver>>,
}

impl TaryDestination for PosPrinter {
    fn init(cfg: Arc<Config>, storage: Arc<Storage>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        if let Some(p) = &cfg.destinations.pos_printer
            && p.enabled
        {
            info!("Setting up POS printer");

            let mut pos = Self {
                connection_type: p.connection.clone(),
                usb_driver: None,
            };

            match p.connection {
                Pct::USB => {
                    let vid: u16 = p
                        .usb_vid
                        .expect("USB VID Not specified, but USB connection type is selected!");
                    let pid: u16 = p
                        .usb_pid
                        .expect("USB PID Not specified, but USB connection type is selected!");
                    pos.usb_driver = Some(Box::new(
                        NativeUsbDriver::open(vid, pid)
                            .expect(format!("Unable to open USB device {vid:x}:{pid:x}!").as_str()),
                    ));
                }
            };

            Ok(Some(Box::new(pos)))
        } else {
            Ok(None)
        }
    }

    async fn listen(self, mut rx: Receiver<Content>) {
        let f = async || -> Result<(), Box<dyn Error>> {
            let mut printer = Printer::new(
                match self.connection_type {
                    Pct::USB => *self.usb_driver.unwrap(),
                },
                Protocol::default(),
                Some(PrinterOptions::default()),
            );
            printer
                .debug_mode(Some(DebugMode::Dec))
                .init()?
                .smoothing(true)?
                .bold(true)?;

            let date_format = "%a %Y-%m-%d %H:%M";

            loop {
                let content = rx.recv().await.unwrap();
                trace!("POS printer dest received content");

                // title
                printer
                    .size(4, 3)?
                    .justify(JustifyMode::CENTER)?
                    .underline(UnderlineMode::Double)?
                    .bold(true)?
                    .writeln("TODO")?;

                // source
                printer
                    .feed()?
                    .size(2, 1)?
                    .justify(JustifyMode::LEFT)?
                    .underline(UnderlineMode::None)?
                    .bold(false)?
                    .writeln(content.source.as_str())?;

                // date
                {
                    let s = format!("Created: {}", content.date.format(date_format));
                    printer
                        .reset_size()?
                        .feed()?
                        .justify(JustifyMode::RIGHT)?
                        .underline(UnderlineMode::None)?
                        .bold(false)?
                        .writeln(s.as_str())?;
                }

                // due
                if let Some(due) = content.due {
                    let s = format!("Due: {}", due.format(date_format));
                    printer
                        .size(1, 2)?
                        .feed()?
                        .justify(JustifyMode::RIGHT)?
                        .underline(UnderlineMode::None)?
                        .bold(true)?
                        .writeln(s.as_str())?;
                }

                // dest
                if let Some(dest) = content.dest {
                    let s = format!("To: {dest}");
                    printer
                        .reset_size()?
                        .feed()?
                        .justify(JustifyMode::LEFT)?
                        .underline(UnderlineMode::None)?
                        .bold(false)?
                        .writeln(s.as_str())?;
                }

                // content
                printer
                    .reset_size()?
                    .feed()?
                    .justify(JustifyMode::LEFT)?
                    .underline(UnderlineMode::None)?
                    .bold(false)?
                    .writeln(content.content.as_str())?;

                // cut
                printer.print_cut()?;
            }
        };

        match f().await {
            Ok(_) => {}
            Err(e) => error!("Error {e} in printing to pos_printer, aborting this destination!"),
        }
    }
}

impl PosPrinter {}
