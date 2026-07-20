use axum::{routing::get, Router};
use kernel::EventBus;

#[derive(Clone)]
pub struct AppState {
    pub pool: (),
    pub event_bus: EventBus,
}

pub fn app() -> Router {
    let _state = AppState {
        pool: (),
        event_bus: EventBus::new(16),
    };

    Router::new().route("/api/v1/health", get(|| async { "ok" }))
}

#[cfg(test)]
mod tests;
