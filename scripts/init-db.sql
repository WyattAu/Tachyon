-- Tachyon PostgreSQL Database Initialization
-- This script runs on first database creation

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Optional: Enable Apache AGE for graph queries
-- CREATE EXTENSION IF NOT EXISTS age;

-- Grant permissions to tachyon user
GRANT ALL PRIVILEGES ON DATABASE tachyon TO tachyon;
GRANT ALL PRIVILEGES ON SCHEMA public TO tachyon;

-- Create a simple test table to verify setup
CREATE TABLE IF NOT EXISTS _db_setup_test (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

INSERT INTO _db_setup_test (created_at) VALUES (NOW());

-- Log successful setup
DO $$
BEGIN
    RAISE NOTICE 'Tachyon database initialized successfully';
END $$;
