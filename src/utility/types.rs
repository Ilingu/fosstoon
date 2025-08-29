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

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Original => "Originals",
                Type::Canvas => "Canvas",
            }
        )
    }
}

#[derive(Serialize, Clone, Copy, Deserialize, Debug)]
pub struct WebtoonId {
    pub wt_id: u32,
    pub wt_type: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebtoonSearchInfo {
    pub id: WebtoonId,
    pub title: String,
    pub thumbnail: String,
    pub creator: String,
}

/// Represents the languages that `webtoons.com` has.
#[derive(
    Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Language {
    /// English
    #[default]
    En,
    /// Chinese
    Zh,
    /// Thai
    Th,
    /// Indonesian
    Id,
    /// Spanish
    Es,
    /// French
    Fr,
    /// German
    De,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebtoonInfo {
    pub id: WebtoonId,

    pub title: String,
    pub thumbnail: Option<String>,
    pub language: Language,
    pub banner: Option<String>,
    pub creators: Vec<String>,
    pub genres: Vec<Genre>,
    pub schedule: Option<Schedule>,
    pub is_completed: bool,
    pub views: u64,
    pub likes: u32,
    pub subs: u32,
    pub summary: String,

    pub episodes: Option<Vec<EpisodePreview>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodePreview {
    pub parent_wt_id: WebtoonId,

    pub number: usize,
    pub title: String,
    pub thumbnail: String,
    pub likes: usize,
    pub posted_at: String,
    pub ep_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Schedule {
    /// Released on a single day of the week
    Weekday(Weekday),
    /// Released multiple days of the week
    Weekdays(Vec<Weekday>),
    /// Released daily
    Daily,
    /// Webtoon is completed
    Completed,
}

/// Represents a day of the week
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Weekday {
    /// Released on Sunday
    Sunday,
    /// Released on Monday
    Monday,
    /// Released on Tuesday
    Tuesday,
    /// Released on Wednesday
    Wednesday,
    /// Released on Thursday
    Thursday,
    /// Released on Friday
    Friday,
    /// Released on Saturday
    Saturday,
}

#[allow(clippy::upper_case_acronyms)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub enum Genre {
    Comedy,
    Fantasy,
    Romance,
    SliceOfLife,
    SciFi,
    Drama,
    ShortStory,
    Action,
    Superhero,
    Heartwarming,
    Thriller,
    Horror,
    PostApocalyptic,
    Zombies,
    School,
    Supernatural,
    Animals,
    Mystery,
    Historical,
    /// Tiptoon
    Informative,
    Sports,
    Inspirational,
    AllAges,
    LGBTQ,
    RomanticFantasy,
    MartialArts,
    WesternPalace,
    EasternPalace,
    MatureRomance,
    /// Reincarnation/Time-travel
    TimeSlip,
    Local,
    /// Modern/Workplace
    CityOffice,
    Adaptation,
    Shonen,
    WebNovel,
    GraphicNovel,
}
