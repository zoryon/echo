use crate::models::pagination_models::{Pagination, PaginationError};

/// Validate pagination parameters
pub fn validate_pagination(p: &Pagination) -> Result<(i64, i64), PaginationError> {
    let limit = p.limit.unwrap_or(Pagination::DEFAULT_LIMIT);
    if limit > Pagination::MAX_LIMIT {
        return Err(PaginationError(format!(
            "Limit too high: maximum allowed is {}",
            Pagination::MAX_LIMIT
        )));
    }
    let offset = p.offset.unwrap_or(0);
    Ok((limit, offset))
}
