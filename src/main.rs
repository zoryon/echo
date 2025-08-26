mod models;
mod routes;
mod handlers;
mod db;
mod schema;
mod token_utils;

use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use diesel::r2d2::{self, ConnectionManager};
use diesel::MysqlConnection;

#[actix_web::get("/")]
async fn index(_req: HttpRequest) -> impl Responder {
    format!("Welcome!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = 8080;
    println!("Starting server on port {port}");

    // Setup DB pool from DATABASE_URL env
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "mysql://root:password@127.0.0.1/echo".to_string());
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(8)
        .build(manager)
        .expect("Failed to create DB pool");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set in .env")
        .into_bytes();

     let secret_data = web::Data::new(jwt_secret);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(secret_data.clone())
            .service(index)
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", port))?
    .workers(1)
    .run()
    .await
}