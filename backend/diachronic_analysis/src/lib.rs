pub mod routes;
pub mod service;

pub use routes::routes;
pub use service::{AnalysisService, CorpusExample, DiachronicAnalysis, PeriodGroup};
