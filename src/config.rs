//! Manage loading of the config file

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Error, Write},
};

const CONFIG_DIR: &str = "tary";
const CONFIG_FILE: &str = "tary.toml";

const DEFAULT_CONFIG: &str = r#"# Default tary config.
# Reset this configuration by running `tary --create-config`

## General settings
[general]
# Your username
name = "Jane Doe"

## (Optional) HTTP server source
[sources.http_server]
# use HTTP server as an input source?
enabled = true
# bind address
host = "127.0.0.1"
# bind port
port = 3000

# (Optional) path to a custom HTML file served on '/'.
# If not specified, a default HTML page is served.
# html_path = "/path/to/index.html"

## (Optional) Console destination
[destinations.console]
# use console output?
enabled = true
# (Optional) where to output to? (do not specify for stdout)
output = "/tmp/tary.log"

## (Optional) POS Printer destination
[destinations.pos_printer]
# use POS (receipt) printer output?
enabled = false

# Printer connection type: "USB"
connection = "USB"

# (Optional, required for USB connection type)
# USB PID/VID for USB communication
# Example for Seiko Epson Corp. TM-T20II
# Ensure you have access to writing this USB device. You may need to make a new udev rule.
# A la `SUBSYSTEM=="usb", ATTR{idVendor}=="0fe6", ATTR{idProduct}=="811e", MODE="0666"`
usb_vid = 0x0fe6
usb_pid = 0x811e
"#;

/// Root of config
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    #[serde(default)]
    pub sources: SourcesConfig,
    pub destinations: DestinationsConfig,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralConfig {
    pub name: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SourcesConfig {
    pub http_server: Option<HttpServerConfig>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct HttpServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub html_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DestinationsConfig {
    pub console: Option<ConsoleDestConfig>,
    pub pos_printer: Option<PosDestConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct ConsoleDestConfig {
    pub enabled: bool,
    pub output: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PosConnectionTypes {
    #[serde(rename = "USB")]
    Usb,
}

#[derive(Serialize, Deserialize)]
pub struct PosDestConfig {
    pub enabled: bool,
    pub connection: PosConnectionTypes,
    pub usb_vid: Option<u16>,
    pub usb_pid: Option<u16>,
}

impl Config {
    fn default_config() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap()
    }

    /// load from CONFIG_FILE, use defaults if failure
    pub fn load_or_default() -> Self {
        let try_load = || -> Result<Self, Error> {
            let mut path = dirs::config_dir().unwrap();
            path.push(CONFIG_DIR);
            path.push(CONFIG_FILE);
            info!("Loading config from {}", path.display());
            let s = fs::read_to_string(path)?;
            toml::from_str(&s)
                .map_err(|e| Error::other(format!("Unable to deserialize config: {e}")))
        };

        match try_load() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed loading from config file: {e}\nUsing default settings.");
                let cfg = Self::default_config();
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
}
