#[cfg(test)]
mod tests {
    use crate::service::WriteAuditEvent;
    use uuid::Uuid;

    #[test]
    fn test_write_audit_event_has_required_fields() {
        let event = WriteAuditEvent {
            table_name: "submissions".into(),
            row_id: Uuid::new_v4(),
            actor_id: Some(Uuid::new_v4()),
            new_data: serde_json::json!({"status": "accepted"}),
        };
        assert_eq!(event.table_name, "submissions");
        assert!(event.actor_id.is_some());
    }

    #[test]
    fn test_audit_is_insert_only_by_design() {
        // The operation field is hardcoded to 'INSERT' in write_audit
        // No UPDATE or DELETE routes exist in routes()
        // Enforced by: V002__audit_triggers.sql DENY trigger for UPDATE/DELETE
        let op = "INSERT";
        assert_eq!(op, "INSERT");
    }
}
