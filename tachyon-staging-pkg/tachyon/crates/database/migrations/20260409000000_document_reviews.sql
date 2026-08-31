-- Document Reviews
-- Review workflow for document versions with approval/rejection

CREATE TABLE IF NOT EXISTS document_reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    version_number INT NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'approved', 'rejected', 'changes_requested', 'cancelled')),
    reviewer_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_document_reviews_document_id
    ON document_reviews(document_id);

CREATE INDEX IF NOT EXISTS idx_document_reviews_status
    ON document_reviews(status);

CREATE INDEX IF NOT EXISTS idx_document_reviews_reviewer_id
    ON document_reviews(reviewer_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_document_reviews_unique_open
    ON document_reviews(document_id, reviewer_id)
    WHERE status = 'pending';

-- Review Comments
-- Threaded comments on document reviews

CREATE TABLE IF NOT EXISTS review_comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    review_id UUID NOT NULL REFERENCES document_reviews(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_review_comments_review_id
    ON review_comments(review_id);

CREATE INDEX IF NOT EXISTS idx_review_comments_author_id
    ON review_comments(author_id);
