use crate::Config;
use crate::content::Content;
use crate::destinations::TaryDestination;
use log::trace;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;

pub struct Console {}

impl TaryDestination for Console {
    fn init(cfg: Arc<Config>) -> Result<Option<Box<Self>>, Box<dyn Error>> {
        if let Some(c) = &cfg.destinations.console
            && c.enabled
        {
            Ok(Some(Box::new(Self {})))
        } else {
            Ok(None)
        }
    }

    async fn listen(self, mut rx: Receiver<Content>) {
        loop {
            let content = rx.recv().await.unwrap();
            trace!("Console dest received content");
            println!(
                "[{ctype}] Date: {date}\nFrom: {from}\nDue: {due}\nDestination: {dest} \n\t{text}",
                ctype = content.content_type,
                date = content.date,
                from = content.source,
                due = content
                    .due
                    .map(|d| d.to_string())
                    .unwrap_or("Unspecified".into()),
                dest = content.dest.unwrap_or("Unspecified".into()),
                text = content.content
            );
        }
    }
}

impl Console {}
