//! HTTP route handlers for our REST API.
//! Each handler enforces JWT auth (except `/login`), executes queries, and returns JSON or HTTP errors.

use actix_web::{web, HttpResponse, Responder, HttpRequest, Error};
use actix_web::error::{ErrorUnauthorized, ErrorInternalServerError};
use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use crate::{auth, models::{Stock, Transaction}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use bcrypt;

/// Payload for `/login`.
#[derive(Deserialize)]
pub struct LoginRequest { username: String, hashed_password: String }

/// Success response for `/login`.
#[derive(Serialize)]
pub struct LoginResponse { token: String }

/// Payload for buy/sell transactions.
#[derive(Deserialize)]
pub struct TransactionRequest { stock_id: Uuid, quantity: i32 }

/// Extract and validate JWT from `Authorization` header.
fn authorize(req: &HttpRequest) -> Result<Uuid, Error> {
    let header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ErrorUnauthorized("Missing or malformed Authorization header"))?;
    let sub = auth::validate_token(token)
        .ok_or_else(|| ErrorUnauthorized("Invalid token"))?;
    Uuid::parse_str(&sub)
        .map_err(|_| ErrorUnauthorized("Invalid user ID in token"))
}

/// POST /login: authenticate user and return JWT.
pub async fn login(
    pool: web::Data<SqlitePool>,
    body: web::Json<LoginRequest>
) -> impl Responder {
    let row = match sqlx::query("SELECT id, hashed_password FROM users WHERE username = ?")
        .bind(&body.username)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(r) => r,
        Err(_) => return HttpResponse::Unauthorized().body("Invalid credentials"),
    };
    let user_id_str: String = row.try_get("id").unwrap_or_default();
    let stored_hash: String = row.try_get("hashed_password").unwrap_or_default();
    if !bcrypt::verify(&body.hashed_password, &stored_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().body("Invalid credentials");
    }
    let user_id = Uuid::parse_str(&user_id_str).unwrap();
    let token = auth::create_token(user_id);
    HttpResponse::Ok().json(LoginResponse { token })
}

/// POST /buy: record a buy transaction directly.
pub async fn buy_stock(
    pool: web::Data<SqlitePool>,
    req: HttpRequest,
    body: web::Json<TransactionRequest>
) -> Result<impl Responder, Error> {
    let user_id = authorize(&req)?;
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO transactions (id, user_id, stock_id, quantity, transaction_type) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(id.to_string())
    .bind(user_id.to_string())
    .bind(body.stock_id.to_string())
    .bind(body.quantity)
    .bind("buy")
    .execute(pool.get_ref())
    .await
    .map_err(|_| ErrorInternalServerError("Failed to record buy transaction"))?;
    Ok(HttpResponse::Ok().json(json!({
        "id": id,
        "user_id": user_id,
        "stock_id": body.stock_id,
        "quantity": body.quantity,
        "transaction_type": "buy"
    })))
}

/// POST /sell: record a sell transaction directly.
pub async fn sell_stock(
    pool: web::Data<SqlitePool>,
    req: HttpRequest,
    body: web::Json<TransactionRequest>
) -> Result<impl Responder, Error> {
    let user_id = authorize(&req)?;
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO transactions (id, user_id, stock_id, quantity, transaction_type) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(id.to_string())
    .bind(user_id.to_string())
    .bind(body.stock_id.to_string())
    .bind(body.quantity)
    .bind("sell")
    .execute(pool.get_ref())
    .await
    .map_err(|_| ErrorInternalServerError("Failed to record sell transaction"))?;
    Ok(HttpResponse::Ok().json(json!({
        "id": id,
        "user_id": user_id,
        "stock_id": body.stock_id,
        "quantity": body.quantity,
        "transaction_type": "sell"
    })))
}

/// GET /transactions: retrieve all transactions for the authenticated user.
pub async fn get_transactions(
    pool: web::Data<SqlitePool>,
    req: HttpRequest
) -> Result<impl Responder, Error> {
    let user_id = authorize(&req)?;
    let rows = sqlx::query(
        "SELECT id, user_id, stock_id, quantity, transaction_type FROM transactions WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(user_id.to_string())
    .fetch_all(pool.get_ref())
    .await
    .map_err(|_| ErrorInternalServerError("Failed to query transactions"))?;
    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let id_str: String = row.try_get("id").map_err(|_| ErrorInternalServerError("Invalid `id` field"))?;
        let uid_str: String = row.try_get("user_id").map_err(|_| ErrorInternalServerError("Invalid `user_id` field"))?;
        let sid_str: String = row.try_get("stock_id").map_err(|_| ErrorInternalServerError("Invalid `stock_id` field"))?;
        let quantity: i32 = row.try_get("quantity").map_err(|_| ErrorInternalServerError("Invalid `quantity` field"))?;
        let transaction_type: String = row.try_get("transaction_type").map_err(|_| ErrorInternalServerError("Invalid `transaction_type` field"))?;
        let transaction = Transaction { id: Uuid::parse_str(&id_str).map_err(|_| ErrorInternalServerError("Invalid UUID in `id`"))?, user_id: Uuid::parse_str(&uid_str).map_err(|_| ErrorInternalServerError("Invalid UUID in `user_id`"))?, stock_id: Uuid::parse_str(&sid_str).map_err(|_| ErrorInternalServerError("Invalid UUID in `stock_id`"))?, quantity, transaction_type };
        results.push(transaction);
    }
    Ok(HttpResponse::Ok().json(results))
}

/// GET /stocks/{symbol}: fetch a stock by its symbol.
pub async fn get_stock(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>
) -> impl Responder {
    let symbol = path.into_inner();
    eprintln!("🔍 get_stock called with symbol = {:?}", symbol);
    let row = match sqlx::query(
        "SELECT id, symbol, price FROM stocks WHERE symbol = ?"
    )
    .bind(&symbol)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(err) => {
            eprintln!("⚠️  get_stock error: {:?}", err);
            return HttpResponse::NotFound().body("Stock not found or DB error");
        }
    };
    let id_str: String = match row.try_get("id") {
        Ok(s) => s,
        Err(_) => return HttpResponse::InternalServerError().body("Invalid `id` field"),
    };
    let price: f64 = match row.try_get("price") {
        Ok(p) => p,
        Err(_) => return HttpResponse::InternalServerError().body("Invalid `price` field"),
    };
    let stock = Stock { id: match Uuid::parse_str(&id_str) { Ok(u) => u, Err(_) => return HttpResponse::InternalServerError().body("Invalid UUID in `id`"), }, symbol, price };
    HttpResponse::Ok().json(stock)
}

/// Register all routes with Actix.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)));
    cfg.service(web::resource("/buy").route(web::post().to(buy_stock)));
    cfg.service(web::resource("/sell").route(web::post().to(sell_stock)));
    cfg.service(web::resource("/transactions").route(web::get().to(get_transactions)));
    cfg.service(web::resource("/stocks/{symbol}").route(web::get().to(get_stock)));
}