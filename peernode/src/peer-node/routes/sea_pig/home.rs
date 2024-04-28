#![allow(non_snake_case)]
use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use proto::market::{FileInfoHash, User};
use serde::{Deserialize, Serialize};

use crate::ServerState;

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
        Ok(market) => match market.check_holders(FileInfoHash(hash)).await {
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

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct UploadParams {
    filePath: String,
    price: i64,
}
async fn upload_file(
    State(state): State<ServerState>,
    Json(UploadParams { filePath, price }): Json<UploadParams>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    let hash = match config.add_file(&PathBuf::from(filePath), price).await {
        Ok(hash) => hash,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add file").into_response(),
    };
    let hash_str = hash.as_str();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(r#"{{"hash":"{hash_str}"}}"#)))
        .unwrap()
}

async fn delete_file(
    State(state): State<ServerState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;
    match config.remove_file_by_hash(FileInfoHash(hash.clone())).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(format!(r#"{{"hash": "{hash}"}}"#)))
            .unwrap(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to remove file: {e}"),
        )
            .into_response(),
    }
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/file/:hash/info", get(get_file_info))
        .route("/upload", post(upload_file))
        .route("/file/:hash", delete(delete_file))
}
