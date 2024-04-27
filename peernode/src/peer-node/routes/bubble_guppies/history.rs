#![allow(non_snake_case)]
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;

use crate::ServerState;

///
/// HISTORY endpoints
///

async fn get_history(State(state): State<ServerState>) -> impl IntoResponse {
    let mut config = state.config.lock().await;
    let history = config.jobs_mut().get_job_history().await;

    let history_json = serde_json::to_string(&history).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(r#"{{"jobs":{history_json:?}}}"#,)))
        .unwrap()
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct RemoveFromHistory {
    jobID: String,
}

async fn remove_from_history(
    State(state): State<ServerState>,
    Json(job): Json<RemoveFromHistory>,
) -> impl IntoResponse {
    let mut config = state.config.lock().await;

    let successful = config.jobs_mut().remove_job_from_history(&job.jobID).await;

    if !successful {
        return (StatusCode::NOT_FOUND, "Job not found").into_response();
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Job successfully removed from history"))
        .unwrap()
}

async fn clear_history(State(state): State<ServerState>) -> impl IntoResponse {
    let mut config = state.config.lock().await;
    config.jobs_mut().clear_job_history().await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("History cleared"))
        .unwrap()
}


pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/get-history", get(get_history))
        .route("/remove-from-history", put(remove_from_history))
        .route("/clear-history", put(clear_history))
}
