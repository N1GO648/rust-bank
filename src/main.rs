// Import necessary items from external crates and internal modules to configure and run the web server.
// `actix_web` is used to build web applications and handle HTTP interactions.
// `dotenv` is used to load environment variables from a `.env` file.
// `std::env` is used for accessing environment variables.
// Internal module imports include `handlers` for routing, `models` for data structures, `auth` for authentication, and `db` for database operations.
//! Application entry point: configures and starts the Actix-Web server.
//!
//! 1. Loads environment variables via `dotenv`.
//! 2. Initializes the database (runs migrations).
//! 3. Reads server bind address from env.
//! 4. Configures routes and launches the HTTP server.

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;

mod db;    // DB initialization and migrations
mod auth;  // JWT auth helpers
mod models;// Data models and FromRow derivations
mod handlers;// HTTP route handlers

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env (DATABASE_URL, JWT_SECRET, SERVER_HOST, SERVER_PORT)
    dotenv().ok();

    // 2) Read the DATABASE_URL into a variable _before_ you print it
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./pbank.db".to_string());

    // 3) Now it’s safe to print it
    println!("🔍 Server is using database: {}", database_url);

    // 4) Then initialize the pool
    let db_pool = db::init_db().await;
    println!("✅ Initialized DB and ran migrations");
    

    // Determine bind address
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("{}:{}", host, port);

    // Start HTTP server with configured routes
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .configure(handlers::config)
    })
    .bind(&bind_addr)?
    .run()
    .await
}