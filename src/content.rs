//! content created by sources and delivered to destinations
use chrono::prelude::*;

#[derive(Clone, Debug)]
pub struct Content {
    pub source: String,
    pub dest: Option<String>,
    pub date: DateTime<Local>,
    pub due: Option<DateTime<Local>>,
    pub content: String,
}

impl Content {
    pub fn new(src: String, date: Option<DateTime<Local>>, content: String) -> Self {
        let d: DateTime<Local> = match date {
            Some(arg) => arg,
            None => Local::now(),
        };

        Self {
            source: src,
            dest: None,
            date: d,
            due: None,
            content: content,
        }
    }

    pub fn dest(mut self, s: String) -> Self {
        self.dest = Some(s);
        self
    }
    pub fn due(mut self, d: DateTime<Local>) -> Self {
        self.due = Some(d);
        self
    }
}
