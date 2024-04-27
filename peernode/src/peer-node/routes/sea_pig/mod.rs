pub mod home;
pub mod peer;

use axum::Router;
use crate::ServerState;

pub fn routes() -> Router<ServerState> {
    Router::new()
}
