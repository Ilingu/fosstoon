use std::{fmt::Display, time::Duration};

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
