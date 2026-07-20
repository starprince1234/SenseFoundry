ALTER TABLE corpus_submissions
    ADD COLUMN rejection_reason TEXT;

ALTER TABLE corpus_submissions
    ADD CONSTRAINT corpus_submissions_rejection_reason_check CHECK (
        (status = 'rejected' AND rejection_reason IS NOT NULL)
        OR (status <> 'rejected' AND rejection_reason IS NULL)
    );
