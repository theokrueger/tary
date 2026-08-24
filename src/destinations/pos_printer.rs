use crate::{
    Content,
    config::{Config, PosConnectionTypes as Pct},
    destinations::TaryDestination,
};
use escpos::{driver::*, printer::Printer, printer_options::PrinterOptions, utils::*};
use log::{error, info, trace};
use std::{error::Error, sync::Arc};
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
            trace!("Setting up POS printer");

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
                    info!("Settig up USB POS Printer at [{:x}:{:x}]", vid, pid);
                    pos.init_usb()?;
                }
            }

            return Ok(Some(Box::new(pos)));
        }
        Ok(None)
    }

    async fn listen(self, mut rx: Receiver<Content>) {
        if let Err(e) = self.print_loop(&mut rx).await {
            error!("Error {e} in printing to pos_printer, aborting this print!");
        }
    }
}

impl PosPrinter {
    fn init_usb(&mut self) -> Result<(), Box<dyn Error>> {
        self.usb_driver = None;
        let (vid, pid) = self.usb_id.expect("No USB pid and vid specified");
        self.usb_driver = Some(Box::new(NativeUsbDriver::open(vid, pid)?));
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
            .real_time_status(RealTimeStatusRequest::Printer)?
            .real_time_status(RealTimeStatusRequest::RollPaperSensor)?;

        let date_format = "%a %Y-%m-%d %H:%M";

        loop {
            let Ok(content) = rx.recv().await else {
                error!("POS Printer failed to recieve content");
                break;
            };
            trace!("POS printer dest received content");
            let mut i = 0;
            while i < 3 {
                // reinit device if needed
                // TODO check to see if USB device needs to be reinitialized for some reason
                // TODO check to see if printer needs to be reinitialized if needed.

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
                if let Some(ref due) = content.due {
                    let s = format!("Due: {}", due.format(date_format));
                    printer
                        .size(1, 2)?
                        .feed()?
                        .bold(true)?
                        .writeln(s.as_str())?;
                }

                // dest
                printer.justify(JustifyMode::LEFT)?.bold(false)?;
                if let Some(ref dest) = content.dest {
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

                // check status
                //printer.send_status()?;
                //let mut buf = [0; 1];
                // TODO let driver be read
                //driver.read(&mut buf)?;
                //let status = RealTimeStatusResponse::parse(RealTimeStatusRequest::Printer, buf[0])?;
                // TODO check status for if paper needed and log and break if so.
                // TODO check status for if unrecoverable error, and log and  break if so
                // TODO check status for if recoverable error, recover if possible, then retry

                // retry
                break; // TODO remove this break when retry logic impletmented correctly.
                i += 1;
            }
        }

        Ok(())
    }
}
