mod market;
mod history;
mod jobs;
mod files;
mod settings;

use crate::ServerState;
use axum::Router;

pub fn routes() -> Router<ServerState> {
    Router::new()
        .merge(market::routes())
        .merge(settings::routes())
}
