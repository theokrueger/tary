//! Manage loading of the config file

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Write},
};
const CONFIG_FILE: &str = "tary.toml";

const DEFAULT_CONFIG: &str = r#"# Default tary config.
# Reset your configuration by running `tary --create-config`

## General settings
[general]
# Your username
name = "Jane Doe"

## (Optional) Telegram settings
[sources.telegram]
enabled = true

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

extern crate dirs;

/// Root of config
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub ollama: OllamaConfig,
    pub sources: Option<SourcesConfig>,
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
        path.push(CONFIG_FILE);
        println!("Creating default config at '{}'", path.display());
        let mut file = File::create(path)?;
        file.write_all(DEFAULT_CONFIG.as_bytes())?;
        Ok(())
    }
}
