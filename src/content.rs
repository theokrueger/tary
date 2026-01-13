//! content created by sources and delivered to destinations
use chrono::prelude::*;
use std::fmt;

#[derive(Clone, Debug)]
pub enum ContentType {
    Todo,
}
impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ContentType::Todo => "TODO",
            }
        )
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
        let d: DateTime<Local> = match date {
            Some(arg) => arg,
            None => Local::now(),
        };

        Self {
            content_type: ctype,
            source: src,
            dest: None,
            date: d,
            due: None,
            content: "".to_string(),
        }
    }
}
