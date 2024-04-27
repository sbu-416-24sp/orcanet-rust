#![allow(non_snake_case)]
use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use proto::market::User;
use serde::{Deserialize, Serialize};

use crate::{
    consumer::encode::{self, try_decode_user},
    producer::{
        self,
        jobs::{JobListItem, JobStatus},
    },
    ServerState,
};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct FileInfo {
    name: String,
    size: i64,
    numberOfPeers: i64,
    listProducers: Vec<User>,
}

async fn get_file_info(
    State(state): State<ServerState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;
    let response = match config.get_market_client().await {
        Ok(market) => match market.check_holders(hash).await {
            Ok(holders) => holders,
            Err(_) => {
                return (StatusCode::SERVICE_UNAVAILABLE, "Could not check holders").into_response()
            }
        },
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Could not connect to market",
            )
                .into_response()
        }
    };
    let file_info = response.file_info.unwrap();
    let file_info = FileInfo {
        name: file_info.file_name,
        size: file_info.file_size,
        numberOfPeers: response.holders.len() as i64,
        listProducers: response.holders,
    };

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            serde_json::to_string(&file_info).expect("to serialize"),
        ))
        .unwrap()
}

pub fn routes() -> Router<ServerState> {
    Router::new().route("/file/:hash/info", put(get_file_info))
}
