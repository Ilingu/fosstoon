use std::{fmt::Display, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AlertLevel {
    Success,
    Info,
    Warning,
    Error,
}

impl Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: usize,
    pub msg: String,
    pub level: AlertLevel,
    pub duration: Duration,
}

impl Alert {
    pub fn new(msg: String, level: AlertLevel, duration: Option<Duration>) -> Self {
        Self {
            id: 0,
            msg,
            level,
            duration: duration.unwrap_or(Duration::from_secs(3)),
        }
    }
}

/* BACKEND TYPES */

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// An Original webtoon.
    Original,
    /// A Canvas webtoon.
    Canvas,
}

#[derive(Serialize, Clone, Copy, Deserialize, Debug)]
pub struct WebtoonId {
    pub wt_id: u32,
    pub wt_type: Type,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebtoonSearchInfo {
    id: WebtoonId,

    title: String,
    thumbnail: String,
    creator: String,
}
