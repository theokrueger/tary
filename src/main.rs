mod args;
mod config;
mod content;
mod destinations;
mod middlewares;
mod sources;

use crate::args::Args;
use crate::config::Config;
use crate::content::Content;
use crate::destinations::Destinations;
use crate::middlewares::Middlewares;
use crate::sources::Sources;

use clap::Parser;
use inquire::Confirm;
use log::{error, info, trace};
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "INFO");
        }
    }
    pretty_env_logger::init();
    trace!("Starting Tary");
    std::env::set_current_dir(Config::config_dir())?;

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

    let cfg = Arc::new(match Config::load_or_default() {
        Ok(c) => c,
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    });

    let sources = Sources::new(cfg.clone());
    let sc = sources.count();
    info!("Loaded {sc} sources");

    let middlewares = Middlewares::new(cfg.clone());
    let mc = middlewares.count();
    info!("Loaded {mc} middlewares");

    let destinations = Destinations::new(cfg.clone());
    let dc = destinations.count();
    info!("Loaded {dc} destinations");

    if sc == 0 || dc == 0 {
        error!(
            "You have {sc} sources and {dc} destinations enabled! You must have at least one of each."
        );
        std::process::exit(1);
    }

    info!("Starting tary");
    let (txin, _) = broadcast::channel::<Content>(4);
    let (txout, _) = broadcast::channel::<Content>(4);
    tokio::join!(
        sources.start(txin.clone()),
        middlewares.start(txin, txout.clone()),
        destinations.start(txout)
    );
    Ok(())
}
