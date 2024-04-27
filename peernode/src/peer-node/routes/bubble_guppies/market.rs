use axum::Router;

use crate::ServerState;

use super::{history, jobs};

pub fn routes() -> Router<ServerState> {
    Router::new().merge(history::routes()).merge(jobs::routes())
}
