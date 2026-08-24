extern crate pretty_env_logger;

#[macro_use]
extern crate log;

mod config;
use crate::config::Config;

mod args;
use crate::args::Args;

mod content;
use content::Content;

mod sources;
use crate::sources::Sources;

mod destinations;
use crate::destinations::Destinations;

use clap::Parser;
use inquire::Confirm;
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    trace!("Starting Tary");

    let args = Args::parse();

    if args.create_config || args.create_minimal_config {
        let ans = Confirm::new(
            "Creating a new config will overwrite any existing configuration. Continue?",
        )
        .with_default(false)
        .prompt();

        match ans {
            Ok(true) => {
                if args.create_config {
                    Config::create_default_config().unwrap_or_else(|e| {
                        println!("Unable to save default configuration: {e}");
                    });
                } else if args.create_minimal_config {
                    Config::create_minimal_config().unwrap_or_else(|e| {
                        println!("Unable to save minimal configuration: {e}");
                    });
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
        if sc <= 0 || dc <= 0 {
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
