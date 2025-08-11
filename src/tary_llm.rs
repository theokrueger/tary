//! ollama_rs wrapper for tary
use crate::config::{Config, OllamaConfig};

use ollama_rs::Ollama;
use ollama_rs::generation::completion::{GenerationResponse, request::GenerationRequest};

use std::sync::Arc;

use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;

/// Ollama wrapper for convenience
pub struct TaryLLM {
    ollama: Ollama,
    model: String,
}

impl TaryLLM {
    pub async fn new(cfg: Arc<Config>) -> Self {
        let ollama = Ollama::default();
        let model = cfg.ollama.model.clone();
        assert!(
            ollama
                .list_local_models()
                .await
                .unwrap()
                .into_iter()
                .map(|m| -> String { m.name })
                .collect::<Vec<String>>()
                .contains(&model),
            "Selected model '{model}' does not exist! Please ensure it is installed through Ollama."
        );
        TaryLLM {
            ollama: ollama,
            model: model,
        }
    }

    pub async fn no_context_prompt(self, prompt: String) -> GenerationResponse {
        self.ollama
            .generate(GenerationRequest::new(self.model, prompt).think(true))
            .await
            .unwrap()
    }
}
