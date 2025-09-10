//! Manage loading of the config file

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Write},
};
use teloxide::types::UserId;

const CONFIG_DIR: &str = "tary";
const CONFIG_FILE: &str = "tary.toml";

const DEFAULT_CONFIG: &str = r#"# Default tary config.
# Reset your configuration by running `tary --create-config`

## General settings
[general]
# Your username
name = "Jane Doe"

## (Optional) Telegram settings
[sources.telegram]
# use Telegram input?
enabled = true

# User allowed to send messages to the bot
# Get your user id by messaging @userinfobot
user = 1234

## (Optional) Console destination
[destinations.console]
# use console output?
enabled = true

## (Optional) POS Printer destination
[destinations.pos_printer]
# use POS (receipt) printer output?
enabled = true

# Printer connection type
# Options: USB
connection = "USB"

# (Optional, required for USB connection type)
# USB PID/VID for USB communication
# Example for Seiko Epson Corp. TM-T20II
# Ensure you have access to writing this USB device. You may need to make a new udev rule.
usb_vid = 0x04b8
usb_pid = 0x0e15

## Ollama settings
[ollama]
# Address to connect to Ollama server at
address = "127.0.0.1"

# Port to connect to Ollama server on
port = 11434

# What model to run. Must be downloaded already via Ollama.
# The bigger the model your host can run, the better.
# Response time is not crucial, use the best model you can fit in your VRAM
model = "gemma3:4b"

# (Optional) The system prompt to be given to the model.
# Your username and the model's 'Name' will always be prepended to the system prompt
system_prompt = '''
Your directives are as follows:
- Summarise in 20 words or less.
- Use as short a sentence as is possible.
- You never fail to summarise.
'''

"#;

const MINIMAL_CONFIG: &str = r#"# Minimal tary config.
# This is the bare minimum config to get tary running.
# No sources or destinations are enabled, so the program does basically nothing.
# Reset this config by running `tary --create-minimal-config`
[general]
name = "Jane Doe"

[ollama]
address = "127.0.0.1"
port = 11434
model = "gemma3:4b"
"#;

extern crate dirs;

/// Root of config
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub sources: SourcesConfig,
    pub destinations: DestinationsConfig,
    pub ollama: OllamaConfig,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralConfig {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct SourcesConfig {
    pub telegram: Option<TelegramSourceConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct TelegramSourceConfig {
    pub enabled: bool,
    pub user: Option<UserId>,
}

#[derive(Serialize, Deserialize)]
pub struct DestinationsConfig {
    pub console: Option<ConsoleDestConfig>,
    pub pos_printer: Option<PosDestConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct ConsoleDestConfig {
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PosConnectionTypes {
    USB,
}

#[derive(Serialize, Deserialize)]
pub struct PosDestConfig {
    pub enabled: bool,
    pub connection: PosConnectionTypes,
    pub usb_pid: Option<u16>,
    pub usb_vid: Option<u16>,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaConfig {
    pub model: String,
    pub system_prompt: Option<String>,
    pub address: String,
    pub port: u16,
}

impl Config {
    fn default() -> Self {
        toml::from_str(&DEFAULT_CONFIG).unwrap()
    }

    /// load from CONFIG_FILE, use defaults if failure
    pub fn load_or_default() -> Self {
        let try_load = || -> Result<Self, Error> {
            let mut path = dirs::config_dir().unwrap();
            path.push(CONFIG_DIR);
            path.push(CONFIG_FILE);
            info!("Loading config from {}", path.display());
            let s: String = fs::read_to_string(path)?;
            let cfg: Config = toml::from_str(&s).or_else(|e| -> Result<Self, Error> {
                Err(Error::new(
                    ErrorKind::Other,
                    format!("Unable to deserialize config: {e}"),
                ))
            })?;
            Ok(cfg)
        };

        match try_load() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed loading from config file: {e}\nUsing default settings.");
                let cfg = Self::default();
                info!("Using config:\n{}", toml::to_string(&cfg).unwrap());
                cfg
            }
        }
    }

    pub fn create_default_config() -> Result<(), Error> {
        let mut path = dirs::config_dir().unwrap();
        path.push(CONFIG_DIR);
        fs::create_dir_all(path.clone())?;
        path.push(CONFIG_FILE);
        println!("Creating default config at '{}'", path.display());
        let mut file = File::create(path)?;
        file.write_all(DEFAULT_CONFIG.as_bytes())?;
        Ok(())
    }

    pub fn create_minimal_config() -> Result<(), Error> {
        let mut path = dirs::config_dir().unwrap();
        path.push(CONFIG_DIR);
        fs::create_dir_all(path.clone())?;
        path.push(CONFIG_FILE);
        println!("Creating minimal config at '{}'", path.display());
        let mut file = File::create(path)?;
        file.write_all(MINIMAL_CONFIG.as_bytes())?;
        Ok(())
    }
}
