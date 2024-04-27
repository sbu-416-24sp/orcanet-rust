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
use serde::Deserialize;

use crate::{consumer::encode, producer, ServerState};


// Get History
async fn get_history(State(state): State<ServerState>) -> impl IntoResponse {
    let mut config = state.config.lock().await;
    let history = config.jobs_mut().get_job_history().await;

    let history_json = serde_json::to_string(&history).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("{{\"jobs\": \"{:?}\"}}", history_json)))
        .unwrap()
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/get-history", get(get_history))
}