-- Drop remaining foreign key constraints referencing users table
-- The auth system is demo-only and does not persist users to the database.
-- All user IDs are generated at login time and stored only in JWT tokens.
-- Re-add these constraints once user persistence is implemented.

BEGIN;

-- document_versions.created_by references users(id)
ALTER TABLE document_versions DROP CONSTRAINT IF EXISTS document_versions_created_by_fkey;

-- attachments.created_by references users(id)
ALTER TABLE attachments DROP CONSTRAINT IF EXISTS attachments_created_by_fkey;

COMMIT;
