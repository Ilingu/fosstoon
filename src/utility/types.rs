use std::{fmt::Display, str::FromStr, time::Duration};

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
    pub fn new(msg: &str, level: AlertLevel, duration: Option<Duration>) -> Self {
        Self {
            id: 0,
            msg: msg.to_string(),
            level,
            duration: duration.unwrap_or(Duration::from_secs(3)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum DownloadState {
    WebtoonData(u8),
    EpisodeInfo(u8),
    CachingImages(u8),

    Idle,
    Completed,
}

impl DownloadState {
    pub fn get_progress(&self) -> u8 {
        *match self {
            Self::WebtoonData(p) | Self::CachingImages(p) | Self::EpisodeInfo(p) => p,
            _ => &0_u8,
        }
    }
    pub fn get_state(&self) -> String {
        match self {
            Self::WebtoonData(_) => "Fetching webtoon informations...",
            Self::EpisodeInfo(_) => "Fecthing episodes informations...",
            Self::CachingImages(_) => "Downloading images (thumbail|panels: may take a while)...",
            Self::Idle => "Currently not doing anything",
            Self::Completed => "Finished to download webtoon",
        }
        .to_string()
    }
}

/* BACKEND TYPES */

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WtType {
    /// An Original webtoon.
    Original,
    /// A Canvas webtoon.
    Canvas,
}

#[derive(Debug)]
pub struct WtTypeParseError(String);
impl Display for WtTypeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for WtTypeParseError {}

impl FromStr for WtType {
    type Err = WtTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "original" | "originals" => Ok(Self::Original),
            "canvas" => Ok(Self::Canvas),
            _ => Err(WtTypeParseError("non existing field".to_string())),
        }
    }
}

impl Display for WtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WtType::Original => "Originals",
                WtType::Canvas => "Canvas",
            }
        )
    }
}

#[derive(Serialize, Clone, Copy, Deserialize, Debug, PartialEq)]
pub struct WebtoonId {
    pub wt_id: usize,
    pub wt_type: WtType,
}

impl WebtoonId {
    pub fn new(id: usize, wt_type: WtType) -> Self {
        Self { wt_id: id, wt_type }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebtoonSearchInfo {
    pub id: WebtoonId,
    pub title: String,
    pub thumbnail: String,
    pub creator: Option<String>,
}

impl PartialEq for WebtoonSearchInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
    pub thumbnail: String,
    pub banner: Option<String>,
    pub creators: Vec<String>,
    pub genres: Vec<Genre>,
    pub schedule: Option<Schedule>,
    pub views: String,
    pub subs: String,
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

impl PartialEq for EpisodePreview {
    fn eq(&self, other: &Self) -> bool {
        self.parent_wt_id == other.parent_wt_id && self.number == other.number
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EpisodeData {
    pub parent_wt_id: WebtoonId,
    pub number: usize,

    pub panels: Vec<String>,
    pub author_note: Option<String>,
    pub author_name: String,
    pub author_thumb: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Post {
    pub wt_id: WebtoonId,
    pub ep_num: usize,

    pub id: String,
    pub content: String,
    pub is_spoiler: bool,
    pub upvotes: u32,
    pub downvotes: u32,
    pub posted_at: u64,
    pub poster_name: String,
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

impl Weekday {
    pub fn to_acronym(self) -> &'static str {
        match self {
            Weekday::Sunday => "Sun",
            Weekday::Monday => "Mon",
            Weekday::Tuesday => "Tue",
            Weekday::Wednesday => "Wed",
            Weekday::Thursday => "Thu",
            Weekday::Friday => "Fri",
            Weekday::Saturday => "Sat",
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq, Hash)]
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

    Other(String),
}

impl Display for Genre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(s) => write!(f, "{s}"),
            g => write!(f, "{g:?}"),
        }
    }
}
