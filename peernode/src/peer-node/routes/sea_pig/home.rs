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

#[derive(Deserialize, Serialize)]
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HashBody {
    hash: String,
}

#[allow(dead_code)]
const BASE_URL: &str = "http://localhost:3000";
#[allow(dead_code)]
const GIRAFFE_HASH: &str = "908b7415fea62428bb69eb01d8a3ce64190814cc01f01cae0289939e72909227";

#[tokio::test]
#[ignore]
async fn api_test_upload_delete() {
    let client = reqwest::Client::new();
    let upload_res = client
        .post(format!("{BASE_URL}/upload"))
        .json(&UploadParams {
            filePath: "files/giraffe.jpg".into(),
            price: 416,
        })
        .send()
        .await
        .expect("a response");

    let HashBody { hash } = upload_res.json().await.expect("to deserialize");
    assert_eq!(hash, GIRAFFE_HASH);

    // should be successful
    let delete_res = client
        .delete(format!("{BASE_URL}/file/{GIRAFFE_HASH}"))
        .send()
        .await
        .expect("a response");

    let HashBody { hash } = delete_res.json().await.expect("to deserialize");
    assert_eq!(hash, GIRAFFE_HASH);

    // should fail
    let delete_res = client
        .delete(format!("{BASE_URL}/file/{GIRAFFE_HASH}"))
        .send()
        .await
        .expect("a response");

    assert_ne!(delete_res.status(), StatusCode::OK);
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/file/:hash/info", get(get_file_info))
        .route("/upload", post(upload_file))
        .route("/file/:hash", delete(delete_file))
}
