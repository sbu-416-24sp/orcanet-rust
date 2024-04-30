pub mod home;
pub mod peer;

use crate::ServerState;
use axum::Router;

pub fn routes() -> Router<ServerState> {
    Router::new().merge(home::routes()).merge(peer::routes())
}
