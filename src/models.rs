//! Core domain models representing our database entities.
//! These structs derive serialization for JSON I/O and debugging support.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

/// A registered user of the system.
///
/// Fields:
/// - `id`: Primary key (UUID).
/// - `username`: Unique login name.
/// - `hashed_password`: Bcrypt hash of the user’s password.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub hashed_password: String,
}

/// Represents a stock listing.
///
/// Fields:
/// - `id`: Primary key (UUID).
/// - `symbol`: Ticker symbol, e.g., "AAPL".
/// - `price`: Current price per share.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Stock {
    pub id: Uuid,
    pub symbol: String,
    pub price: f64,
}

/// Represents a buy/sell transaction.
///
/// Fields:
/// - `id`: Primary key (UUID).
/// - `user_id`: References `users.id`.
/// - `stock_id`: References `stocks.id`.
/// - `quantity`: Number of shares.
/// - `transaction_type`: Either "buy" or "sell".
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stock_id: Uuid,
    pub quantity: i32,
    pub transaction_type: String,
}