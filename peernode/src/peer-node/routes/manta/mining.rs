use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::ServerState;

use super::PubKey;

#[derive(Debug, Serialize)]
struct DeviceList {
    _id: String,
    pub_key: String,
    devices: Vec<Device>,
}

#[derive(Debug, Serialize)]
struct Device {
    device_id: String,
    device_name: String,
    hash_power: String,
    power: String,
    profitability: String,
}

async fn device_list(
    State(state): State<ServerState>,
    Json(PubKey { pub_key }): Json<PubKey>,
) -> impl IntoResponse {
    let res = DeviceList {
        _id: "65680d250505420b42427a82".into(),
        pub_key,
        devices: vec![],
    };
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            serde_json::to_string(&res).expect("to serialize"),
        ))
        .unwrap()
}

#[derive(Debug, Deserialize)]
struct KeySwitch {
    pub_key: String,
    switch: String,
}

#[derive(Debug, Serialize)]
struct DeviceStatus {
    _id: String,
    pub_key: String,
    device_id: String,
    device_name: String,
    hash_power: String,
    status: String,
    power: String,
    profitability: String,
}

async fn device(
    State(state): State<ServerState>,
    Query(device_id): Query<String>,
    Json(switch): Json<KeySwitch>,
) -> impl IntoResponse {
    let res = DeviceStatus {
        _id: "65680d250505420b42427a82".into(),
        pub_key: switch.pub_key,
        device_id,
        device_name: "GeForce RTX 4090".into(),
        hash_power: "37.56".into(),
        status: "Mining".into(),
        power: "30".into(),
        profitability: "0.5523342".into(),
    };
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            serde_json::to_string(&res).expect("to serialize"),
        ))
        .unwrap()
}


#[derive(Debug, Deserialize)]
struct WalletId {
    wallet_id: String,
}
async fn unpaid_balance(State(state): State<ServerState>, Json(WalletId{wallet_id}): Json<WalletId>) -> impl IntoResponse {
    
    let unpaid_balance = "100.00".to_owned();
    
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            r#"
{{"wallet_id":{wallet_id},"unpaidBalance":{}}}
"#,
unpaid_balance
        )))
        .unwrap()
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/device_list", get(device_list))
        .route("/device", get(device))
        .route("/unpaidBalance", get(unpaid_balance))
}
