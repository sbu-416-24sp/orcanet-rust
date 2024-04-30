use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, TimeZone, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::ServerState;

#[derive(Debug, Deserialize)]
struct WalletId {
    wallet_id: String,
}

async fn balance(
    State(state): State<ServerState>,
    Json(WalletId { wallet_id }): Json<WalletId>,
) -> impl IntoResponse {
    let balance = "100.00".to_owned();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            r#"{{"wallet_id":{wallet_id},balance:{balance}}}"#,
        )))
        .unwrap()
}

#[derive(Debug, Serialize)]
struct Revenue {
    date: String,
    earning: f64,
    spending: f64,
}

async fn revenue(
    State(state): State<ServerState>,
    Path(time): Path<String>,
    Json(WalletId { wallet_id }): Json<WalletId>,
) -> impl IntoResponse {
    let revenue: Vec<Revenue> = vec![];
    match time.as_str() {
        "daily" => {}
        "monthly" => {}
        "yearly" => {}
        _ => return StatusCode::BAD_REQUEST.into_response(),
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            r#"
{{"wallet_id":{wallet_id},revenue:{}}}
"#,
            serde_json::to_string(&revenue).expect("to serialize")
        )))
        .unwrap()
}

#[derive(Debug, Serialize)]
struct Transaction {
    id: String,
    receiver: String,
    amount: f64,
    status: String,
    reason: String,
    date: DateTime<Utc>,
}

async fn transactions_latest(
    State(state): State<ServerState>,
    Json(WalletId { wallet_id }): Json<WalletId>,
) -> impl IntoResponse {
    let transactions: Vec<Transaction> = vec![];

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            r#"
{{"wallet_id":{wallet_id},transactions:{}}}
"#,
            serde_json::to_string(&transactions).expect("to serialize")
        )))
        .unwrap()
}

async fn transactions_all(
    State(state): State<ServerState>,
    Json(WalletId { wallet_id }): Json<WalletId>,
) -> impl IntoResponse {
    let transactions: Vec<Transaction> = vec![];

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            r#"
{{"wallet_id":{wallet_id},transactions:{}}}
"#,
            serde_json::to_string(&transactions).expect("to serialize")
        )))
        .unwrap()
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct Transfer {
    wallet_id: String,
    receiver: String,
    sendAmount: f64,
    reason: String,
    date: DateTime<Utc>,
}
async fn transfer(
    State(state): State<ServerState>,
    Json(WalletId { wallet_id }): Json<WalletId>,
) -> impl IntoResponse {
    let res = Transfer {
        wallet_id,
        receiver: "13hgriwdajaPyWFABDX6QByyxvN8cWCgDp".into(),
        sendAmount: 0.00432,
        reason: "きかんしゃトーマス".into(),
        date: Utc.with_ymd_and_hms(2023, 11, 10, 0, 0, 0).unwrap(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            serde_json::to_string(&res).expect("to serialize"),
        ))
        .unwrap()
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/balance", get(balance))
        .route("/revenue/:timePeriod", get(revenue))
        .route("/transactions/latest", get(transactions_latest))
        .route("/transactions/complete", get(transactions_all))
        .route("/transfer", post(transfer))
}
