ALTER TABLE sources DROP CONSTRAINT IF EXISTS sources_copyright_status_check;
ALTER TABLE sources ADD CONSTRAINT sources_copyright_status_check CHECK (
    copyright_status IN (
        'verified',
        'partially_verified',
        'unverifiable',
        'rejected',
        'legal_review_required'
    )
);

ALTER TABLE sources
    ADD COLUMN IF NOT EXISTS is_storable BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS is_trainable BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS is_publishable BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

ALTER TABLE sources ADD CONSTRAINT sources_publishable_authorization_check CHECK (
    NOT is_publishable OR copyright_status = 'verified'
);
