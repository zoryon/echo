use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Pagination {
    pub fn limit(&self) -> i64 { self.limit.unwrap_or(100) }
    pub fn offset(&self) -> i64 { self.offset.unwrap_or(0) }

    /// Aiuta a generare la parte LIMIT/OFFSET per raw SQL
    pub fn sql_clause(&self) -> String {
        format!("LIMIT {} OFFSET {}", self.limit(), self.offset())
    }
}