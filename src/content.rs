//! content created by sources and delivered to destinations
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug)]
pub enum ContentType {
    Todo,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Todo => f.write_str("TODO"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ContentPriority {
    Low,
    Normal,
    High,
}

impl fmt::Display for ContentPriority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Low => f.write_str("At Will"),
            Self::Normal => f.write_str("Normal"),
            Self::High => f.write_str("High"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Content {
    pub content_type: ContentType,
    pub content_priority: Option<ContentPriority>,
    pub source: String,
    pub dest: Option<String>,
    pub date: DateTime<Local>,
    pub due: Option<DateTime<Local>>,
    pub content: String,
}

impl Content {
    pub fn new(ctype: ContentType, src: String, date: Option<DateTime<Local>>) -> Self {
        Self {
            content_type: ctype,
            content_priority: None,
            source: src,
            dest: None,
            date: date.unwrap_or_else(Local::now),
            due: None,
            content: String::new(),
        }
    }
}
