//! Secure secret storage
use log::{error, info};
use securestore::{KeySource, SecretsManager};
use std::{path::PathBuf, sync::Arc};

use crate::config::Config;

const STORAGE_FOLDER: &str = "tary/";
const SECRETS_FILE: &str = "secrets.json";
const SECRETS_KEY: &str = "secrets.key";

pub struct Storage {
    folder: PathBuf,
    secrets: SecretsManager,
}

impl Storage {
    pub fn new(_cfg: Arc<Config>) -> Self {
        let mut storage_folder = dirs::data_dir().unwrap();
        storage_folder.push(STORAGE_FOLDER);

        Self {
            folder: storage_folder.clone(),
            secrets: Self::load_secrets(storage_folder),
        }
    }

    fn load_secrets(path: PathBuf) -> SecretsManager {
        let mut sfile = path.clone();
        sfile.push(SECRETS_FILE);

        let mut kfile = path.clone();
        kfile.push(SECRETS_KEY);

        info!(
            "Loading secrets from {} with key {}",
            sfile.display(),
            kfile.display()
        );

        match SecretsManager::load(sfile, KeySource::Path(&kfile)) {
            Ok(sman) => sman,
            Err(e) => {
                error!("Unable to load secrets: {e}");
                std::process::exit(1);
            }
        }
    }

    pub fn get_secret(&self, key: &str) -> Result<String, securestore::Error> {
        self.secrets.get(key)
    }
}
