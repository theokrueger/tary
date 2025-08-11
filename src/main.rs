extern crate pretty_env_logger;

#[macro_use]
extern crate log;

mod config;
use crate::config::Config;

mod args;
use crate::args::Args;

mod tary_llm;
use crate::tary_llm::TaryLLM;

use clap::Parser;

use std::sync::Arc;

use tokio::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    trace!("Starting Tary");

    let args = Args::parse();

    if args.create_config {
        Config::create_default_config().unwrap_or_else(|e| {
            println!("Unable to save default configuration: {e}");
        });
        return Ok(());
    }

    let cfg = Arc::new(Config::load_or_default());

    // ollama test
    let tary = TaryLLM::new(cfg.clone()).await;

    println!(
        "{}",
        tary.no_context_prompt("big ol balls".to_string())
            .await
            .response
    );

    Ok(())
}
