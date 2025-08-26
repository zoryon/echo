use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use diesel::MysqlConnection;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

/// Custom error type for DB connection issues
#[derive(Debug)]
pub struct DbError;

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database connection error")
    }
}

impl ResponseError for DbError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().body("Database connection error")
    }
}

/// Helper function to get a pooled DB connection
pub fn get_conn(pool: &DbPool) -> Result<PooledConnection<ConnectionManager<MysqlConnection>>, DbError> {
    pool.get().map_err(|_| DbError)
}
