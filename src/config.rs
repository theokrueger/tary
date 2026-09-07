//! Manage loading of the config file

use log::{error, trace};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Error, Write},
    path::PathBuf,
};

const CONFIG_DIR: &str = "tary";
const CONFIG_FILE: &str = "config.toml";

const DEFAULT_CONFIG: &str = r#"# Default tary config.
# Reset this configuration by running `tary --create-config`
#
# For paths, absolute paths are preferred but relative paths
# are relaltive to the config directory

### General Config ###
[general]
# Your username.
name = "Jane Doe"


### Sources ###
# All sources are optional, but at least one must be enabled

## HTTP server source
[sources.http_server]
enabled = true
# bind address
host = "127.0.0.1"
# http bind port
port = 3000

# (Optional) HTTPS support
# All four options must be specified if https_enabled is true
#https_enabled = false
#https_port = 3001
#https_cert = "/path/to/cert"
#https_key = "/path/to/key"


# (Optional) path to a custom HTML file served on '/'.
# If not specified, a default HTML page is served.
# html_path = "/path/to/index.html"


### Middlewares ###
# All middlewares are optional
# some may transform content before it gets forwarded to destinations
# others may simply perform inferencing on the content data

# All middlewares have some way to define their order in the pipeline
# This is generally set per-middleware with the optional `order` parameter.
# Without specifying order, a default order will be used.

## regex middleware
# applies a list of regexes sequentially to incoming content
[middlewares.regex]
enabled = true

# (Optional) order of this middleware in the pipeline
order = 10

# List of regex rules to apply
# Each rule is formatted as [pattern: string, replacement: string, global: bool]
# Full pattern/replacement syntax: https://docs.rs/regex/latest/regex/#syntax
regexes = [
    # surround first word with style
    ["(?<group>[[:alpha:]]+)", "xX_$group_Xx", false],
    # improve program morale
    ["tary is the worst", "tary is the best", true],
  ]

### Destinations ###
# All destinations are optional, but at least one must be enabled

## Console destination
[destinations.console]
enabled = true
# (Optional) where to output to? (do not specify for stdout)
#output = "/tmp/tary.log"

## POS Printer (reciept printer) destination
[destinations.pos_printer]
enabled = false

# Printer connection type: "USB"
connection = "USB"

# (Optional, required for connection = "USB")
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
    #[serde(default)]
    pub middlewares: MiddlewaresConfig,
    #[serde(default)]
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
    pub https_enabled: Option<bool>,
    pub https_port: Option<u16>,
    pub https_cert: Option<String>,
    pub https_key: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct MiddlewaresConfig {
    pub regex: Option<RegexMiddleware>,
}

#[derive(Serialize, Deserialize)]
pub struct RegexMiddleware {
    pub enabled: bool,
    #[serde(default)]
    pub order: u32,
    pub regexes: Vec<(String, String, Option<bool>)>,
}

#[derive(Serialize, Deserialize, Default)]
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
    pub fn config_dir() -> PathBuf {
        let mut path = dirs::config_dir().unwrap();
        path.push(CONFIG_DIR);
        path
    }

    fn default_config() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap()
    }

    /// load from CONFIG_FILE, use defaults if failure
    pub fn load_or_default() -> Result<Self, Error> {
        let try_load = || -> Result<Self, Error> {
            let mut path = Self::config_dir();
            path.push(CONFIG_FILE);
            trace!("Loading config from {}", path.display());
            let s = fs::read_to_string(path)?;
            toml::from_str(&s)
                .map_err(|e| Error::other(format!("Unable to deserialize config: {e}")))
        };

        let cfg = try_load().unwrap_or_else(|e| {
            error!("Failed loading from config file: {e}\nUsing default settings.");
            Self::default_config()
        });

        // sanity checks
        let mut fail = false;

        // ensure all https optional args are presented when secure server is enabled, allowing unwrap later :)
        if let Some(ref a) = cfg.sources.http_server
            && let Some(b) = a.https_enabled
            && b
        {
            [
                (a.https_port.is_some(), "https_port"),
                (a.https_cert.is_some(), "https_cert"),
                (a.https_key.is_some(), "https_key"),
            ]
            .iter()
            .for_each(|(b, s)| {
                if !b {
                    error!(
                        "HTTPS is enabled for the HTTP Server source, yet {s} is not specified!"
                    );
                    fail = true;
                }
            });
        }

        if fail {
            Err(Error::other(
                "Sanity check of config failed! Review error log for suggestions.",
            ))
        } else {
            Ok(cfg)
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
