// only implements episode scrapping, as it seem the only problem with the "webtoon" crate
pub mod episodes;
pub mod image_dl;
pub mod recommandations;
pub mod webtoon;

use serde::{Deserialize, Serialize};

/* Type Definition */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WebtoonId {
    pub wt_id: usize,
    pub wt_type: WtType,
}

impl WebtoonId {
    pub fn new(wt_id: usize, wt_type: WtType) -> Self {
        Self { wt_id, wt_type }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WtType {
    Canvas,
    Original,
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

impl From<String> for Genre {
    fn from(raw_genre: String) -> Self {
        match serde_json::from_str::<Genre>(&raw_genre) {
            Ok(g) => g,
            Err(_) => Self::Other(raw_genre),
        }
    }
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

/// Represents a kind of release schedule for Originals.
///
/// For the days of the week, a Webtoon can have multiple.
///
/// If its not a day of the week, it can only be either `Daily` or `Completed`, alone.
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

impl TryFrom<String> for Schedule {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let raw_schedule = value.to_lowercase();
        match raw_schedule.as_str() {
            "completed" => Ok(Self::Completed),
            "daily" => Ok(Self::Daily),
            _ => {
                let weekdays = raw_schedule
                    .replace("every ", "")
                    .split(", ")
                    .map(|s| match s {
                        "mon" => Ok(Weekday::Monday),
                        "tue" => Ok(Weekday::Tuesday),
                        "wed" => Ok(Weekday::Wednesday),
                        "thu" => Ok(Weekday::Thursday),
                        "fri" => Ok(Weekday::Friday),
                        "sat" => Ok(Weekday::Saturday),
                        "sun" => Ok(Weekday::Sunday),
                        _ => Err("Failed to parse weekday".to_string()),
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                match weekdays.as_slice() {
                    &[single] => Ok(Self::Weekday(single)),
                    &[] => Err("no matches".to_string()),
                    multiples => Ok(Self::Weekdays(multiples.to_vec())),
                }
            }
        }
    }
}

/* HELPERS */
pub(crate) fn generate_webtoon_url(id: WebtoonId) -> String {
    match id.wt_type {
        WtType::Canvas => format!(
            "https://www.webtoons.com/en/canvas/*/list?title_no={}",
            id.wt_id,
        ),
        WtType::Original => {
            format!("https://www.webtoons.com/en/*/*/list?title_no={}", id.wt_id)
        }
    }
}
