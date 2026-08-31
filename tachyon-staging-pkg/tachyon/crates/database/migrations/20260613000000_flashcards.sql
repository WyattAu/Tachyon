-- Flashcards and spaced repetition (FSRS)
CREATE TABLE IF NOT EXISTS flashcards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_flashcards_document_id ON flashcards(document_id);
CREATE INDEX IF NOT EXISTS idx_flashcards_created_at ON flashcards(created_at);

CREATE TABLE IF NOT EXISTS flashcard_review_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flashcard_id UUID NOT NULL REFERENCES flashcards(id) ON DELETE CASCADE,
    rating SMALLINT NOT NULL CHECK (rating BETWEEN 0 AND 3),
    reviewed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_flashcard_review_log_flashcard_id ON flashcard_review_log(flashcard_id);
CREATE INDEX IF NOT EXISTS idx_flashcard_review_log_reviewed_at ON flashcard_review_log(reviewed_at);

CREATE TABLE IF NOT EXISTS flashcard_srs_state (
    flashcard_id UUID PRIMARY KEY REFERENCES flashcards(id) ON DELETE CASCADE,
    state SMALLINT NOT NULL DEFAULT 0 CHECK (state BETWEEN 0 AND 3),
    step SMALLINT NOT NULL DEFAULT 0,
    stability DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    difficulty DOUBLE PRECISION NOT NULL DEFAULT 0.3,
    due TIMESTAMPTZ NOT NULL DEFAULT now(),
    reps INTEGER NOT NULL DEFAULT 0,
    lapses INTEGER NOT NULL DEFAULT 0,
    last_review TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
