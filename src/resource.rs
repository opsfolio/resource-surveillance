use std::error::Error;

use chrono::{DateTime, Utc};

pub trait ResourceContent<T> {
    fn content_digest_hash(&self, target: T) -> &str;
    fn content_binary(&self, target: T) -> &Vec<u8>;
    fn content_text(&self, target: T) -> &str;
}

pub struct Resource<T> {
    pub uri: String,
    pub nature: Option<String>,
    pub size: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_modified_at: Option<DateTime<Utc>>,
    pub content: Option<Box<dyn Fn() -> Result<Box<dyn ResourceContent<T>>, Box<dyn Error>>>>,
}

pub enum ResourceSupplied<T> {
    Ignored,
    NotFound(String),
    NotFile(String),
    Resource(T),
    Error(Box<dyn Error>),
}

pub trait ResourceSupplier<Resource> {
    fn resource(&self, uri: &str) -> ResourceSupplied<Resource>;
}
