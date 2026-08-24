mod args;
mod config;
mod content;
mod destinations;
mod sources;

use crate::args::Args;
use crate::config::Config;
use crate::content::Content;
use crate::destinations::Destinations;
use crate::sources::Sources;

use clap::Parser;
use inquire::Confirm;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "warn");
        }
    }
    pretty_env_logger::init();
    info!("Starting Tary");

    let args = Args::parse();

    if args.create_config {
        let ans = Confirm::new(
            "Creating a new config will overwrite any existing configuration. Continue?",
        )
        .with_default(false)
        .prompt();

        match ans {
            Ok(true) => {
                if let Err(e) = Config::create_default_config() {
                    println!("Unable to save default configuration: {e}");
                }
            }
            Ok(false) => println!("Configuration NOT overwritten."),
            Err(e) => println!("{e}"),
        }
        return Ok(());
    }

    let cfg = Arc::new(Config::load_or_default());

    let sources = Sources::new(cfg.clone());
    let destinations = Destinations::new(cfg.clone());

    {
        let sc = sources.count();
        let dc = destinations.count();
        if sc == 0 || dc == 0 {
            error!(
                "You have {sc} sources and {dc} destinations enabled! You must have at least one of each."
            );
            std::process::exit(1);
        }
    }

    let (tx, _) = broadcast::channel::<Content>(64); // absurd 64
    tokio::join!(sources.start(tx.clone()), destinations.start(tx));
    Ok(())
}
