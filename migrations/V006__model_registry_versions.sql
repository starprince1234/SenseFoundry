ALTER TABLE model_registry
    ADD COLUMN revision TEXT NOT NULL DEFAULT 'main',
    ADD COLUMN feature_version TEXT NOT NULL DEFAULT '0.1.0',
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE prompt_templates
    ADD COLUMN model_type TEXT NOT NULL DEFAULT 'llm'
        CHECK (model_type IN ('embedding', 'mlm', 'reranker', 'llm')),
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;

INSERT INTO model_registry
    (model_name, model_version, revision, model_type, feature_version, is_active)
VALUES
    ('bert-base-chinese', 'bert-base-chinese@main', 'main', 'mlm', '0.1.0', TRUE),
    ('BAAI/bge-reranker-base', 'BAAI/bge-reranker-base@main', 'main', 'reranker', '0.1.0', TRUE)
ON CONFLICT (model_version) DO NOTHING;
