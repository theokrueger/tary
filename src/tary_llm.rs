//! ollama_rs wrapper for tary
use crate::config::Config;

use ollama_rs::Ollama;
use ollama_rs::generation::completion::{GenerationResponse, request::GenerationRequest};

use std::sync::Arc;

use log::{error, trace};

/// Ollama wrapper for convenience
pub struct TaryLLM {
    ollama: Ollama,
    model: String,
    /// Base context for all prompts
    system_context: String,
}

impl TaryLLM {
    pub async fn new(cfg: Arc<Config>) -> Self {
        let ollama = Ollama::new(
            format!("http://{}", cfg.ollama.address.clone()),
            cfg.ollama.port.clone(),
        );

        let model = cfg.ollama.model.clone();
        if !ollama
            .list_local_models()
            .await
            .unwrap_or_else(|e| {
                error!(
                    "Unable to establish connection to Ollama! Please ensure it is running ({e})."
                );
                std::process::exit(1);
            })
            .into_iter()
            .map(|m| -> String { m.name })
            .collect::<Vec<String>>()
            .contains(&model)
        {
            error!(
                "Selected model '{model}' does not exist! Please ensure it is installed through Ollama."
            );
            std::process::exit(1);
        };

        let mut context = format!(
            "You are Tary, the assistant to {name}. Your job is to summarise emails for {name}.\n",
            name = cfg.general.name.clone()
        );
        context.push_str(
            cfg.ollama
                .system_prompt
                .clone()
                .unwrap_or("".to_string())
                .as_str(),
        );

        TaryLLM {
            ollama: ollama,
            model: model,
            system_context: context,
        }
    }

    /// Prompt the model with only the system context
    pub async fn no_context_prompt(&self, prompt: String) -> GenerationResponse {
        let mut p = self.system_context.clone();
        p.push_str(prompt.as_str());

        trace!("Prompting model: {p}");
        self.ollama
            .generate(GenerationRequest::new(self.model.clone(), p))
            .await
            .unwrap()
    }
}
