//! jev.rs
//!
//! support for jev-style decision models for various content management purposes.
//! usage:
//! ```rust
//! use crate::jev::{JevQuestion, JevRequest};
//! let jr = JevRequest::state("Help! I am in great pain!").add_question(
//!     "pain",
//!     JevQuestion::noul("Is this a problem?"),
//! );
//! println!("{:?}", jev.request(jr).await);
//! ```

use crate::config::JevProvider;
use crate::{config::Config, content::Content};

use log::error;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast::Sender;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum JevQuestionType {
    #[serde(rename = "choice")]
    Choice,
    #[serde(rename = "score")]
    Score,
    #[serde(rename = "noul")]
    Noul,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JevCriteria {
    Choice(HashMap<String, String>),
    Score(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JevQuestion {
    #[serde(rename = "type")]
    pub qtype: JevQuestionType,
    pub instructions: String,
    /// req for type::Score and type::Choice
    pub criteria: Option<JevCriteria>,
}
impl JevQuestion {
    pub fn choice(instructions: &str, criteria: JevCriteria) -> Self {
        assert!(matches!(criteria, JevCriteria::Choice(_)));
        Self {
            qtype: JevQuestionType::Choice,
            instructions: instructions.to_string(),
            criteria: Some(criteria),
        }
    }

    pub fn score(instructions: &str, criteria: JevCriteria) -> Self {
        assert!(matches!(criteria, JevCriteria::Score(_)));
        Self {
            qtype: JevQuestionType::Score,
            instructions: instructions.to_string(),
            criteria: Some(criteria),
        }
    }

    pub fn noul(instructions: &str) -> Self {
        Self {
            qtype: JevQuestionType::Noul,
            instructions: instructions.to_string(),
            criteria: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JevRequest {
    pub state: String,
    pub questions: HashMap<String, JevQuestion>,
}

impl JevRequest {
    pub fn state(s: &str) -> Self {
        Self {
            state: s.to_string(),
            questions: HashMap::new(),
        }
    }

    pub fn add_question(mut self, id: &str, q: JevQuestion) -> Self {
        self.questions.insert(id.to_string(), q);
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JevAnswer {
    #[serde(rename = "type")]
    pub qtype: JevQuestionType,
    /// req on type::Choice
    pub choice: Option<String>,
    /// req on type::Score
    pub score: Option<f32>,
    /// req on type::Noul
    pub noul: Option<f32>,
    /// noreq on type::Noul
    pub confidence: Option<f32>,
    // toss probabilities
    // toss legend
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JevResponse {
    //model: String,
    pub answers: HashMap<String, JevAnswer>,
    //usage: HashMap<String, u32>,
    //latency_ms: f32,
}

#[derive(Debug)]
pub enum JevError {
    Disabled,
    RequestError(String),
}

impl From<reqwest::Error> for JevError {
    fn from(value: reqwest::Error) -> Self {
        error!("{value}");
        Self::RequestError(format!("{value}"))
    }
}

pub struct Jev {
    pub enabled: bool,
    client: reqwest::Client,
    uri: String,
}

impl Jev {
    pub fn new(cfg: Arc<Config>) -> Self {
        let prot = if cfg.jev.https_only.unwrap_or_default() {
            "https"
        } else {
            "http"
        };

        let host = cfg.jev.host.clone().unwrap_or_default();
        let port = cfg.jev.port.clone().unwrap_or_default();
        let api = "v1/systemone";

        Self {
            enabled: cfg.jev.enabled,
            client: reqwest::Client::new(),
            uri: format!("{prot}://{host}:{port}/{api}",),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub async fn request(&self, req: JevRequest) -> Result<JevResponse, JevError> {
        if !self.is_enabled() {
            return Err(JevError::Disabled);
        }

        Ok(self
            .client
            .post(&self.uri)
            .json(&req)
            .send()
            .await?
            .json::<JevResponse>()
            .await
            .unwrap())
    }
}
