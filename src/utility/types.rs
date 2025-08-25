use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AlertLevel {
    Success,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: usize,
    pub msg: String,
    pub level: AlertLevel,
    pub duration: Duration,
}
