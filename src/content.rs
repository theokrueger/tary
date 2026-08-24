//! content created by sources and delivered to destinations
use chrono::{DateTime, Local};
use std::fmt;

#[derive(Clone, Debug)]
pub enum ContentType {
    Todo,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContentType::Todo => f.write_str("TODO"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Content {
    pub content_type: ContentType,
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
            source: src,
            dest: None,
            date: date.unwrap_or_else(Local::now),
            due: None,
            content: String::new(),
        }
    }
}
