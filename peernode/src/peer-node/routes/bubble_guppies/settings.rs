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

use crate::{store::Theme, ServerState};

#[derive(Deserialize)]
#[allow(dead_code)]
struct SetSettings {
    theme: Theme,
    server: String,
}

async fn set_settings(
    State(state): State<ServerState>,
    Json(settings): Json<SetSettings>,
) -> Response {
    let mut config = state.config.lock().await;
    config.set_theme(settings.theme);

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

async fn get_settings(State(state): State<ServerState>) -> impl IntoResponse {
    let config = state.config.lock().await;
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            r#"
{{"theme": "{:?}","server":"rust"}}
"#,
            config.get_theme()
        )))
        .unwrap()
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/set-settings", put(set_settings))
        .route("/get-settings", get(get_settings))
}
