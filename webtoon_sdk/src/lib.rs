// only implements episode scrapping, as it seem the only problem with the "webtoon" crate
pub mod episodes;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WebtoonId {
    pub wt_id: u32,
    pub wt_type: WtType,
}

impl WebtoonId {
    pub fn new(wt_id: u32, wt_type: WtType) -> Self {
        Self { wt_id, wt_type }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WtType {
    Canvas,
    Original,
}
