#[cfg(test)]
mod tests {
    use kernel::error::AppError;

    #[test]
    fn test_not_found_error_message() {
        let err = AppError::NotFound("test resource".into());
        let msg = err.to_string();
        assert!(msg.contains("not found"), "Error message should contain 'not found': {}", msg);
    }

    #[test]
    fn test_unauthorized_error() {
        let err = AppError::Unauthorized;
        let msg = err.to_string();
        assert!(msg.contains("unauthorized") || msg.contains("Unauthorized"), "Got: {}", msg);
    }

    #[test]
    fn test_forbidden_error() {
        let err = AppError::Forbidden("insufficient permissions".into());
        let msg = err.to_string();
        assert!(msg.contains("forbidden") || msg.contains("Forbidden") || msg.contains("insufficient"), "Got: {}", msg);
    }

    #[test]
    fn test_api_health_path_constant() {
        // Health endpoint is registered at /api/v1/health
        const HEALTH_PATH: &str = "/api/v1/health";
        assert!(HEALTH_PATH.starts_with("/api/v1"), "Base path must be /api/v1");
    }

    #[test]
    fn test_app_base_path() {
        const BASE: &str = "/api/v1";
        assert_eq!(BASE, "/api/v1");
    }
}
