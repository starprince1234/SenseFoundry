pub mod routes;
pub mod service;

#[cfg(test)]
mod tests;

pub use service::{write_audit, WriteAuditEvent};
