use serde::Deserialize;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Errore di paginazione
#[derive(Debug)]
pub struct PaginationError(pub String);

impl fmt::Display for PaginationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for PaginationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().body(self.0.clone())
    }
}

impl Pagination {
    pub const DEFAULT_LIMIT: i64 = 100;
    pub const MAX_LIMIT: i64 = 100;

    pub fn limit(&self) -> Result<i64, PaginationError> {
        let l = self.limit.unwrap_or(Self::DEFAULT_LIMIT);
        if l > Self::MAX_LIMIT {
            return Err(PaginationError(format!(
                "Limit too high: maximum allowed is {}",
                Self::MAX_LIMIT
            )));
        }
        Ok(l)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0)
    }

    pub fn sql_clause(&self) -> Result<String, PaginationError> {
        Ok(format!("LIMIT {} OFFSET {}", self.limit()?, self.offset()))
    }
}
