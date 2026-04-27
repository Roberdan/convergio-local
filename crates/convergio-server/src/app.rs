//! Router assembly + shared state.

use axum::Router;
use convergio_bus::Bus;
use convergio_durability::Durability;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

/// Application state injected into every handler.
#[derive(Clone)]
pub struct AppState {
    /// Layer 1 facade.
    pub durability: Arc<Durability>,
    /// Layer 2 facade.
    pub bus: Arc<Bus>,
}

/// Build the top-level router. Test harnesses call this directly with
/// a tempdir-backed [`Durability`] + [`Bus`].
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(crate::routes::health::router())
        .merge(crate::routes::plans::router())
        .merge(crate::routes::tasks::router())
        .merge(crate::routes::evidence::router())
        .merge(crate::routes::audit::router())
        .merge(crate::routes::messages::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
