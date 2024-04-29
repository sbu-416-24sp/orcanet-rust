mod history;
mod jobs;
mod market;

use crate::ServerState;
use axum::Router;

pub fn routes() -> Router<ServerState> {
    Router::new()
        .merge(market::routes())
}
