use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PageParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub cursor: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

impl PageParams {
    pub fn limit(&self) -> i64 {
        self.page_size.min(100) as i64
    }

    pub fn offset(&self) -> i64 {
        self.page
            .saturating_sub(1)
            .saturating_mul(self.page_size) as i64
    }
}

#[derive(Debug, Serialize)]
pub struct Page<T: Serialize> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total: Option<i64>,
    pub cursor: Option<String>,
}

impl<T: Serialize> Page<T> {
    pub fn new(items: Vec<T>, params: &PageParams, total: Option<i64>) -> Self {
        Self {
            items,
            page: params.page,
            page_size: params.page_size,
            total,
            cursor: params.cursor.clone(),
        }
    }
}
