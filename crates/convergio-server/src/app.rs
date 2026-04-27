//! Router assembly + shared state.

use axum::Router;
use convergio_durability::Durability;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

/// Application state injected into every handler.
#[derive(Clone)]
pub struct AppState {
    /// Layer 1 facade.
    pub durability: Arc<Durability>,
}

/// Build the top-level router. Test harnesses call this directly with
/// a tempdir-backed `Durability`.
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(crate::routes::health::router())
        .merge(crate::routes::plans::router())
        .merge(crate::routes::tasks::router())
        .merge(crate::routes::evidence::router())
        .merge(crate::routes::audit::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
