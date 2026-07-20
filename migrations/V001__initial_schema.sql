CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id TEXT NOT NULL UNIQUE,
    email TEXT,
    roles JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id),
    permission_id UUID NOT NULL REFERENCES permissions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (role_id, permission_id)
);

CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    role_name TEXT NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    deleted_at TIMESTAMPTZ,
    UNIQUE (user_id, role_name)
);

CREATE TABLE sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uri TEXT NOT NULL,
    title TEXT,
    author TEXT,
    publisher TEXT,
    publication_date DATE,
    license TEXT,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('web_page', 'book', 'journal', 'user_submission', 'internal_seed')),
    copyright_status TEXT NOT NULL CHECK (copyright_status IN ('public_domain', 'licensed', 'unknown', 'user_submitted')),
    is_authoritative BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE authorization_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES sources(id),
    authorization_kind TEXT NOT NULL,
    terms_uri TEXT,
    valid_from DATE,
    valid_until DATE,
    verified_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE bibliographic_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES sources(id),
    isbn TEXT,
    doi TEXT,
    issn TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES sources(id),
    content_hash TEXT NOT NULL UNIQUE,
    original_bytes BYTEA,
    encoding TEXT,
    byte_count INT CHECK (byte_count IS NULL OR byte_count >= 0),
    language TEXT NOT NULL DEFAULT 'zh',
    processing_version TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id),
    content_hash TEXT NOT NULL UNIQUE,
    storage_uri TEXT,
    processing_version TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE corpus_submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submitter_id UUID NOT NULL REFERENCES users(id),
    document_id UUID NOT NULL REFERENCES documents(id),
    submission_type TEXT NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('pending', 'processing', 'accepted', 'rejected')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE model_registry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_name TEXT NOT NULL,
    model_version TEXT NOT NULL UNIQUE,
    model_type TEXT NOT NULL CHECK (model_type IN ('embedding', 'mlm', 'reranker', 'llm')),
    hf_repo TEXT,
    device_requirement TEXT,
    checksum TEXT,
    loaded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    license TEXT NOT NULL,
    model_type TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE model_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL REFERENCES models(id),
    version TEXT NOT NULL,
    artifact_hash TEXT NOT NULL UNIQUE,
    metrics JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (model_id, version)
);

CREATE TABLE prompt_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    template_text TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (name, version)
);

CREATE TABLE prompt_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prompt_template_id UUID NOT NULL REFERENCES prompt_templates(id),
    version TEXT NOT NULL,
    template_text TEXT NOT NULL,
    content_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (prompt_template_id, version)
);

CREATE TABLE feature_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    description TEXT,
    processing_pipeline_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE processing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    status TEXT NOT NULL CHECK (status IN ('QUEUED', 'RUNNING', 'SUCCEEDED', 'FAILED', 'RETRYING', 'DEAD_LETTER')),
    attempts INT NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    max_attempts INT NOT NULL DEFAULT 3 CHECK (max_attempts > 0),
    random_seed INT,
    run_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    idempotency_key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE usage_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id),
    document_version_id UUID REFERENCES document_versions(id),
    processing_job_id UUID REFERENCES processing_jobs(id),
    target_headword TEXT NOT NULL,
    sentence_text TEXT NOT NULL,
    sentence_start INT CHECK (sentence_start IS NULL OR sentence_start >= 0),
    sentence_end INT,
    context_window TEXT,
    embedding vector(1536),
    processing_version TEXT,
    feature_version TEXT,
    model_version TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CHECK (sentence_end IS NULL OR sentence_start IS NULL OR sentence_end >= sentence_start)
);

CREATE TABLE target_spans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    usage_instance_id UUID NOT NULL REFERENCES usage_instances(id),
    start_char INT NOT NULL CHECK (start_char >= 0),
    end_char INT NOT NULL,
    surface TEXT NOT NULL,
    target_headword TEXT NOT NULL,
    target_lexeme TEXT,
    match_type TEXT NOT NULL CHECK (match_type IN ('char_exact', 'char_in_lexeme', 'lexeme_multi_char', 'char_in_word')),
    detector TEXT,
    human_verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CHECK (end_char > start_char)
);

CREATE TABLE corpus_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    usage_instance_id UUID NOT NULL REFERENCES usage_instances(id),
    status TEXT NOT NULL CHECK (status IN ('DRAFT', 'PROCESSING', 'NEEDS_VERIFICATION', 'VERIFIED', 'MATCHED', 'CLUSTERED', 'REVIEWED', 'ARCHIVED')),
    quality_score DOUBLE PRECISION,
    annotation JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE headwords (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    character TEXT NOT NULL,
    normalized TEXT,
    pinyin TEXT[],
    stroke_count INT CHECK (stroke_count IS NULL OR stroke_count >= 0),
    radical TEXT,
    traditional_form TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE lexemes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    headword_id UUID NOT NULL REFERENCES headwords(id),
    lemma TEXT NOT NULL,
    normalized_lemma TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (headword_id, normalized_lemma)
);

CREATE TABLE pronunciations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    headword_id UUID NOT NULL REFERENCES headwords(id),
    lexeme_id UUID REFERENCES lexemes(id),
    pinyin TEXT NOT NULL,
    system TEXT NOT NULL DEFAULT 'hanyu_pinyin',
    source_id UUID REFERENCES sources(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE parts_of_speech (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE reference_dictionaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES sources(id),
    name TEXT NOT NULL,
    license TEXT,
    copyright_isolate BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE reference_dictionary_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dictionary_id UUID NOT NULL REFERENCES reference_dictionaries(id),
    release TEXT NOT NULL,
    content_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (dictionary_id, release)
);

CREATE TABLE reference_senses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    headword_id UUID NOT NULL REFERENCES headwords(id),
    lexeme_id UUID REFERENCES lexemes(id),
    source_id UUID NOT NULL REFERENCES sources(id),
    dictionary_version_id UUID REFERENCES reference_dictionary_versions(id),
    original_id TEXT,
    sense_number INT,
    pos TEXT,
    pos_id UUID REFERENCES parts_of_speech(id),
    gloss TEXT NOT NULL,
    example_text TEXT,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('authoritative', 'internal_seed', 'user_submitted')),
    is_authoritative BOOLEAN NOT NULL DEFAULT FALSE,
    is_publishable BOOLEAN NOT NULL DEFAULT FALSE,
    copyright_isolate BOOLEAN NOT NULL DEFAULT TRUE,
    embedding vector(1536),
    model_version TEXT,
    feature_version TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE sense_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corpus_card_id UUID NOT NULL REFERENCES corpus_cards(id),
    reference_sense_id UUID REFERENCES reference_senses(id),
    match_score DOUBLE PRECISION,
    rerank_score DOUBLE PRECISION,
    is_unknown BOOLEAN NOT NULL DEFAULT FALSE,
    match_method TEXT,
    model_version TEXT,
    feature_version TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CHECK (is_unknown OR reference_sense_id IS NOT NULL)
);

CREATE TABLE unknown_pool (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corpus_card_id UUID NOT NULL REFERENCES corpus_cards(id),
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE cluster_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    headword_id UUID NOT NULL REFERENCES headwords(id),
    processing_job_id UUID REFERENCES processing_jobs(id),
    method TEXT NOT NULL CHECK (method IN ('hdbscan', 'hierarchical')),
    min_cluster_size INT,
    min_samples INT,
    random_seed INT,
    stability_score DOUBLE PRECISION,
    model_version TEXT,
    feature_version TEXT,
    run_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE clusters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cluster_run_id UUID NOT NULL REFERENCES cluster_runs(id),
    cluster_label INT NOT NULL,
    membership_count INT NOT NULL DEFAULT 0 CHECK (membership_count >= 0),
    representative_instance_id UUID REFERENCES usage_instances(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (cluster_run_id, cluster_label)
);

CREATE TABLE cluster_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cluster_id UUID NOT NULL REFERENCES clusters(id),
    corpus_card_id UUID NOT NULL REFERENCES corpus_cards(id),
    probability DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (cluster_id, corpus_card_id)
);

CREATE TABLE sense_candidates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cluster_id UUID NOT NULL REFERENCES clusters(id),
    headword_id UUID NOT NULL REFERENCES headwords(id),
    target_lexeme_id UUID REFERENCES lexemes(id),
    proposed_gloss TEXT,
    status TEXT NOT NULL CHECK (status IN ('proposed', 'under_review', 'accepted', 'rejected', 'merged')),
    evidence_count INT NOT NULL DEFAULT 0 CHECK (evidence_count >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE sense_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_sense_id UUID NOT NULL REFERENCES sense_candidates(id),
    target_sense_id UUID NOT NULL REFERENCES sense_candidates(id),
    relation_type TEXT NOT NULL CHECK (relation_type IN ('synonym', 'antonym', 'broader', 'narrower', 'related', 'derived_from')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CHECK (source_sense_id <> target_sense_id)
);

CREATE TABLE evidence_packs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sense_candidate_id UUID NOT NULL REFERENCES sense_candidates(id),
    content_hash TEXT NOT NULL UNIQUE,
    evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE definition_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sense_candidate_id UUID NOT NULL REFERENCES sense_candidates(id),
    evidence_pack_id UUID REFERENCES evidence_packs(id),
    draft_text TEXT NOT NULL,
    evidence_ids UUID[] NOT NULL DEFAULT '{}'::uuid[],
    prompt_template_id UUID REFERENCES prompt_templates(id),
    prompt_version_id UUID REFERENCES prompt_versions(id),
    model_version TEXT,
    llm_model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE examples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    usage_instance_id UUID NOT NULL REFERENCES usage_instances(id),
    sense_candidate_id UUID NOT NULL REFERENCES sense_candidates(id),
    score DOUBLE PRECISION,
    diversity_rank INT,
    selected BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE citations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES sources(id),
    usage_instance_id UUID REFERENCES usage_instances(id),
    reference_sense_id UUID REFERENCES reference_senses(id),
    locator TEXT,
    excerpt TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE review_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type TEXT NOT NULL CHECK (task_type IN ('card_verify', 'match_review', 'cluster_review', 'candidate_review', 'definition_review', 'example_review', 'publication_approve')),
    target_id UUID NOT NULL,
    assignee_id UUID REFERENCES users(id),
    status TEXT NOT NULL CHECK (status IN ('pending', 'in_progress', 'completed', 'expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE review_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_task_id UUID NOT NULL REFERENCES review_tasks(id),
    reviewer_id UUID NOT NULL REFERENCES users(id),
    decision TEXT NOT NULL CHECK (decision IN ('approve', 'reject', 'split', 'merge', 'amend', 'abstain')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE editions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    headword_id UUID NOT NULL REFERENCES headwords(id),
    version_number INT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('draft', 'review', 'published', 'superseded', 'rolled_back')),
    content_hash TEXT NOT NULL,
    signature TEXT,
    signed_by UUID REFERENCES users(id),
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (headword_id, version_number)
);

CREATE TABLE change_sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edition_id UUID NOT NULL REFERENCES editions(id),
    review_decision_id UUID NOT NULL REFERENCES review_decisions(id),
    definition_draft_id UUID NOT NULL REFERENCES definition_drafts(id),
    changes JSONB NOT NULL DEFAULT '{}'::jsonb,
    content_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE publications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edition_id UUID NOT NULL REFERENCES editions(id),
    change_set_id UUID REFERENCES change_sets(id),
    published_by UUID NOT NULL REFERENCES users(id),
    change_summary TEXT,
    diff_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    rollback_of UUID REFERENCES publications(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE sync_manifests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edition_id UUID NOT NULL REFERENCES editions(id),
    manifest_hash TEXT NOT NULL,
    signature TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE sync_deltas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sync_manifest_id UUID NOT NULL REFERENCES sync_manifests(id),
    delta_bytes BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE feature_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    usage_instance_id UUID NOT NULL REFERENCES usage_instances(id),
    feature_version_id UUID NOT NULL REFERENCES feature_versions(id),
    processing_job_id UUID REFERENCES processing_jobs(id),
    storage_uri TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE embedding_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    usage_instance_id UUID NOT NULL REFERENCES usage_instances(id),
    model_registry_id UUID NOT NULL REFERENCES model_registry(id),
    feature_artifact_id UUID REFERENCES feature_artifacts(id),
    dimension INT NOT NULL CHECK (dimension > 0),
    embedding vector(1536),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CHECK (dimension = 1536)
);

CREATE TABLE provenance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES sources(id),
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    upstream_entity_type TEXT,
    upstream_entity_id UUID,
    processing_job_id UUID REFERENCES processing_jobs(id),
    model_registry_id UUID REFERENCES model_registry(id),
    prompt_version_id UUID REFERENCES prompt_versions(id),
    feature_version_id UUID REFERENCES feature_versions(id),
    parameters JSONB NOT NULL DEFAULT '{}'::jsonb,
    random_seed INT,
    output_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE diachronic_slices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    headword_id UUID NOT NULL REFERENCES headwords(id),
    source_id UUID NOT NULL REFERENCES sources(id),
    cluster_run_id UUID REFERENCES cluster_runs(id),
    time_bucket TEXT NOT NULL,
    instance_count INT NOT NULL CHECK (instance_count >= 0),
    centroid vector(1536),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    table_name TEXT NOT NULL,
    row_id UUID,
    operation TEXT NOT NULL CHECK (operation IN ('INSERT', 'UPDATE', 'DELETE')),
    old_data JSONB,
    new_data JSONB,
    changed_by UUID REFERENCES users(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    trace_id TEXT
);
