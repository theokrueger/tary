//! Manage loading of the config file

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Write},
};

const CONFIG_DIR: &str = "tary";
const CONFIG_FILE: &str = "tary.toml";

const DEFAULT_CONFIG: &str = r#"# Default tary config.
# Reset this configuration by running `tary --create-config`

## General settings
[general]
# Your username
name = "Jane Doe"

## (Optional) Console destination
[destinations.console]
# use console output?
enabled = true

## (Optional) POS Printer destination
[destinations.pos_printer]
# use POS (receipt) printer output?
enabled = false

# Printer connection type
# Options: USB
connection = "USB"

# (Optional, required for USB connection type)
# USB PID/VID for USB communication
# Example for Seiko Epson Corp. TM-T20II
# Ensure you have access to writing this USB device. You may need to make a new udev rule.
# A la `SUBSYSTEM=="usb", ATTR{idVendor}=="0fe6", ATTR{idProduct}=="811e", MODE="0666"`
usb_vid = 0x0fe6
usb_pid = 0x811e
"#;

const MINIMAL_CONFIG: &str = r#"# Minimal tary config.
# This is the bare minimum config to get tary running.
# No sources or destinations are enabled, so the program does basically nothing.
# Reset this config by running `tary --create-minimal-config`
[general]
name = "Jane Doe"
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
pub struct SourcesConfig {}

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
    pub usb_vid: Option<u16>,
    pub usb_pid: Option<u16>,
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
