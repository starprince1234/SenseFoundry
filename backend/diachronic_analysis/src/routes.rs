use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::service::{AnalysisService, DiachronicAnalysis};

pub fn routes(service: Arc<AnalysisService>) -> Router {
    Router::new()
        .route("/diachronic-analysis/:headword", get(analyze))
        .with_state(service)
}

async fn analyze(
    State(service): State<Arc<AnalysisService>>,
    Path(headword): Path<String>,
) -> Json<DiachronicAnalysis> {
    Json(service.analyze(&headword))
}
