// Import necessary items from the `sqlx` crate for SQLite database connection pooling.
// `Pool` is used to manage a pool of database connections, while `Sqlite` and `SqlitePoolOptions` are specific to SQLite.
//! This module provides database initialization logic for the application.
//! It reads configuration, creates a pooled connection to a file-based SQLite database,
//! and applies any pending migrations to ensure the schema is up to date.

use sqlx::{Pool, Sqlite};
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

/// Initialize and return a SQLite connection pool.
///
/// Steps performed:
/// 1. Reads the `DATABASE_URL` environment variable to locate the database file.
///    - If absent, defaults to `sqlite://./pbank.db`, creating `pbank.db` in the working directory.
/// 2. Configures a connection pool allowing up to 5 simultaneous connections.
/// 3. Establishes the connection asynchronously, panicking if it fails.
/// 4. Runs embedded migrations from the `migrations/` folder via `sqlx::migrate!()`.
/// 5. Returns the live pool for use by the rest of the application.
///
/// # Panics
/// - If connecting to the database fails.
/// - If running migrations fails.
pub async fn init_db() -> Pool<Sqlite> {
    // Determine the database URL, using a relative path by default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./pbank.db".to_string());

    // Build the SQLite connection pool with up to 5 concurrent connections
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Apply any pending migrations to bring schema up to date
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    pool
}