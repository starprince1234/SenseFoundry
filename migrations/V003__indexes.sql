CREATE INDEX usage_instances_embedding_hnsw_idx
    ON usage_instances USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

CREATE INDEX reference_senses_embedding_hnsw_idx
    ON reference_senses USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

CREATE INDEX embedding_artifacts_embedding_hnsw_idx
    ON embedding_artifacts USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

CREATE INDEX diachronic_slices_centroid_hnsw_idx
    ON diachronic_slices USING hnsw (centroid vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

CREATE INDEX role_permissions_role_id_idx ON role_permissions (role_id);
CREATE INDEX role_permissions_permission_id_idx ON role_permissions (permission_id);
CREATE INDEX user_roles_user_id_idx ON user_roles (user_id);
CREATE INDEX user_roles_granted_by_idx ON user_roles (granted_by);
CREATE INDEX authorization_records_source_id_idx ON authorization_records (source_id);
CREATE INDEX authorization_records_verified_by_idx ON authorization_records (verified_by);
CREATE INDEX bibliographic_records_source_id_idx ON bibliographic_records (source_id);
CREATE INDEX documents_source_id_idx ON documents (source_id);
CREATE INDEX document_versions_document_id_idx ON document_versions (document_id);
CREATE INDEX corpus_submissions_submitter_id_idx ON corpus_submissions (submitter_id);
CREATE INDEX corpus_submissions_document_id_idx ON corpus_submissions (document_id);
CREATE INDEX model_versions_model_id_idx ON model_versions (model_id);
CREATE INDEX prompt_versions_prompt_template_id_idx ON prompt_versions (prompt_template_id);
CREATE INDEX usage_instances_document_id_idx ON usage_instances (document_id);
CREATE INDEX usage_instances_document_version_id_idx ON usage_instances (document_version_id);
CREATE INDEX usage_instances_processing_job_id_idx ON usage_instances (processing_job_id);
CREATE INDEX target_spans_usage_instance_id_idx ON target_spans (usage_instance_id);
CREATE INDEX corpus_cards_usage_instance_id_idx ON corpus_cards (usage_instance_id);
CREATE INDEX lexemes_headword_id_idx ON lexemes (headword_id);
CREATE INDEX pronunciations_headword_id_idx ON pronunciations (headword_id);
CREATE INDEX pronunciations_lexeme_id_idx ON pronunciations (lexeme_id);
CREATE INDEX pronunciations_source_id_idx ON pronunciations (source_id);
CREATE INDEX reference_dictionaries_source_id_idx ON reference_dictionaries (source_id);
CREATE INDEX reference_dictionary_versions_dictionary_id_idx ON reference_dictionary_versions (dictionary_id);
CREATE INDEX reference_senses_headword_id_idx ON reference_senses (headword_id);
CREATE INDEX reference_senses_lexeme_id_idx ON reference_senses (lexeme_id);
CREATE INDEX reference_senses_source_id_idx ON reference_senses (source_id);
CREATE INDEX reference_senses_dictionary_version_id_idx ON reference_senses (dictionary_version_id);
CREATE INDEX reference_senses_pos_id_idx ON reference_senses (pos_id);
CREATE INDEX sense_matches_corpus_card_id_idx ON sense_matches (corpus_card_id);
CREATE INDEX sense_matches_reference_sense_id_idx ON sense_matches (reference_sense_id);
CREATE INDEX unknown_pool_corpus_card_id_idx ON unknown_pool (corpus_card_id);
CREATE INDEX cluster_runs_headword_id_idx ON cluster_runs (headword_id);
CREATE INDEX cluster_runs_processing_job_id_idx ON cluster_runs (processing_job_id);
CREATE INDEX clusters_cluster_run_id_idx ON clusters (cluster_run_id);
CREATE INDEX clusters_representative_instance_id_idx ON clusters (representative_instance_id);
CREATE INDEX cluster_memberships_cluster_id_idx ON cluster_memberships (cluster_id);
CREATE INDEX cluster_memberships_corpus_card_id_idx ON cluster_memberships (corpus_card_id);
CREATE INDEX sense_candidates_cluster_id_idx ON sense_candidates (cluster_id);
CREATE INDEX sense_candidates_headword_id_idx ON sense_candidates (headword_id);
CREATE INDEX sense_candidates_target_lexeme_id_idx ON sense_candidates (target_lexeme_id);
CREATE INDEX sense_relations_source_sense_id_idx ON sense_relations (source_sense_id);
CREATE INDEX sense_relations_target_sense_id_idx ON sense_relations (target_sense_id);
CREATE INDEX evidence_packs_sense_candidate_id_idx ON evidence_packs (sense_candidate_id);
CREATE INDEX definition_drafts_sense_candidate_id_idx ON definition_drafts (sense_candidate_id);
CREATE INDEX definition_drafts_evidence_pack_id_idx ON definition_drafts (evidence_pack_id);
CREATE INDEX definition_drafts_prompt_template_id_idx ON definition_drafts (prompt_template_id);
CREATE INDEX definition_drafts_prompt_version_id_idx ON definition_drafts (prompt_version_id);
CREATE INDEX examples_usage_instance_id_idx ON examples (usage_instance_id);
CREATE INDEX examples_sense_candidate_id_idx ON examples (sense_candidate_id);
CREATE INDEX citations_source_id_idx ON citations (source_id);
CREATE INDEX citations_usage_instance_id_idx ON citations (usage_instance_id);
CREATE INDEX citations_reference_sense_id_idx ON citations (reference_sense_id);
CREATE INDEX review_tasks_assignee_id_idx ON review_tasks (assignee_id);
CREATE INDEX review_decisions_review_task_id_idx ON review_decisions (review_task_id);
CREATE INDEX review_decisions_reviewer_id_idx ON review_decisions (reviewer_id);
CREATE INDEX editions_headword_id_idx ON editions (headword_id);
CREATE INDEX editions_signed_by_idx ON editions (signed_by);
CREATE INDEX change_sets_edition_id_idx ON change_sets (edition_id);
CREATE INDEX change_sets_review_decision_id_idx ON change_sets (review_decision_id);
CREATE INDEX change_sets_definition_draft_id_idx ON change_sets (definition_draft_id);
CREATE INDEX publications_edition_id_idx ON publications (edition_id);
CREATE INDEX publications_change_set_id_idx ON publications (change_set_id);
CREATE INDEX publications_published_by_idx ON publications (published_by);
CREATE INDEX publications_rollback_of_idx ON publications (rollback_of);
CREATE INDEX sync_manifests_edition_id_idx ON sync_manifests (edition_id);
CREATE INDEX sync_deltas_sync_manifest_id_idx ON sync_deltas (sync_manifest_id);
CREATE INDEX feature_artifacts_usage_instance_id_idx ON feature_artifacts (usage_instance_id);
CREATE INDEX feature_artifacts_feature_version_id_idx ON feature_artifacts (feature_version_id);
CREATE INDEX feature_artifacts_processing_job_id_idx ON feature_artifacts (processing_job_id);
CREATE INDEX embedding_artifacts_usage_instance_id_idx ON embedding_artifacts (usage_instance_id);
CREATE INDEX embedding_artifacts_model_registry_id_idx ON embedding_artifacts (model_registry_id);
CREATE INDEX embedding_artifacts_feature_artifact_id_idx ON embedding_artifacts (feature_artifact_id);
CREATE INDEX provenance_records_source_id_idx ON provenance_records (source_id);
CREATE INDEX provenance_records_processing_job_id_idx ON provenance_records (processing_job_id);
CREATE INDEX provenance_records_model_registry_id_idx ON provenance_records (model_registry_id);
CREATE INDEX provenance_records_prompt_version_id_idx ON provenance_records (prompt_version_id);
CREATE INDEX provenance_records_feature_version_id_idx ON provenance_records (feature_version_id);
CREATE INDEX diachronic_slices_headword_id_idx ON diachronic_slices (headword_id);
CREATE INDEX diachronic_slices_source_id_idx ON diachronic_slices (source_id);
CREATE INDEX diachronic_slices_cluster_run_id_idx ON diachronic_slices (cluster_run_id);
CREATE INDEX audit_logs_changed_by_idx ON audit_logs (changed_by);

CREATE INDEX corpus_cards_status_idx ON corpus_cards (status) WHERE deleted_at IS NULL;
CREATE INDEX processing_jobs_status_run_at_idx ON processing_jobs (status, run_at) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX processing_jobs_idempotency_key_uidx ON processing_jobs (idempotency_key);
CREATE INDEX provenance_records_entity_idx ON provenance_records (entity_type, entity_id);
CREATE INDEX provenance_records_upstream_entity_idx ON provenance_records (upstream_entity_type, upstream_entity_id);
CREATE INDEX audit_logs_table_row_changed_at_idx ON audit_logs (table_name, row_id, changed_at DESC);
