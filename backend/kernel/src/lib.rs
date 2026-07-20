pub mod audit;
pub mod error;
pub mod etag;
pub mod events;
pub mod idempotency;
pub mod otel;
pub mod pagination;

pub use error::{AppError, AppResult};
pub use events::EventBus;
pub use pagination::{Page, PageParams};

#[cfg(test)]
mod tests;
