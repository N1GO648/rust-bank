# pbank API Testing Guide

A step-by-step guide to get your local pbank server up and running, seed test users, generate bcrypt hashes, and exercise all API endpoints.

---

## Prerequisites

* **Rust** (stable) and **cargo** installed
* **SQLite3** CLI (`sqlite3`)
* **Python 3** (optional, for JSON parsing in shell)
* **bcrypt** Rust crate (already in `Cargo.toml`)

---

## 1  Build

```bash
cd csc1106-rust-practice-01-bank-main
cargo build               # compile both server and helper
```

---

## 2 Configuration

Create a `.env` file in the project root:

```env
DATABASE_URL=sqlite:./pbank.db
JWT_SECRET=your_super_secret_key
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

---

## 3 Run the Server & Migrate

Start the API (it will auto-run migrations):

```bash
cargo run --bin practice-01
```

You should see:

```
🔍 Server is using database: sqlite:./pbank.db
✅ Initialized DB and ran migrations
Listening on 127.0.0.1:8080
```

Leave this terminal running.

---

## 4 Create & Seed `pbank.db`

Open a new terminal in the **project** folder and run:

```bash
# Insert an admin user (UUID is arbitrary but fixed)
sqlite3 pbank.db <<'SQL'
INSERT OR IGNORE INTO users (id, username, hashed_password) VALUES (
  '11111111-1111-1111-1111-111111111111',
  'admin',
  '<bcrypt-hash-for-fake>'
);

INSERT OR IGNORE INTO stocks (id, symbol, price) VALUES (
  '22222222-2222-2222-2222-222222222222',
  'TEST',
  42.0
);
SQL
```

> Note: Replace `<bcrypt-hash-for-fake>` with a real hash (see next section).

---

## 5 Generate a Bcrypt Hash

### Rust Helper

```bash
# Hash the default password "fake"
cargo run --bin gen_hash

```

Copy the printed `$2b$...` string and paste it into the `INSERT` above (re-run the SQL) if needed.


---

## 6 Log In & Obtain JWT

In a new terminal:

```bash
# Login and capture raw response
_resp=$(curl -s -X POST http://127.0.0.1:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","hashed_password":"fake"}')

# (Optional) inspect:
echo "$ _resp"

# Extract token (with sed)
TOKEN=$(printf '%s' "$_resp" \
  | sed -E 's/.*"token"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/')
echo "Admin token: $TOKEN"
```

---

## 7 Test API Endpoints

Use your `$TOKEN` variable for the `Authorization` header.

### 7.1 Fetch a Stock

```bash
curl -i -H "Authorization: Bearer $TOKEN" \
     http://127.0.0.1:8080/stocks/TEST
```

### 7.2 Buy Shares

```bash
curl -i -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"stock_id":"22222222-2222-2222-2222-222222222222","quantity":10}' \
  http://127.0.0.1:8080/buy
```

### 7.3 Sell Shares

```bash
curl -i -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"stock_id":"22222222-2222-2222-2222-222222222222","quantity":5}' \
  http://127.0.0.1:8080/sell
```

### 7.4 List Transactions

```bash
curl -i -H "Authorization: Bearer $TOKEN" \
     http://127.0.0.1:8080/transactions
```

---

## 8 Add & Test a Second User

1. **Generate** a new hash: `cargo run --bin gen_hash`
2. **Seed** test2:

   ```bash
   sqlite3 pbank.db <<'SQL'
   INSERT OR REPLACE INTO users (id, username, hashed_password) VALUES (
     '33333333-3333-3333-3333-333333333333',
     'test2',
     '<new-bcrypt-hash>'
   );
   SQL
   ```
3. **Login** as test2:

   ```bash
   _resp=$(curl -s -X POST http://127.0.0.1:8080/login \
     -H "Content-Type: application/json" \
     -d '{"username":"test2","hashed_password":"fake"}')
   echo "$_resp"  # should show {"token":...}
   ```
4. **Extract** and use `$TOKEN2` just like admin for `/buy`, `/sell`, etc.

---
# rust-bank
