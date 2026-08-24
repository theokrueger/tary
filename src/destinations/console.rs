use crate::config::Config;
use crate::content::Content;
use crate::destinations::TaryDestination;
use log::{error, trace};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;

pub struct Console {
    dest: Option<PathBuf>,
}

impl TaryDestination for Console {
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        if let Some(c) = &cfg.destinations.console
            && c.enabled
        {
            let dest = c.output.as_ref().map(PathBuf::from);
            Ok(Some(Box::new(Self { dest })))
        } else {
            Ok(None)
        }
    }

    async fn listen(self, mut rx: Receiver<Content>) {
        let file_dest = self.dest;
        loop {
            let content = match rx.recv().await {
                Ok(c) => c,
                Err(_) => break,
            };
            trace!("Console dest received content");
            let txt = format!(
                "[{ctype}] Date: {date}\nFrom: {from}\nDue: {due}\nDestination: {dest} \n\t{text}",
                ctype = content.content_type,
                date = content.date,
                from = content.source,
                due = content
                    .due
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "Unspecified".into()),
                dest = content.dest.unwrap_or_else(|| "Unspecified".into()),
                text = content.content
            );
            if let Some(path) = &file_dest {
                match File::options().append(true).create(true).open(path) {
                    Ok(mut f) => {
                        let _ = writeln!(&mut f, "{txt}");
                    }
                    Err(e) => error!("Failed to open log file {path:?}: {e}"),
                }
            } else {
                println!("{txt}");
            }
        }
    }
}
