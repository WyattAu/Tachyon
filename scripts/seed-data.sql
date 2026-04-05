-- Tachyon Seed Data
-- Run this script to populate the database with test data
-- Usage: docker exec -i tachyon-postgres psql -U tachyon -d tachyon < seed-data.sql

-- ============================================================================
-- Test Users
-- ============================================================================

-- Insert test users (passwords are bcrypt hashes of 'password123')
-- Note: These match the seed users in the auth code
INSERT INTO users (id, username, email, display_name, password_hash, role, status, settings, metadata)
VALUES 
    ('00000000-0000-0000-0000-000000000001', 'admin', 'admin@tachyon.local', 'Administrator', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4wq.4j6vBQYJjJyW', 'admin', 'active', '{}', '{"description": "System administrator"}'),
    ('00000000-0000-0000-0000-000000000002', 'guest', 'guest@tachyon.local', 'Guest User', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4wq.4j6vBQYJjJyW', 'reader', 'active', '{}', '{"description": "Guest user for public access"}'),
    ('00000000-0000-0000-0000-000000000003', 'editor', 'editor@tachyon.local', 'Editor User', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4wq.4j6vBQYJjJyW', 'editor', 'active', '{}', '{"description": "Content editor"}'),
    ('00000000-0000-0000-0000-000000000004', 'developer', 'developer@tachyon.local', 'Developer User', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4wq.4j6vBQYJjJyW', 'writer', 'active', '{}', '{"description": "Developer account"}')
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Organizations
-- ============================================================================

INSERT INTO organizations (id, name, slug, description, settings)
VALUES 
    ('10000000-0000-0000-0000-000000000001', 'Tachyon Labs', 'tachyon-labs', 'Tachyon Development Organization', '{"website": "https://tachyon.dev"}'),
    ('10000000-0000-0000-0000-000000000002', 'Acme Corp', 'acme-corp', 'Acme Corporation - Demo Organization', '{"website": "https://acme.example.com"}')
ON CONFLICT (id) DO NOTHING;

-- Add users to organizations
INSERT INTO organization_members (organization_id, user_id, role)
VALUES 
    ('10000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000001', 'owner'),
    ('10000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000003', 'member'),
    ('10000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000004', 'owner')
ON CONFLICT (organization_id, user_id) DO NOTHING;

-- ============================================================================
-- Projects (Backstage-like Catalog)
-- ============================================================================

INSERT INTO projects (
    id, name, slug, description, project_type, owner_id, organization_id,
    lifecycle, repository_url, docs_url, api_url, tags, metadata,
    language, framework, visibility, status
)
VALUES 
    -- Tachyon Platform - Main Project
    (
        '20000000-0000-0000-0000-000000000001',
        'Tachyon Platform',
        'tachyon-platform',
        'High-performance knowledge management platform built with Rust and Leptos',
        'website',
        '00000000-0000-0000-0000-000000000001',
        '10000000-0000-0000-0000-000000000001',
        'production',
        'https://github.com/tachyon-labs/tachyon',
        'https://docs.tachyon.dev',
        'https://api.tachyon.dev',
        '["rust", "leptos", "knowledge-management", "documentation"]',
        '{"version": "0.2.0", "tier": "core"}',
        'Rust',
        'Leptos',
        'public',
        'active'
    ),
    -- API Gateway
    (
        '20000000-0000-0000-0000-000000000002',
        'API Gateway',
        'api-gateway',
        'Central API gateway for microservices with authentication and rate limiting',
        'service',
        '00000000-0000-0000-0000-000000000003',
        '10000000-0000-0000-0000-000000000001',
        'production',
        'https://github.com/tachyon-labs/api-gateway',
        'https://api-gateway.docs.tachyon.dev',
        NULL,
        '["rust", "api", "gateway", "microservices"]',
        '{"version": "1.0.0", "tier": "infrastructure"}',
        'Rust',
        'Axum',
        'internal',
        'active'
    ),
    -- Documentation Portal
    (
        '20000000-0000-0000-0000-000000000003',
        'Documentation Portal',
        'docs-portal',
        'Public-facing documentation site for Tachyon Platform',
        'documentation',
        '00000000-0000-0000-0000-000000000004',
        '10000000-0000-0000-0000-000000000001',
        'production',
        'https://github.com/tachyon-labs/docs',
        'https://tachyon.dev',
        NULL,
        '["documentation", "website", "mdbook"]',
        '{"version": "latest", "tier": "user-facing"}',
        'Markdown',
        'mdBook',
        'public',
        'active'
    ),
    -- CLI Tool
    (
        '20000000-0000-0000-0000-000000000004',
        'Tachyon CLI',
        'tachyon-cli',
        'Command-line interface for Tachyon Platform management',
        'library',
        '00000000-0000-0000-0000-000000000001',
        '10000000-0000-0000-0000-000000000001',
        'development',
        'https://github.com/tachyon-labs/tachyon-cli',
        NULL,
        NULL,
        '["rust", "cli", "tool"]',
        '{"version": "0.1.0", "tier": "tooling"}',
        'Rust',
        'Clap',
        'public',
        'active'
    ),
    -- Analytics Dashboard
    (
        '20000000-0000-0000-0000-000000000005',
        'Analytics Dashboard',
        'analytics-dashboard',
        'Real-time analytics and monitoring dashboard',
        'website',
        '00000000-0000-0000-0000-000000000003',
        '10000000-0000-0000-0000-000000000001',
        'experimental',
        'https://github.com/tachyon-labs/analytics',
        NULL,
        NULL,
        '["rust", "leptos", "analytics", "dashboard"]',
        '{"version": "0.0.1", "tier": "experimental"}',
        'Rust',
        'Leptos',
        'internal',
        'active'
    ),
    -- Mobile App (Demo)
    (
        '20000000-0000-0000-0000-000000000006',
        'Mobile App',
        'tachyon-mobile',
        'Mobile companion app for Tachyon Platform',
        'mobile-app',
        '00000000-0000-0000-0000-000000000004',
        '10000000-0000-0000-0000-000000000002',
        'development',
        'https://github.com/acme-corp/tachyon-mobile',
        NULL,
        NULL,
        '["mobile", "react-native", "ios", "android"]',
        '{"version": "0.5.0", "tier": "client"}',
        'TypeScript',
        'React Native',
        'internal',
        'active'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Components
-- ============================================================================

INSERT INTO components (
    id, name, component_type, project_id, owner_id, system_id,
    repository_url, docs_url, api_spec_url, tags, lifecycle
)
VALUES 
    -- Core Components
    (
        '30000000-0000-0000-0000-000000000001',
        'Document Parser',
        'service',
        '20000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        NULL,
        'https://github.com/tachyon-labs/tachyon/tree/main/crates/parser',
        NULL,
        NULL,
        '["rust", "parsing", "markdown"]',
        'production'
    ),
    (
        '30000000-0000-0000-0000-000000000002',
        'Search Engine',
        'service',
        '20000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        NULL,
        'https://github.com/tachyon-labs/tachyon/tree/main/crates/search',
        NULL,
        NULL,
        '["rust", "search", "tantivy"]',
        'production'
    ),
    (
        '30000000-0000-0000-0000-000000000003',
        'Database Layer',
        'service',
        '20000000-0000-0000-0000-000000000001',
        '00000000-0000-0000-0000-000000000001',
        NULL,
        'https://github.com/tachyon-labs/tachyon/tree/main/crates/database',
        NULL,
        NULL,
        '["rust", "database", "postgresql"]',
        'production'
    ),
    (
        '30000000-0000-0000-0000-000000000004',
        'Auth Service',
        'service',
        '20000000-0000-0000-0000-000000000002',
        '00000000-0000-0000-0000-000000000003',
        NULL,
        'https://github.com/tachyon-labs/api-gateway/tree/main/crates/auth',
        NULL,
        NULL,
        '["rust", "authentication", "jwt"]',
        'production'
    ),
    (
        '30000000-0000-0000-0000-000000000005',
        'Rate Limiter',
        'service',
        '20000000-0000-0000-0000-000000000002',
        '00000000-0000-0000-0000-000000000003',
        NULL,
        'https://github.com/tachyon-labs/api-gateway/tree/main/crates/ratelimit',
        NULL,
        NULL,
        '["rust", "rate-limiting", "redis"]',
        'production'
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Note: project_members table not yet implemented
-- Project ownership is tracked via owner_id in projects table
-- ============================================================================

-- ============================================================================
-- Sample Documents
-- ============================================================================

INSERT INTO documents (id, title, slug, author_id, project_id, description, tags, visibility, status, content_type, content, word_count, character_count)
VALUES 
    (
        '40000000-0000-0000-0000-000000000001',
        'Getting Started with Tachyon',
        'getting-started',
        '00000000-0000-0000-0000-000000000001',
        '20000000-0000-0000-0000-000000000001',
        'Quick start guide for setting up Tachyon Platform',
        '["tutorial", "setup", "beginner"]',
        'public',
        'published',
        'markdown',
        '# Getting Started with Tachyon

Welcome to Tachyon! This guide will help you get up and running quickly.

## Prerequisites

- Rust 1.75 or later
- PostgreSQL 15 or later
- Node.js 18+ (for frontend development)

## Installation

1. Clone the repository
2. Run `cargo build --release`
3. Configure your database
4. Start the server with `cargo run --release`

## Next Steps

Check out the full documentation for more details.',
        75,
        450
    ),
    (
        '40000000-0000-0000-0000-000000000002',
        'API Reference',
        'api-reference',
        '00000000-0000-0000-0000-000000000003',
        '20000000-0000-0000-0000-000000000001',
        'Complete API reference for Tachyon Platform',
        '["api", "reference", "documentation"]',
        'public',
        'published',
        'markdown',
        '# API Reference

## Authentication

All API requests require authentication via Bearer token.

## Endpoints

### Users
- `GET /api/v1/users` - List users
- `POST /api/v1/users` - Create user
- `GET /api/v1/users/:id` - Get user

### Documents
- `GET /api/v1/documents` - List documents
- `POST /api/v1/documents` - Create document
- `GET /api/v1/documents/:id` - Get document

### Catalog
- `GET /api/v1/catalog/stats` - Get catalog statistics
- `GET /api/v1/projects` - List projects
- `POST /api/v1/projects` - Create project',
        95,
        570
    )
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Verify Data
-- ============================================================================

DO $$
DECLARE
    user_count INTEGER;
    project_count INTEGER;
    component_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO user_count FROM users;
    SELECT COUNT(*) INTO project_count FROM projects;
    SELECT COUNT(*) INTO component_count FROM components;
    
    RAISE NOTICE 'Seed data loaded successfully!';
    RAISE NOTICE '  Users: %', user_count;
    RAISE NOTICE '  Projects: %', project_count;
    RAISE NOTICE '  Components: %', component_count;
END $$;
