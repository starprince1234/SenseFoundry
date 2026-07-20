CREATE OR REPLACE FUNCTION audit_logs_immutable()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'audit_logs is append-only: % not allowed', TG_OP;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER audit_logs_no_update
    BEFORE UPDATE OR DELETE ON audit_logs
    FOR EACH ROW EXECUTE FUNCTION audit_logs_immutable();

CREATE OR REPLACE FUNCTION audit_generic()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_logs (
        table_name,
        row_id,
        operation,
        old_data,
        new_data,
        changed_by,
        trace_id
    )
    VALUES (
        TG_TABLE_NAME,
        COALESCE(NEW.id, OLD.id),
        TG_OP,
        CASE WHEN TG_OP IN ('UPDATE', 'DELETE') THEN to_jsonb(OLD) END,
        CASE WHEN TG_OP IN ('INSERT', 'UPDATE') THEN to_jsonb(NEW) END,
        NULLIF(current_setting('app.user_id', TRUE), '')::UUID,
        NULLIF(current_setting('app.trace_id', TRUE), '')
    );

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = public, pg_temp;

DO $$
DECLARE
    audited_table TEXT;
BEGIN
    FOREACH audited_table IN ARRAY ARRAY[
        'users', 'roles', 'permissions', 'role_permissions', 'user_roles',
        'sources', 'authorization_records', 'bibliographic_records',
        'documents', 'document_versions', 'corpus_submissions',
        'model_registry', 'models', 'model_versions', 'prompt_templates',
        'prompt_versions', 'feature_versions', 'processing_jobs',
        'usage_instances', 'target_spans', 'corpus_cards', 'headwords',
        'lexemes', 'pronunciations', 'parts_of_speech',
        'reference_dictionaries', 'reference_dictionary_versions',
        'reference_senses', 'sense_matches', 'unknown_pool', 'cluster_runs',
        'clusters', 'cluster_memberships', 'sense_candidates',
        'sense_relations', 'evidence_packs', 'definition_drafts', 'examples',
        'citations', 'review_tasks', 'review_decisions', 'editions',
        'change_sets', 'publications', 'sync_manifests', 'sync_deltas',
        'feature_artifacts', 'embedding_artifacts', 'provenance_records',
        'diachronic_slices'
    ]
    LOOP
        EXECUTE format(
            'CREATE TRIGGER %I AFTER INSERT OR UPDATE OR DELETE ON %I '
            'FOR EACH ROW EXECUTE FUNCTION audit_generic()',
            audited_table || '_audit',
            audited_table
        );
    END LOOP;
END;
$$;
