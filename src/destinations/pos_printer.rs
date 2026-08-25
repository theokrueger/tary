use crate::{
    Content,
    config::{Config, PosConnectionTypes as Pct},
    destinations::TaryDestination,
};
use escpos::{driver::*, printer::Printer, printer_options::PrinterOptions, utils::*};
use log::{error, info, trace};
use std::{error::Error, sync::Arc, time::Duration};
use tokio::sync::broadcast::Receiver;

pub struct PosPrinter {
    connection_type: Pct,
    usb_driver: Option<Box<NativeUsbDriver>>,
    usb_id: Option<(u16, u16)>,
    usb_initialized: bool,
}

impl TaryDestination for PosPrinter {
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        if let Some(p) = &cfg.destinations.pos_printer
            && p.enabled
        {
            trace!("Setting up POS printer");

            let mut pos = Self {
                connection_type: p.connection.clone(),
                usb_driver: None,
                usb_id: None,
                usb_initialized: false,
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
                    info!("Setting up USB POS Printer at [{:x}:{:x}]", vid, pid);
                    pos.init_usb().unwrap_or_else(|e| {
                        error!("Unable to init USB POS Printer with {e}! Will retry lazily.")
                    });
                }
            }

            return Ok(Some(Box::new(pos)));
        }
        Ok(None)
    }

    async fn listen(self, mut rx: Receiver<Content>) {
        if let Err(e) = self.print_loop(&mut rx).await {
            error!("Error {e} in printing to pos_printer, dropping this destination!");
        }
    }
}

impl PosPrinter {
    fn init_usb(&mut self) -> Result<(), Box<dyn Error>> {
        self.usb_driver = None;
        let (vid, pid) = self.usb_id.unwrap();
        self.usb_driver = Some(Box::new(NativeUsbDriver::open(vid, pid)?));
        self.usb_initialized = true;
        Ok(())
    }

    async fn print_loop(mut self, rx: &mut Receiver<Content>) -> Result<(), Box<dyn Error>> {
        let date_format = "%a %Y-%m-%d %H:%M";

        loop {
            let Ok(content) = rx.recv().await else {
                error!("POS Printer failed to recieve content");
                break;
            };
            trace!("POS printer dest received content");

            for i in 1..4 {
                // reinit
                match self.connection_type {
                    Pct::Usb => {
                        if !self.usb_initialized {
                            // sleep for usb reinit
                            std::thread::sleep(Duration::from_millis(5000));
                            if let Err(e) = self.init_usb() {
                                error!("[{i}/3] USB POS Printer {e}");
                                continue;
                            }
                        }
                    }
                };

                // print
                match self.try_print(&content, date_format) {
                    Ok(_) => break,
                    Err(e) => {
                        error!("[{i}/3] POS printer error: {e}");
                        self.usb_initialized = false;
                        continue;
                    }
                }
            }
        }

        Ok(())
    }

    /// Print content over a fresh connection
    fn try_print(&mut self, content: &Content, date_format: &str) -> Result<(), Box<dyn Error>> {
        let driver = match self.connection_type {
            Pct::Usb => *self.usb_driver.clone().unwrap(),
        };
        let mut printer = Printer::new(
            driver.clone(),
            Protocol::default(),
            Some(PrinterOptions::default()),
        );
        printer
            .debug_mode(Some(DebugMode::Dec))
            .smoothing(true)?
            .init()?;

        // title
        printer
            .size(8, 3)?
            .justify(JustifyMode::CENTER)?
            .underline(UnderlineMode::Double)?
            .bold(true)?
            .writeln(content.content_type.to_string().as_str())?;

        // source
        printer
            .line_spacing(0)?
            .reset_size()?
            .feed()?
            .justify(JustifyMode::RIGHT)?
            .underline(UnderlineMode::None)?
            .bold(false)?
            .writeln(format!("Source: {}", content.source).as_str())?;

        // date
        {
            let s = format!("Created: {}", content.date.format(date_format));
            printer.feed()?.writeln(s.as_str())?;
        }

        // due
        if let Some(due) = &content.due {
            let s = format!("Due: {}", due.format(date_format));
            printer
                .size(1, 2)?
                .feed()?
                .bold(true)?
                .writeln(s.as_str())?;
        }

        // dest
        printer.justify(JustifyMode::LEFT)?.bold(false)?;
        if let Some(dest) = &content.dest {
            let s = format!("To: {dest}");
            printer.reset_size()?.feed()?.writeln(s.as_str())?;
        }

        // content
        printer
            .reset_line_spacing()?
            .size(1, 2)?
            .feed()?
            .writeln(content.content.as_str())?;

        printer.print_cut()?;

        Ok(())
    }
}
