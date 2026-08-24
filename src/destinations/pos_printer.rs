use crate::config::{Config, PosConnectionTypes as Pct};
use crate::content::Content;
use crate::destinations::TaryDestination;
use escpos::driver::*;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use log::{error, info, trace};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;

pub struct PosPrinter {
    connection_type: Pct,
    usb_driver: Option<Box<NativeUsbDriver>>,
    usb_id: Option<(u16, u16)>,
}

impl TaryDestination for PosPrinter {
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        if let Some(p) = &cfg.destinations.pos_printer
            && p.enabled
        {
            info!("Setting up POS printer");

            let mut pos = Self {
                connection_type: p.connection.clone(),
                usb_driver: None,
                usb_id: None,
            };

            match p.connection {
                Pct::Usb => {
                    let vid: u16 = p
                        .usb_vid
                        .expect("USB VID Not specified, but USB connection type is selected!");
                    let pid: u16 = p
                        .usb_pid
                        .expect("USB PID Not specified, but USB connection type is selected!");
                    pos.usb_id = Some((vid, pid));
                    pos.init_usb().expect("Unable to init USB");
                }
            }

            Ok(Some(Box::new(pos)))
        } else {
            Ok(None)
        }
    }

    async fn listen(self, mut rx: Receiver<Content>) {
        if let Err(e) = self.print_loop(&mut rx).await {
            error!("Error {e} in printing to pos_printer, aborting this print!");
        }
    }
}

impl PosPrinter {
    fn init_usb(&mut self) -> Result<(), Box<dyn Error>> {
        // Dropping the previous driver (if any) via reassignment to None.
        self.usb_driver = None;
        let (vid, pid) = self.usb_id.expect("No USB pid and vid specified");
        self.usb_driver = Some(Box::new(
            NativeUsbDriver::open(vid, pid)
                .unwrap_or_else(|_| panic!("Unable to open USB device {vid:x}:{pid:x}!")),
        ));
        Ok(())
    }

    async fn print_loop(self, rx: &mut Receiver<Content>) -> Result<(), Box<dyn Error>> {
        let mut printer = Printer::new(
            match self.connection_type {
                Pct::Usb => *self.usb_driver.unwrap(),
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
            let content = match rx.recv().await {
                Ok(c) => c,
                Err(_) => break,
            };
            trace!("POS printer dest received content");

            // title
            printer
                .size(8, 3)?
                .justify(JustifyMode::CENTER)?
                .underline(UnderlineMode::Double)?
                .bold(true)?
                .writeln(content.content_type.to_string().as_str())?;

            printer
                .line_spacing(0)?
                // source
                .reset_size()?
                .feed()?
                .justify(JustifyMode::RIGHT)?
                .underline(UnderlineMode::None)?
                .bold(false)?
                .writeln(content.source.as_str())?;

            // date
            {
                let s = format!("Created: {}", content.date.format(date_format));
                printer.feed()?.writeln(s.as_str())?;
            }

            // due
            if let Some(due) = content.due {
                let s = format!("Due: {}", due.format(date_format));
                printer
                    .size(1, 2)?
                    .feed()?
                    .bold(true)?
                    .writeln(s.as_str())?;
            }

            // dest
            printer.justify(JustifyMode::LEFT)?.bold(false)?;
            if let Some(dest) = content.dest {
                let s = format!("To: {dest}");
                printer.reset_size()?.feed()?.writeln(s.as_str())?;
            }

            // content
            printer
                .reset_line_spacing()?
                .size(1, 2)?
                .feed()?
                .writeln(content.content.as_str())?;

            // cut
            printer.print_cut()?;
        }

        Ok(())
    }
}
