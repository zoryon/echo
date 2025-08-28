mod models;
mod routes;
mod handlers;
mod db;
mod schema;
mod utils;
mod sftp;
mod middleware;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use diesel::r2d2::{self, ConnectionManager};
use diesel::MysqlConnection;

#[actix_web::get("/health")]
async fn health(_req: HttpRequest) -> impl Responder {
    format!("Ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let port: u16 = 8080;
    println!("Starting server on port {port}");

    // Setup DB pool from DATABASE_URL env
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
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
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(secret_data.clone())
            .wrap(middleware::session_middleware::SessionMiddlewareFactory)
            .service(health)
            .service(web::scope("/api").configure(routes::configure))
    })
    .bind(("0.0.0.0", port))?
    .workers(1)
    .run()
    .await
}