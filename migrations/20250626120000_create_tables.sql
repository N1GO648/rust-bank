CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL
);

CREATE TABLE stocks (
    id TEXT PRIMARY KEY,
    symbol TEXT NOT NULL UNIQUE,
    price REAL NOT NULL
);

CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    stock_id TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    transaction_type TEXT CHECK(transaction_type IN ('buy', 'sell')),
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id),
    FOREIGN KEY(stock_id) REFERENCES stocks(id)
);
