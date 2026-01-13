extern crate pretty_env_logger;

#[macro_use]
extern crate log;

mod config;
use crate::config::Config;

mod args;
use crate::args::Args;

mod tary_llm;
use crate::tary_llm::TaryLLM;

mod content;
use content::Content;

mod sources;
use crate::sources::Sources;

mod destinations;
use crate::destinations::Destinations;

mod storage;
use crate::storage::Storage;

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
    let storage = Arc::new(Storage::new(cfg.clone()));

    let sources = Sources::new(cfg.clone(), storage.clone());
    let destinations = Destinations::new(cfg.clone(), storage.clone());

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

    // ollama test
    let tary = TaryLLM::new(cfg.clone()).await;
    //     let p = r#"
    // Please summarise the following email in 20 words or less:

    // [BEGIN EMAIL]

    // Dear Staff,
    // I want to inform you that one of our staff members, Mrs. Lilian Grant, is giving her late father's piano to a loving home for free. She is currently relocating to a smaller apartment she just purchased, and she is looking to give the piano out with no cost to a loving home and to someone who'll make use of the piano. It's a 2014 Yamaha Baby Grand, used like new.

    // You can show your interest by messaging her ASAP before someone else shows interest in the piano. Her contact info and pictures of the piano can be found in the attachments to this email.

    // Kind Regards,
    // Grace Falkner

    // [END EMAIL]
    // "#;
    //     println!("{}", tary.no_context_prompt(p.to_string()).await.response);

    let (tx, _) = broadcast::channel::<Content>(64); // absurd 64
    tokio::join!(sources.start(tx.clone()), destinations.start(tx));
    Ok(())
}
