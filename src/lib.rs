#![feature(iter_intersperse)]

use gargoyle::{Monitor, Action};
use feed_rs::model::Entry;
use chrono::{DateTime, Utc};
use bytes::{Buf, Bytes};

pub use reqwest::blocking::Client;

pub struct WebFeedUpdate {
    pub url: String,
    web_client: Client,
    last_updated: Option<DateTime<Utc>>,
}

impl Monitor for WebFeedUpdate {
    fn check(&mut self) -> Action {
        log::info!("Checking feed {}", self.url);

        let feed_bytes = self.get_feed_bytes();
        if feed_bytes.is_none() {
            return Action::Nothing;
        }
        let feed_bytes = feed_bytes.unwrap();

        self.parse_and_handle_feed(feed_bytes)
    }
}

impl WebFeedUpdate {
    pub fn new(url: &str) -> Self {
        let web_client = Client::builder()
            .user_agent("Gargoyle/0.1")
            .build()
            .unwrap();
        Self {
            url: url.to_string(),
            web_client,
            last_updated: None,
        }
    }

    pub fn with_user_agent(url: &str, user_agent: &str) -> Self {
        let web_client = Client::builder()
            .user_agent(user_agent)
            .build()
            .unwrap();
        Self {
            url: url.to_string(),
            web_client,
            last_updated: None,
        }
    }

    pub fn with_client(url: &str, web_client: Client) -> Self {
        Self {
            url: url.to_string(),
            web_client,
            last_updated: None,
        }
    }
}

impl WebFeedUpdate {
    fn get_feed_bytes(&self) -> Option<Bytes> {
        let response = match self.web_client.get(&self.url).send() {
            Ok(response) => response,
            Err(e) => {
                log::error!("{e}");
                return None;
            },
        };

        match response.bytes() {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                log::error!("{e}");
                None
            }
        }
    }

    fn parse_and_handle_feed(&mut self, bytes: Bytes) -> Action {
        let parser = feed_rs::parser::Builder::new()
            .base_uri(Some(self.url.clone()))
            .build();

        let feed = match parser.parse(bytes.reader()) {
            Ok(feed) => feed,
            Err(e) => {
                log::error!("{e}");
                return Action::Nothing;
            }
        };

        if feed.updated.is_none() {
            log::error!("No updated field");
            return Action::Nothing;
        }

        if self.last_updated.is_none() {
            self.last_updated = feed.updated;
            log::info!("Initial check on {}", self.url);
            return Action::Nothing;
        }

        let last_updated = self.last_updated.unwrap();

        if last_updated == feed.updated.unwrap() {
            log::info!("Nothing updated on {} since {}", self.url, last_updated);
            return Action::Nothing;
        }

        self.last_updated = feed.updated;

        let new_entries = feed.entries.iter()
            .filter(|entry| entry.updated.is_some())
            .filter(|entry| entry.updated.unwrap() > last_updated);

        let new_entries: Vec<&Entry> = new_entries.collect();
        if new_entries.is_empty() {
            log::info!("No new feed entries on {} since {}", self.url, last_updated);
            return Action::Nothing;
        }

        log::info!("Found {} new entries for {}", new_entries.len(), self.url);

        let mut message = String::from("New entries in feed:\r\n");
        for entry in new_entries {
            let links: String = entry.links.iter()
                .map(|link| link.href.clone())
                .intersperse(", ".to_string())
                .collect();
            if let Some(ref title) = entry.title {
                message.push_str(&format!("    - {} ({})\r\n", title.content, links));
            } else {
                message.push_str(&format!("    - {}\r\n", links));
            }
        }

        Action::Update { message: Some(message) }
    }
}

