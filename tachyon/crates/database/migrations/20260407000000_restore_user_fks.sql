-- Restore foreign key constraints referencing users table
-- These were dropped in migrations 20260330000000 and 20260401000000 because the
-- auth system was demo-only and users were not persisted. User persistence is now
-- implemented, so these FKs are valid again.
--
-- ON DELETE SET NULL is used for author/created_by columns so that deleting a user
-- does not cascade-delete their documents, versions, or attachments.
-- ON DELETE CASCADE is used for sessions.user_id so sessions are cleaned up.

BEGIN;

-- documents.author_id -> users(id)
-- Original schema had NOT NULL; keep nullable for system-generated documents.
DO $$ BEGIN
    ALTER TABLE documents ADD CONSTRAINT documents_author_id_fkey
        FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE SET NULL;
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- document_versions.created_by -> users(id)
DO $$ BEGIN
    ALTER TABLE document_versions ADD CONSTRAINT document_versions_created_by_fkey
        FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- attachments.created_by -> users(id)
DO $$ BEGIN
    ALTER TABLE attachments ADD CONSTRAINT attachments_created_by_fkey
        FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- sessions.user_id -> users(id)
DO $$ BEGIN
    ALTER TABLE sessions ADD CONSTRAINT sessions_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- Index on sessions.user_id (should already exist from 20260403000000)
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);

COMMIT;
