//! Manage loading of the config file

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Write},
    path::Path,
};
use toml::Table;
const CONFIG_LOCATION: &str = "tary.toml";

const DEFAULT_CONFIG: &str = r#"# Default tary config.

## General settings
[general]
# Your username
name = "<Your Name Here>"

## Ollama settings
[ollama]
# What model to run. Must be downloaded already via Ollama.
# The bigger the model your host can run, the better.
# Response time is not crucial, 1TPS is adequate
# i.e. 'deepseek-r1:1.5b' produces decent results on a Raspberry Pi 4B 8G
model = "deepseek-r1:1.5b"

"#;

extern crate dirs;

/// Root of config
#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "general")]
    pub general: GeneralConfig,
    #[serde(rename = "ollama")]
    pub ollama: OllamaConfig,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralConfig {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaConfig {
    pub model: String,
}

impl Config {
    fn default() -> Self {
        toml::from_str(&DEFAULT_CONFIG).unwrap()
    }

    /// load from CONFIG_LOCATION, use defaults if failure
    pub fn load_or_default() -> Self {
        let try_load = || -> Result<Self, Error> {
            let mut path = dirs::config_dir().unwrap();
            path.push(CONFIG_LOCATION);
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
        path.push(CONFIG_LOCATION);
        println!("Creating default config at '{}'", path.display());
        let mut file = File::create(path)?;
        file.write_all(DEFAULT_CONFIG.as_bytes())?;
        Ok(())
    }
}
