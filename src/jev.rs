//! jev.rs
//!
//! support for jev-style decision models for various content management purposes.
use crate::config::JevProvider;
use crate::{config::Config, content::Content};

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast::Sender;

#[derive(Serialize, Deserialize, Debug)]
pub enum JevQuestionType {
    #[serde(rename = "choice")]
    Choice,
    #[serde(rename = "score")]
    Score,
    #[serde(rename = "noul")]
    Noul,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum JevCriteria {
    Choice(HashMap<String, String>),
    Score(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JevQuestion {
    #[serde(rename = "type")]
    pub qtype: JevQuestionType,
    pub instructions: String,
    /// req for type::Score and type::Choice
    pub criteria: Option<JevCriteria>,
}

impl JevQuestion {
    pub fn choice(instructions: String, criteria: JevCriteria) -> Self {
        Self {
            qtype: JevQuestionType::Choice,
            instructions,
            criteria: Some(criteria),
        }
    }

    pub fn score(instructions: String, criteria: JevCriteria) -> Self {
        Self {
            qtype: JevQuestionType::Score,
            instructions,
            criteria: Some(criteria),
        }
    }

    pub fn noul(instructions: String) -> Self {
        Self {
            qtype: JevQuestionType::Noul,
            instructions,
            criteria: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JevRequest {
    pub state: String,
    pub questions: HashMap<String, JevQuestion>,
}

impl JevRequest {
    pub fn state(s: String) -> Self {
        Self {
            state: s,
            questions: HashMap::new(),
        }
    }

    pub fn add_question(mut self, id: String, q: JevQuestion) -> Self {
        self.questions.insert(id, q);
        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JevAnswer {
    #[serde(rename = "type")]
    qtype: JevQuestionType,
    /// req on type::Choice
    choice: Option<String>,
    /// req on type::Score
    score: Option<f32>,
    /// req on type::Noul
    noul: Option<f32>,
    /// noreq on type::Noul
    confidence: Option<f32>,
    // toss probabilities
    // toss legend
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JevResponse {
    //model: String,
    answers: HashMap<String, JevAnswer>,
    //usage: HashMap<String, u32>,
    //latency_ms: f32,
}

#[derive(Debug)]
pub enum JevError {
    Disabled,
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

        println!("{:?}", serde_json::to_string(&req));

        let resp = self.client.post(&self.uri).json(&req).send().await;

        // println!("{:?}", resp.unwrap().json::<JevResponse>().await.unwrap());
        //println!("{:?}", resp.unwrap().text().await);

        Ok(resp.unwrap().json::<JevResponse>().await.unwrap())
    }
}
