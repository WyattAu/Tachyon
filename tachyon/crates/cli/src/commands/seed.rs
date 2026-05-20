use crate::error::{CliError, CliResult};
use chrono::{Duration, Utc};
use rand::SeedableRng;
use rand::prelude::*;
use rand::rngs::StdRng;
use serde_json::json;
use std::time::Instant;

const SEED: u64 = 42;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedScale {
    Small,
    Medium,
    Large,
    Production,
}

impl SeedScale {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "small" => Some(Self::Small),
            "medium" => Some(Self::Medium),
            "large" => Some(Self::Large),
            "production" => Some(Self::Production),
            _ => None,
        }
    }
}

pub struct SeedConfig {
    pub database_url: String,
    pub scale: SeedScale,
    pub clear: bool,
    pub dry_run: bool,
}

struct SeedCounts {
    users: usize,
    organizations: usize,
    teams: usize,
    spaces: usize,
    documents: usize,
    comments: usize,
    saved_searches: usize,
    notifications: usize,
    sessions: usize,
    graph_edges: usize,
}

impl SeedScale {
    fn counts(&self) -> SeedCounts {
        match self {
            SeedScale::Small => SeedCounts {
                users: 10,
                organizations: 2,
                teams: 3,
                spaces: 5,
                documents: 50,
                comments: 100,
                saved_searches: 20,
                notifications: 30,
                sessions: 15,
                graph_edges: 25,
            },
            SeedScale::Medium => SeedCounts {
                users: 100,
                organizations: 5,
                teams: 10,
                spaces: 20,
                documents: 1000,
                comments: 5000,
                saved_searches: 500,
                notifications: 2000,
                sessions: 150,
                graph_edges: 500,
            },
            SeedScale::Large => SeedCounts {
                users: 1000,
                organizations: 10,
                teams: 30,
                spaces: 100,
                documents: 10000,
                comments: 50000,
                saved_searches: 5000,
                notifications: 10000,
                sessions: 1000,
                graph_edges: 5000,
            },
            SeedScale::Production => SeedCounts {
                users: 10000,
                organizations: 50,
                teams: 100,
                spaces: 500,
                documents: 100000,
                comments: 500000,
                saved_searches: 20000,
                notifications: 50000,
                sessions: 5000,
                graph_edges: 20000,
            },
        }
    }
}

// ============================================================================
// Fake Data Generators
// ============================================================================

const FIRST_NAMES: &[&str] = &[
    "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Hank", "Iris", "Jack", "Kate",
    "Leo", "Maya", "Noah", "Olivia", "Pete", "Quinn", "Rosa", "Sam", "Tina", "Uma", "Vera", "Will",
    "Xena", "Yuri", "Zara", "Alex", "Blake", "Casey", "Dana", "Elliot", "Faye", "Glen", "Hana",
    "Ivan", "Jade", "Karl", "Luna", "Max", "Nora", "Omar", "Piper", "Rex", "Sage", "Troy", "Uma",
    "Vince", "Wren", "Xander", "Yuki",
];

const LAST_NAMES: &[&str] = &[
    "Smith",
    "Johnson",
    "Williams",
    "Brown",
    "Jones",
    "Garcia",
    "Miller",
    "Davis",
    "Rodriguez",
    "Martinez",
    "Hernandez",
    "Lopez",
    "Gonzalez",
    "Wilson",
    "Anderson",
    "Thomas",
    "Taylor",
    "Moore",
    "Jackson",
    "Martin",
    "Lee",
    "Perez",
    "Thompson",
    "White",
    "Harris",
    "Sanchez",
    "Clark",
    "Ramirez",
    "Lewis",
    "Robinson",
    "Walker",
    "Young",
    "Allen",
    "King",
    "Wright",
    "Scott",
    "Torres",
    "Nguyen",
    "Hill",
    "Flores",
    "Green",
    "Adams",
    "Nelson",
    "Baker",
    "Hall",
    "Rivera",
    "Campbell",
    "Mitchell",
    "Carter",
    "Roberts",
];

const COMPANY_NAMES: &[&str] = &[
    "Acme Corp",
    "Globex Inc",
    "Initech",
    "Umbrella Co",
    "Stark Industries",
    "Wayne Enterprises",
    "Cyberdyne Systems",
    "Oscorp Industries",
    "Aperture Science",
    "Soylent Corp",
    "Massive Dynamic",
    "Hooli",
    "Pied Piper",
    "Dunder Mifflin",
    "Sterling Cooper",
    "Prestige Worldwide",
    "Bluth Company",
    "Los Pollos Hermanos",
    "Vortex Industries",
    "Nexus Labs",
    "Quantum Dynamics",
    "Atlas Corp",
    "Zenith Tech",
    "Prism Solutions",
    "Helix Systems",
    "Forge Works",
    "Catalyst Inc",
    "Beacon Labs",
    "Summit Software",
    "Vertex AI",
    "Ember Tech",
    "Flux Dynamics",
    "Pinnacle Systems",
    "Horizon Labs",
    "Nova Corp",
    "Apex Innovations",
    "Orbit Systems",
    "Meridian Tech",
    "Cobalt Solutions",
    "Onyx Labs",
    "Sapphire Networks",
    "Jade Computing",
    "Opal Tech",
    "Ruby Systems",
    "Quartz Analytics",
    "Granite Infrastructure",
    "Slate Platforms",
];

const TEAM_NAMES: &[&str] = &[
    "Engineering",
    "Product",
    "Design",
    "DevOps",
    "Security",
    "Data Science",
    "Frontend",
    "Backend",
    "Mobile",
    "Infrastructure",
    "QA",
    "Documentation",
    "Platform",
    "Research",
    "Analytics",
    "Growth",
    "Marketing",
    "Sales Engineering",
    "Customer Success",
    "Developer Relations",
    "Site Reliability",
    "Machine Learning",
    "Cloud Architecture",
    "API Team",
    "Core Services",
    "Integrations",
    "Performance",
    "Accessibility",
    "Observability",
    "Compliance",
];

const PROJECT_NAMES: &[&str] = &[
    "API Gateway",
    "User Dashboard",
    "Analytics Engine",
    "Data Pipeline",
    "Authentication Service",
    "Notification System",
    "Search Platform",
    "Content Management",
    "Billing System",
    "Deployment Pipeline",
    "Monitoring Stack",
    "CI/CD Framework",
    "Design System",
    "Mobile App",
    "Real-time Chat",
    "Document Editor",
    "Workflow Engine",
    "Identity Provider",
    "Asset Manager",
    "Configuration Service",
    "Event Processing",
    "Cache Layer",
    "Migration Tool",
    "Feature Flags",
    "A/B Testing",
    "Rate Limiter",
    "Logging Infrastructure",
    "Secrets Manager",
    "Load Balancer",
    "Service Mesh",
    "GraphQL Gateway",
    "REST Microservices",
    "Kubernetes Operator",
    "Terraform Modules",
    "Security Scanner",
    "Vulnerability Tracker",
    "Compliance Dashboard",
    "Audit System",
    "Reporting Engine",
    "Data Warehouse",
    "ML Training Pipeline",
    "Model Serving",
    "Recommendation Engine",
    "Personalization Service",
    "Search Relevance",
    "Content Delivery",
    "Edge Computing",
    "Serverless Functions",
    "WebSocket Hub",
    "Email Service",
    "SMS Gateway",
    "Push Notifications",
    "File Storage",
];

const TAG_VOCABULARY: &[&str] = &[
    "rust",
    "typescript",
    "python",
    "golang",
    "java",
    "kubernetes",
    "docker",
    "aws",
    "gcp",
    "azure",
    "postgres",
    "redis",
    "kafka",
    "graphql",
    "rest",
    "microservices",
    "serverless",
    "ci-cd",
    "devops",
    "security",
    "performance",
    "testing",
    "monitoring",
    "logging",
    "observability",
    "authentication",
    "authorization",
    "encryption",
    "caching",
    "load-balancing",
    "database",
    "migration",
    "api-design",
    "event-driven",
    "streaming",
    "batch-processing",
    "machine-learning",
    "data-science",
    "analytics",
    "visualization",
    "documentation",
    "onboarding",
    "architecture",
    "design-patterns",
    "best-practices",
    "troubleshooting",
    "performance-optimization",
    "scalability",
    "reliability",
    "fault-tolerance",
    "backup",
    "disaster-recovery",
    "compliance",
    "gdpr",
    "accessibility",
    "internationalization",
];

const DOCUMENT_TITLES: &[&str] = &[
    "Getting Started with Rust",
    "Advanced TypeScript Patterns",
    "Building Scalable Microservices",
    "Database Migration Best Practices",
    "Kubernetes Production Checklist",
    "API Design Guidelines",
    "Performance Optimization Techniques",
    "Security Audit Procedures",
    "CI/CD Pipeline Configuration",
    "Monitoring and Alerting Setup",
    "Incident Response Playbook",
    "Architecture Decision Records Template",
    "Code Review Guidelines",
    "Onboarding New Engineers",
    "Infrastructure as Code with Terraform",
    "GraphQL Schema Design",
    "Event-Driven Architecture Patterns",
    "Real-time Data Processing",
    "Authentication and Authorization Guide",
    "OAuth 2.0 Implementation",
    "Rate Limiting Strategies",
    "Caching Architecture Overview",
    "Database Indexing Strategies",
    "Query Optimization Tips",
    "Container Orchestration Patterns",
    "Service Mesh Configuration",
    "Load Testing Methodology",
    "Chaos Engineering Handbook",
    "Feature Flag Implementation",
    "A/B Testing Framework",
    "Log Aggregation Pipeline",
    "Distributed Tracing Setup",
    "Error Handling Best Practices",
    "Async Programming in Rust",
    "WebAssembly Build Pipeline",
    "Webhook Integration Guide",
    "Third-Party API Integration",
    "Data Pipeline Architecture",
    "ETL Process Documentation",
    "Data Warehouse Schema Design",
    "Machine Learning Model Deployment",
    "Model Monitoring Framework",
    "Feature Store Architecture",
    "Recommendation System Design",
    "Search Engine Optimization",
    "Full-Text Search Implementation",
    "Content Delivery Network Setup",
    "Edge Computing Architecture",
    "Mobile App Backend Design",
    "Push Notification Service",
    "Real-time Collaboration Architecture",
    "Conflict Resolution Strategies",
    "Offline-First Application Design",
    "Progressive Web App Guide",
    "Accessibility Compliance Guide",
    "Internationalization Checklist",
    "Performance Budget Guidelines",
    "Frontend Build Optimization",
    "State Management Patterns",
    "Component Library Documentation",
    "Design Token System",
    "Responsive Design Best Practices",
];

const MARKDOWN_PARAGRAPHS: &[&str] = &[
    "This section provides a comprehensive overview of the architecture and design decisions that guide our implementation. We focus on maintainability, performance, and developer experience while ensuring the system remains robust under production workloads.",
    "The following diagram illustrates the data flow between components. Each service communicates via well-defined interfaces, enabling independent scaling and deployment. Error handling follows the circuit breaker pattern to prevent cascading failures.",
    "## Key Considerations\n\nWhen implementing this feature, consider the following trade-offs:\n\n1. **Latency vs. Consistency**: Choose the appropriate consistency model based on your use case.\n2. **Complexity vs. Flexibility**: Start simple and iterate based on real-world usage patterns.\n3. **Cost vs. Performance**: Optimize hot paths first, then consider caching strategies.",
    "Performance benchmarks show that the optimized implementation handles 10x the throughput compared to the baseline. Key improvements include connection pooling, query batching, and parallel processing of independent operations.",
    "### Error Handling\n\nAll operations return a `Result` type, making error paths explicit. Common error cases include:\n\n- Network timeouts (configurable retry with exponential backoff)\n- Validation failures (structured error messages with field-level details)\n- Resource exhaustion (graceful degradation with queue-based buffering)",
    "The testing strategy encompasses three levels: unit tests for individual functions, integration tests for component interactions, and end-to-end tests that verify complete user workflows. We aim for >80% code coverage on critical paths.",
    "## Migration Guide\n\nUpgrading from the previous version requires the following steps:\n\n1. Update your dependencies to the latest versions\n2. Run the database migration scripts in order\n3. Update configuration files with new settings\n4. Restart all services and verify health checks",
    "### Configuration\n\nEnvironment variables take precedence over config files. The following table lists all available configuration options with their defaults and descriptions.",
    "Security is a top priority. All data at rest is encrypted using AES-256, and data in transit uses TLS 1.3. Authentication tokens follow the JWT standard with short expiry times and secure refresh mechanisms.",
    "## Monitoring\n\nKey metrics to track include:\n\n- Request latency (p50, p95, p99)\n- Error rate by endpoint\n- Active connection count\n- Queue depth and processing time\n- Cache hit/miss ratios",
];

const COMMENT_TEMPLATES: &[&str] = &[
    "This looks great! I especially like the approach to handling edge cases.",
    "Have we considered the performance implications of this change? Let me run some benchmarks.",
    "I think we should add more documentation here. The behavior isn't immediately obvious.",
    "Nit: could we rename this variable to be more descriptive?",
    "This matches what we discussed in the architecture review. LGTM!",
    "Could we add a test case for when the input is empty?",
    "I noticed a potential race condition here. We should use a mutex or atomic operation.",
    "Great refactor! This is much cleaner than the previous implementation.",
    "Can we break this into smaller functions for better readability?",
    "The error message could be more helpful for users. Something like: 'Expected X but got Y'.",
    "Should we handle the case where the service is unavailable gracefully?",
    "This is a solid foundation. I'd suggest adding metrics to track usage patterns.",
    "Is this compatible with the existing API contract? We should check backwards compatibility.",
    "Would it make sense to cache this result? It looks like it's called frequently.",
    "Let's schedule a follow-up to discuss the long-term direction here.",
    "I've tested this locally and it works well. Ready for review.",
    "One concern: this could be a bottleneck at scale. Have we load-tested?",
    "Nice catch on the off-by-one error. The fix looks correct.",
    "Could we add logging at the debug level for troubleshooting?",
    "This resolves the issue we saw in production last week. Good fix!",
    "I'd suggest extracting this into a shared utility module.",
    "The tests cover the happy path well. Let's add some error scenarios.",
    "Agreed with the approach. Let's also document the rationale for future maintainers.",
    "Small suggestion: use `const` instead of `let` for the configuration values.",
];

const SEARCH_QUERIES: &[&str] = &[
    "rust async patterns",
    "kubernetes deployment guide",
    "typescript generics",
    "database optimization",
    "api authentication",
    "docker best practices",
    "microservices communication",
    "graphql schema design",
    "redis caching strategies",
    "postgresql indexing",
    "ci cd pipeline setup",
    "monitoring alerting rules",
    "security vulnerabilities",
    "performance profiling",
    "load testing tools",
    "error handling patterns",
    "logging configuration",
    "feature flag rollout",
    "oauth implementation",
    "rate limiting design",
    "webhook integration",
    "data pipeline architecture",
    "machine learning deployment",
    "search relevance",
    "content management system",
    "user authentication flow",
    "authorization model",
    "encryption at rest",
    "backup and recovery",
    "disaster recovery plan",
    "capacity planning",
    "cost optimization",
    "infrastructure automation",
];

struct FakeData {
    rng: StdRng,
}

impl FakeData {
    fn new() -> Self {
        Self {
            rng: StdRng::seed_from_u64(SEED),
        }
    }

    fn gen_user(&mut self, index: usize) -> (String, String, String, String, bool) {
        let first = FIRST_NAMES[index % FIRST_NAMES.len()];
        let last = LAST_NAMES[self.rng.gen_range(0..LAST_NAMES.len())];
        let username = format!("{}{}{}", first.to_lowercase(), last.to_lowercase(), index);
        let display_name = format!("{} {}", first, last);
        let email = format!(
            "{}.{}{}.seed.test@example.com",
            first.to_lowercase(),
            last.to_lowercase(),
            index
        );
        let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$c2VlZHNhbHQ$RMOdYqC3l3w5q2Pw4XDqwqYwrDDgcK9wrzCsQOwrbClw4nDlcKAwr3DkMO8YMOEwrfCsA==".to_string();
        let totp_enabled = index % 10 == 0;
        (username, display_name, email, password_hash, totp_enabled)
    }

    fn gen_org_name(&mut self, index: usize) -> String {
        COMPANY_NAMES[index % COMPANY_NAMES.len()].to_string()
    }

    fn gen_team_name(&mut self, index: usize) -> String {
        TEAM_NAMES[index % TEAM_NAMES.len()].to_string()
    }

    fn gen_space_name(&mut self, index: usize) -> String {
        let project = PROJECT_NAMES[index % PROJECT_NAMES.len()];
        if self.rng.gen_bool(0.3) {
            format!(
                "{} - {}",
                project,
                if self.rng.r#gen() { "Docs" } else { "Wiki" }
            )
        } else {
            project.to_string()
        }
    }

    fn gen_document(
        &mut self,
        index: usize,
        _author_idx: usize,
    ) -> (String, String, String, String, i32, i32, String, String) {
        let title = DOCUMENT_TITLES[index % DOCUMENT_TITLES.len()].to_string();
        let suffix = if index >= DOCUMENT_TITLES.len() {
            format!(" (v{})", index / DOCUMENT_TITLES.len() + 1)
        } else {
            String::new()
        };
        let title = format!("{}{}", title, suffix);
        let slug = title
            .to_lowercase()
            .replace(' ', "-")
            .replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "");

        let statuses = ["draft", "published", "published", "published", "archived"];
        let status = statuses[self.rng.gen_range(0..statuses.len())].to_string();

        let visibilities = ["private", "public", "internal"];
        let visibility = visibilities[self.rng.gen_range(0..visibilities.len())].to_string();

        let content_types = ["markdown", "markdown", "markdown", "asciidoc", "mdx"];
        let content_type = content_types[self.rng.gen_range(0..content_types.len())].to_string();

        let num_paragraphs = self.rng.gen_range(1..=4);
        let mut content = String::new();
        content.push_str(&format!("# {}\n\n", title));
        for p_idx in 0..num_paragraphs {
            content.push_str(MARKDOWN_PARAGRAPHS[(index + p_idx) % MARKDOWN_PARAGRAPHS.len()]);
            content.push_str("\n\n");
        }
        let word_count = content.split_whitespace().count() as i32;
        let character_count = content.len() as i32;
        let _read_count = if status == "published" {
            self.rng.gen_range(0..500)
        } else {
            0
        };
        let _edit_count = self.rng.gen_range(1..10);

        (
            title,
            slug,
            status,
            visibility,
            word_count,
            character_count,
            content,
            content_type,
        )
    }

    fn gen_tags(&mut self) -> Vec<String> {
        let count = self.rng.gen_range(1..=5);
        let mut tags: Vec<String> = TAG_VOCABULARY
            .choose_multiple(&mut self.rng, count)
            .map(|s| s.to_string())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    fn gen_comment(&mut self, doc_idx: usize, _author_idx: usize) -> String {
        let template = COMMENT_TEMPLATES[doc_idx % COMMENT_TEMPLATES.len()];
        if self.rng.gen_bool(0.3) {
            format!(
                "{} {}",
                template,
                if self.rng.r#gen() { ":+1:" } else { "Thanks!" }
            )
        } else {
            template.to_string()
        }
    }

    fn gen_search_query(&mut self, index: usize) -> (String, String) {
        let query = SEARCH_QUERIES[index % SEARCH_QUERIES.len()].to_string();
        let name = format!("Search: {}", &query[..query.len().min(30)]);
        (name, query)
    }

    fn gen_notification(
        &mut self,
        user_idx: usize,
        doc_idx: usize,
    ) -> (String, String, Option<String>, Option<String>) {
        let types = [
            "comment",
            "mention",
            "document_updated",
            "review_requested",
            "system",
            "share",
        ];
        let titles = [
            "New comment on your document".to_string(),
            format!("You were mentioned by user{}", user_idx % 10),
            format!(
                "Document \"{}\" was updated",
                DOCUMENT_TITLES[doc_idx % DOCUMENT_TITLES.len()]
            ),
            "Review requested for document".to_string(),
            "System maintenance scheduled".to_string(),
            "A document was shared with you".to_string(),
        ];
        let idx = self.rng.gen_range(0..types.len());
        let notification_type = types[idx].to_string();
        let title = titles[idx].clone();
        let body = if self.rng.gen_bool(0.6) {
            Some(format!("Details for notification: {}", title))
        } else {
            None
        };
        let link = if self.rng.gen_bool(0.5) {
            Some(format!("/documents/{}", doc_idx))
        } else {
            None
        };
        (notification_type, title, body, link)
    }

    fn gen_session(
        &mut self,
        expired: bool,
    ) -> (
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
    ) {
        let session_types = ["web", "desktop", "api", "mobile"];
        let user_agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
            "TachyonCLI/0.1.0",
        ];
        let devices = ["Desktop", "Laptop", "Mobile", "Tablet", "CLI"];
        let ips = [
            "10.0.0.1",
            "10.0.0.2",
            "192.168.1.100",
            "172.16.0.50",
            "10.10.10.10",
        ];

        let session_type = session_types[self.rng.gen_range(0..session_types.len())].to_string();
        let status = if expired { "expired" } else { "active" };
        let user_agent = user_agents[self.rng.gen_range(0..user_agents.len())].to_string();
        let device = devices[self.rng.gen_range(0..devices.len())].to_string();
        let ip = ips[self.rng.gen_range(0..ips.len())].to_string();
        let token_value = uuid::Uuid::new_v4().to_string();

        (
            session_type,
            status.to_string(),
            user_agent,
            device,
            Some(ip),
            Some(token_value),
        )
    }

    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        self.rng.gen_range(min..max)
    }

    fn gen_bool(&mut self, probability: f64) -> bool {
        self.rng.gen_bool(probability)
    }
}

// ============================================================================
// Progress Reporter
// ============================================================================

struct ProgressReporter {
    label: String,
    total: usize,
    start: Instant,
    last_print: std::time::Instant,
}

impl ProgressReporter {
    fn new(label: &str, total: usize) -> Self {
        let start = Instant::now();
        let last_print = std::time::Instant::now();
        println!("Seeding {}... 0/{}", label, total);
        Self {
            label: label.to_string(),
            total,
            start,
            last_print,
        }
    }

    fn update(&mut self, current: usize) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_print).as_millis() < 200 {
            return;
        }
        self.last_print = now;

        let elapsed = self.start.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 {
            current as f64 / elapsed
        } else {
            0.0
        };
        let remaining = if rate > 0.0 {
            (self.total - current) as f64 / rate
        } else {
            0.0
        };
        let eta_str = format_eta(remaining);
        print!(
            "\rSeeding {}... {current}/{} ({rate:.0} rows/s, ETA {})",
            self.label, self.total, eta_str,
        );
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }

    fn finish(&self) {
        let elapsed = self.start.elapsed();
        println!(
            "\rSeeding {}... {}/{} (done in {:.1}s)        ",
            self.label,
            self.total,
            self.total,
            elapsed.as_secs_f64(),
        );
    }
}

fn format_eta(seconds: f64) -> String {
    if seconds < 1.0 {
        return "<1s".to_string();
    }
    if seconds < 60.0 {
        return format!("{:.0}s", seconds);
    }
    let mins = seconds / 60.0;
    if mins < 60.0 {
        return format!("{:.0}m", mins);
    }
    format!("{:.0}h", mins / 60.0)
}

// ============================================================================
// SQL Generation for Dry Run
// ============================================================================

struct DryRunWriter {
    statements: Vec<String>,
}

impl DryRunWriter {
    fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    fn push(&mut self, sql: &str) {
        self.statements.push(sql.to_string());
    }
}

// ============================================================================
// Seed Execution
// ============================================================================

pub fn execute_seed(config: &SeedConfig) -> CliResult<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::database(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async { execute_seed_async(config).await })
}

async fn execute_seed_async(config: &SeedConfig) -> CliResult<()> {
    let counts = config.scale.counts();

    if config.dry_run {
        println!("=== Dry Run: Seed Plan (scale: {:?}) ===\n", config.scale);
        println!("{:<25} {:>10}", "Entity", "Count");
        println!("{}", "-".repeat(37));
        println!("{:<25} {:>10}", "Users", counts.users);
        println!("{:<25} {:>10}", "Organizations", counts.organizations);
        println!("{:<25} {:>10}", "Teams", counts.teams);
        println!("{:<25} {:>10}", "Spaces", counts.spaces);
        println!("{:<25} {:>10}", "Documents", counts.documents);
        println!("{:<25} {:>10}", "Comments", counts.comments);
        println!("{:<25} {:>10}", "Saved Searches", counts.saved_searches);
        println!("{:<25} {:>10}", "Notifications", counts.notifications);
        println!("{:<25} {:>10}", "Sessions", counts.sessions);
        println!("{:<25} {:>10}", "Graph Edges", counts.graph_edges);
        println!();

        let mut dry_run = DryRunWriter::new();
        generate_seed_sql(&mut dry_run, &counts);

        println!(
            "=== Generated SQL (first 50 statements, {} total) ===\n",
            dry_run.statements.len()
        );
        for stmt in dry_run.statements.iter().take(50) {
            let preview = if stmt.len() > 200 { &stmt[..200] } else { stmt };
            println!("{};", preview);
            if stmt.len() > 200 {
                println!("  ... ({} chars total)", stmt.len());
            }
        }
        if dry_run.statements.len() > 50 {
            println!(
                "\n... and {} more statements",
                dry_run.statements.len() - 50
            );
        }
        return Ok(());
    }

    let pool = tachyon_database::DatabasePool::new(&config.database_url)
        .await
        .map_err(|e| CliError::database(format!("Failed to connect: {}", e)))?;

    if config.clear {
        println!("Clearing existing data...");
        clear_tables(&pool).await?;
        println!("Data cleared.\n");
    }

    seed_database(&pool, &counts).await
}

async fn clear_tables(pool: &tachyon_database::DatabasePool) -> CliResult<()> {
    use sqlx::query;

    let tables = [
        "audit_log",
        "document_comments",
        "search_index",
        "graph_edges",
        "graph_nodes",
        "notifications",
        "sessions",
        "saved_searches",
        "document_versions",
        "attachments",
        "documents",
        "project_members",
        "components",
        "projects",
        "space_members",
        "spaces",
        "team_members",
        "teams",
        "organization_members",
        "organizations",
        "user_roles",
        "roles",
        "users",
    ];

    // SAFETY: Table names are compile-time constants defined in TABLES array above.
    for table in &tables {
        let sql = format!("DELETE FROM {} CASCADE", table);
        let mut conn = pool
            .acquire()
            .await
            .map_err(|e| CliError::database(e.to_string()))?;
        query(&sql)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("does not exist") {
                    CliError::database(format!("Table {} does not exist, skipping", table))
                } else {
                    CliError::database(e.to_string())
                }
            })
            .ok();
    }

    Ok(())
}

async fn seed_database(
    pool: &tachyon_database::DatabasePool,
    counts: &SeedCounts,
) -> CliResult<()> {
    let mut data = FakeData::new();
    let total_start = Instant::now();

    let user_ids = seed_users(pool, &mut data, counts.users).await?;
    let org_ids = seed_organizations(pool, &mut data, &user_ids, counts.organizations).await?;
    let _team_ids = seed_teams(pool, &mut data, &user_ids, &org_ids, counts.teams).await?;
    let _space_ids = seed_spaces(pool, &mut data, &user_ids, counts.spaces).await?;
    let doc_ids = seed_documents(pool, &mut data, &user_ids, counts.documents).await?;
    seed_comments(pool, &mut data, &doc_ids, &user_ids, counts.comments).await?;
    seed_saved_searches(pool, &mut data, &user_ids, counts.saved_searches).await?;
    seed_notifications(pool, &mut data, &user_ids, counts.notifications).await?;
    seed_sessions(pool, &mut data, &user_ids, counts.sessions).await?;
    seed_graph_edges(pool, &mut data, &doc_ids, counts.graph_edges).await?;

    println!(
        "\nSeed completed in {:.1}s. {} users, {} docs, {} comments.",
        total_start.elapsed().as_secs_f64(),
        counts.users,
        counts.documents,
        counts.comments,
    );

    Ok(())
}

async fn seed_users(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    count: usize,
) -> CliResult<Vec<uuid::Uuid>> {
    let mut reporter = ProgressReporter::new("users", count);
    let mut user_ids = Vec::with_capacity(count);

    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    for i in 0..count {
        let (username, display_name, email, password_hash, totp_enabled) = data.gen_user(i);
        let user_id = uuid::Uuid::new_v4();
        let roles = ["admin", "editor", "writer", "viewer", "viewer"];
        let role = if i == 0 {
            "admin"
        } else {
            roles[i % roles.len()]
        };

        let is_active = i != count.saturating_sub(1);

        sqlx::query(
            r#"INSERT INTO users (id, username, display_name, email, password_hash, role, user_type, is_active, totp_enabled, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, 'regular', $7, $8, NOW(), NOW())"#,
        )
        .bind(user_id)
        .bind(&username)
        .bind(&display_name)
        .bind(&email)
        .bind(&password_hash)
        .bind(role)
        .bind(is_active)
        .bind(totp_enabled)
        .execute(&mut *conn)
        .await
        .map_err(|e| CliError::database(format!("Failed to insert user {}: {}", username, e)))?;

        user_ids.push(user_id);
        reporter.update(i + 1);
    }
    reporter.finish();

    Ok(user_ids)
}

async fn seed_organizations(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    user_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<Vec<uuid::Uuid>> {
    if count == 0 {
        return Ok(vec![]);
    }

    let mut reporter = ProgressReporter::new("organizations", count);
    let mut org_ids = Vec::with_capacity(count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    for i in 0..count {
        let name = data.gen_org_name(i);
        let slug = slug::slugify(&name);
        let owner_id = user_ids[i % user_ids.len()];
        let org_id = uuid::Uuid::new_v4();
        let billing_plans = ["free", "pro", "enterprise"];
        let plan = billing_plans[i % billing_plans.len()];

        let sql = r#"INSERT INTO organizations (id, name, slug, description, icon, owner_id, default_role, max_members, is_personal, settings, created_at, updated_at)
                      VALUES ($1, $2, $3, $4, 'building', $5, 'viewer', -1, false, $6, NOW(), NOW())"#;

        let settings = json!({"billing_plan": plan});
        let description = format!("{} organization workspace", name);

        sqlx::query(sql)
            .bind(org_id)
            .bind(&name)
            .bind(&slug)
            .bind(&description)
            .bind(owner_id)
            .bind(&settings)
            .execute(&mut *conn)
            .await
            .map_err(|e| CliError::database(format!("Failed to insert org {}: {}", name, e)))?;

        let add_member_sql = r#"INSERT INTO organization_members (id, organization_id, user_id, role, joined_at)
                                 VALUES ($1::uuid, $2::uuid, $3::uuid, 'owner', NOW())"#;
        let member_id = uuid::Uuid::new_v4();
        sqlx::query(add_member_sql)
            .bind(member_id)
            .bind(org_id)
            .bind(owner_id)
            .execute(&mut *conn)
            .await
            .map_err(|e| CliError::database(format!("Failed to add org member: {}", e)))?;

        org_ids.push(org_id);
        reporter.update(i + 1);
    }
    reporter.finish();

    Ok(org_ids)
}

async fn seed_teams(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    user_ids: &[uuid::Uuid],
    org_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<Vec<uuid::Uuid>> {
    if count == 0 {
        return Ok(vec![]);
    }

    let mut reporter = ProgressReporter::new("teams", count);
    let mut team_ids = Vec::with_capacity(count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    for i in 0..count {
        let name = data.gen_team_name(i);
        let slug = slug::slugify(&name);
        let owner_id = user_ids[i % user_ids.len()];
        let team_id = uuid::Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO teams (id, name, slug, description, owner_id, avatar_url, settings, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, NULL, '{}', NOW(), NOW())"#,
        )
        .bind(team_id)
        .bind(&name)
        .bind(&slug)
        .bind(format!("{} team", name))
        .bind(owner_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| CliError::database(format!("Failed to insert team {}: {}", name, e)))?;

        let member_count = data.gen_range(2, std::cmp::min(8, user_ids.len()));
        for j in 0..member_count {
            let member_id = uuid::Uuid::new_v4();
            let member_user_id = user_ids[(i + j) % user_ids.len()];
            let role_names = ["admin", "member", "member", "viewer"];
            let role_name = role_names[j % role_names.len()];

            sqlx::query(
                r#"INSERT INTO team_members (id, team_id, user_id, role_name, joined_at)
                   VALUES ($1, $2, $3, $4, NOW())"#,
            )
            .bind(member_id)
            .bind(team_id)
            .bind(member_user_id)
            .bind(role_name)
            .execute(&mut *conn)
            .await
            .map_err(|e| CliError::database(format!("Failed to add team member: {}", e)))?;
        }

        if !org_ids.is_empty() {
            let org_id = org_ids[i % org_ids.len()];
            let update_sql = "UPDATE teams SET organization_id = $1 WHERE id = $2";
            sqlx::query(update_sql)
                .bind(org_id)
                .bind(team_id)
                .execute(&mut *conn)
                .await
                .ok();
        }

        team_ids.push(team_id);
        reporter.update(i + 1);
    }
    reporter.finish();

    Ok(team_ids)
}

async fn seed_spaces(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    user_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<Vec<uuid::Uuid>> {
    if count == 0 {
        return Ok(vec![]);
    }

    let mut reporter = ProgressReporter::new("spaces", count);
    let mut space_ids = Vec::with_capacity(count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    for i in 0..count {
        let name = data.gen_space_name(i);
        let slug = slug::slugify(&name);
        let owner_id = user_ids[i % user_ids.len()];
        let space_id = uuid::Uuid::new_v4();
        let visibilities = ["private", "internal", "public"];
        let visibility = visibilities[i % visibilities.len()];

        sqlx::query(
            r#"INSERT INTO spaces (id, name, slug, description, icon, color, owner_id, visibility, sort_order, is_default, settings, created_at, updated_at)
               VALUES ($1, $2, $3, $4, 'folder', '#3B82F6', $5, $6, $7, false, '{}', NOW(), NOW())"#,
        )
        .bind(space_id)
        .bind(&name)
        .bind(&slug)
        .bind(format!("{} workspace", name))
        .bind(owner_id)
        .bind(visibility)
        .bind(i as i32)
        .execute(&mut *conn)
        .await
        .map_err(|e| CliError::database(format!("Failed to insert space {}: {}", name, e)))?;

        space_ids.push(space_id);
        reporter.update(i + 1);
    }
    reporter.finish();

    Ok(space_ids)
}

async fn seed_documents(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    user_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<Vec<uuid::Uuid>> {
    let mut reporter = ProgressReporter::new("documents", count);
    let mut doc_ids = Vec::with_capacity(count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    let batch_size = 100;

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, count);

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"INSERT INTO documents (id, title, slug, author_id, description, tags, frontmatter, visibility, status, content_type, word_count, character_count, read_count, edit_count, content, created_at, updated_at, published_at)
               "#,
        );

        query_builder.push_values(batch_start..batch_end, |mut b, i| {
            let doc_id = uuid::Uuid::new_v4();
            doc_ids.push(doc_id);

            let author_id = user_ids[i % user_ids.len()];
            let (
                title,
                slug,
                status,
                visibility,
                word_count,
                character_count,
                content,
                content_type,
            ) = data.gen_document(i, i % user_ids.len());
            let tags = data.gen_tags();
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            let frontmatter_json = serde_json::to_string(
                &json!({"word_count": word_count, "generator": "tachyon-seed"}),
            )
            .unwrap_or_else(|_| "{}".to_string());
            let description = content.chars().take(150).collect::<String>();

            let published_at = if status == "published" {
                Some(Utc::now() - Duration::days(data.gen_range(1, 365) as i64))
            } else {
                None
            };

            b.push_bind(doc_id)
                .push_bind(title)
                .push_bind(slug)
                .push_bind(author_id)
                .push_bind(description)
                .push_bind(tags_json)
                .push_bind(frontmatter_json)
                .push_bind(visibility)
                .push_bind(status)
                .push_bind(content_type)
                .push_bind(word_count)
                .push_bind(character_count)
                .push_bind(data.gen_range(0, 1000) as i32)
                .push_bind(data.gen_range(1, 15) as i32)
                .push_bind(content)
                .push_bind(Utc::now() - Duration::days(data.gen_range(0, 90) as i64))
                .push_bind(Utc::now())
                .push_bind(published_at);
        });

        let sql = query_builder.build();
        sql.execute(&mut *conn)
            .await
            .map_err(|e| CliError::database(format!("Failed to insert documents batch: {}", e)))?;

        reporter.update(batch_end);
    }
    reporter.finish();

    Ok(doc_ids)
}

async fn seed_comments(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    doc_ids: &[uuid::Uuid],
    user_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<()> {
    if count == 0 || doc_ids.is_empty() {
        return Ok(());
    }

    let mut reporter = ProgressReporter::new("comments", count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    let batch_size = 200;

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, count);

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"INSERT INTO document_comments (id, document_id, author_id, author_name, content, status, mentions, created_at, updated_at)
               "#,
        );

        query_builder.push_values(batch_start..batch_end, |mut b, i| {
            let comment_id = uuid::Uuid::new_v4();
            let document_id = doc_ids[i % doc_ids.len()];
            let author_idx = i % user_ids.len();
            let author_id = user_ids[author_idx];
            let author_name = format!("user{}", author_idx);
            let content = data.gen_comment(i, author_idx);
            let statuses = ["open", "open", "open", "resolved"];
            let status = statuses[i % statuses.len()];
            let mentions = if data.gen_bool(0.3) {
                let mentioned_idx = (i + 7) % user_ids.len();
                serde_json::to_string(&vec![format!("user{}", mentioned_idx)])
                    .unwrap_or_else(|_| "[]".to_string())
            } else {
                "[]".to_string()
            };

            b.push_bind(comment_id)
                .push_bind(document_id)
                .push_bind(author_id)
                .push_bind(author_name)
                .push_bind(content)
                .push_bind(status)
                .push_bind(mentions)
                .push_bind(Utc::now() - Duration::hours(data.gen_range(1, 720) as i64))
                .push_bind(Utc::now());
        });

        let sql = query_builder.build();
        sql.execute(&mut *conn)
            .await
            .map_err(|e| CliError::database(format!("Failed to insert comments batch: {}", e)))?;

        reporter.update(batch_end);
    }
    reporter.finish();

    Ok(())
}

async fn seed_saved_searches(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    user_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<()> {
    if count == 0 {
        return Ok(());
    }

    let mut reporter = ProgressReporter::new("saved searches", count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    let batch_size = 200;

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, count);

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"INSERT INTO saved_searches (id, user_id, name, query, filters, created_at, updated_at)
               "#,
        );

        query_builder.push_values(batch_start..batch_end, |mut b, i| {
            let id = uuid::Uuid::new_v4();
            let user_id = user_ids[i % user_ids.len()];
            let (name, query_text) = data.gen_search_query(i);
            let filters = json!({
                "status": ["published", "draft"],
                "sort_by": "relevance"
            });
            let filters_str = serde_json::to_string(&filters).unwrap_or_else(|_| "{}".to_string());

            b.push_bind(id)
                .push_bind(user_id)
                .push_bind(name)
                .push_bind(query_text)
                .push_bind(filters_str)
                .push_bind(Utc::now())
                .push_bind(Utc::now());
        });

        let sql = query_builder.build();
        sql.execute(&mut *conn).await.map_err(|e| {
            CliError::database(format!("Failed to insert saved searches batch: {}", e))
        })?;

        reporter.update(batch_end);
    }
    reporter.finish();

    Ok(())
}

async fn seed_notifications(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    user_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<()> {
    if count == 0 {
        return Ok(());
    }

    let mut reporter = ProgressReporter::new("notifications", count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    let batch_size = 500;

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, count);

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"INSERT INTO notifications (id, user_id, type, title, body, link, read, metadata, created_at)
               "#,
        );

        query_builder.push_values(batch_start..batch_end, |mut b, i| {
            let id = uuid::Uuid::new_v4();
            let user_id = user_ids[i % user_ids.len()];
            let doc_idx = i % 50;
            let (notification_type, title, body, link) =
                data.gen_notification(i % user_ids.len(), doc_idx);
            let is_read = data.gen_bool(0.4);
            let metadata = json!({"source": "seed", "document_index": doc_idx});

            b.push_bind(id)
                .push_bind(user_id)
                .push_bind(notification_type)
                .push_bind(title)
                .push_bind(body)
                .push_bind(link)
                .push_bind(is_read)
                .push_bind(metadata)
                .push_bind(Utc::now() - Duration::hours(data.gen_range(1, 168) as i64));
        });

        let sql = query_builder.build();
        sql.execute(&mut *conn).await.map_err(|e| {
            CliError::database(format!("Failed to insert notifications batch: {}", e))
        })?;

        reporter.update(batch_end);
    }
    reporter.finish();

    Ok(())
}

async fn seed_sessions(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    user_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<()> {
    if count == 0 {
        return Ok(());
    }

    let mut reporter = ProgressReporter::new("sessions", count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    let batch_size = 200;

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, count);

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"INSERT INTO sessions (id, user_id, session_type, status, token_value, token_type, ip_address, user_agent, device_info, created_at, expires_at, last_activity)
               "#,
        );

        query_builder.push_values(batch_start..batch_end, |mut b, i| {
            let session_id = uuid::Uuid::new_v4();
            let user_id = user_ids[i % user_ids.len()];
            let expired = i % 3 == 0;
            let (session_type, status, user_agent, device_info, ip_address, token_value) =
                data.gen_session(expired);

            let created_at = Utc::now() - Duration::days(data.gen_range(1, 30) as i64);
            let expires_at = if expired {
                created_at + Duration::hours(1)
            } else {
                created_at + Duration::days(30)
            };
            let last_activity = if expired {
                created_at + Duration::minutes(30)
            } else {
                Utc::now() - Duration::minutes(data.gen_range(1, 1440) as i64)
            };

            b.push_bind(session_id)
                .push_bind(user_id)
                .push_bind(session_type)
                .push_bind(status)
                .push_bind(token_value.unwrap_or_default())
                .push_bind("bearer")
                .push_bind(ip_address)
                .push_bind(user_agent)
                .push_bind(device_info)
                .push_bind(created_at)
                .push_bind(expires_at)
                .push_bind(last_activity);
        });

        let sql = query_builder.build();
        sql.execute(&mut *conn)
            .await
            .map_err(|e| CliError::database(format!("Failed to insert sessions batch: {}", e)))?;

        reporter.update(batch_end);
    }
    reporter.finish();

    Ok(())
}

async fn seed_graph_edges(
    pool: &tachyon_database::DatabasePool,
    data: &mut FakeData,
    doc_ids: &[uuid::Uuid],
    count: usize,
) -> CliResult<()> {
    if count == 0 || doc_ids.len() < 2 {
        return Ok(());
    }

    let mut reporter = ProgressReporter::new("graph edges", count);
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    let batch_size = 200;

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, count);

        let mut query_builder = sqlx::QueryBuilder::new(
            r#"INSERT INTO graph_edges (id, source_id, target_id, edge_type, label, description, weight, confidence, properties, is_active, created_at, updated_at)
               "#,
        );

        query_builder.push_values(batch_start..batch_end, |mut b, i| {
            let edge_id = uuid::Uuid::new_v4();
            let source_idx = i % doc_ids.len();
            let target_idx = (i + data.gen_range(1, doc_ids.len())) % doc_ids.len();
            let source_id = doc_ids[source_idx];
            let target_id = doc_ids[target_idx];

            let edge_types = [
                "references",
                "depends_on",
                "similar_to",
                "part_of",
                "related_to",
            ];
            let labels = [
                "References",
                "Depends on",
                "Similar to",
                "Part of",
                "Related to",
            ];
            let type_idx = i % edge_types.len();
            let edge_type = edge_types[type_idx];
            let label = labels[type_idx];

            let weight = 0.5 + (i % 10) as f64 * 0.1;
            let confidence = Some(0.7 + (i % 5) as f64 * 0.05);
            let properties = json!({"seed_generated": true});

            b.push_bind(edge_id)
                .push_bind(source_id)
                .push_bind(target_id)
                .push_bind(edge_type)
                .push_bind(label)
                .push_bind(format!("Auto-generated {} edge", edge_type))
                .push_bind(weight)
                .push_bind(confidence)
                .push_bind(properties)
                .push_bind(true)
                .push_bind(Utc::now())
                .push_bind(Utc::now());
        });

        let sql = query_builder.build();
        sql.execute(&mut *conn).await.map_err(|e| {
            CliError::database(format!("Failed to insert graph edges batch: {}", e))
        })?;

        reporter.update(batch_end);
    }
    reporter.finish();

    Ok(())
}

fn generate_seed_sql(writer: &mut DryRunWriter, counts: &SeedCounts) {
    let mut data = FakeData::new();

    writer.push("BEGIN TRANSACTION");

    writer.push("DELETE FROM audit_log CASCADE");
    writer.push("DELETE FROM document_comments CASCADE");
    writer.push("DELETE FROM search_index CASCADE");
    writer.push("DELETE FROM graph_edges CASCADE");
    writer.push("DELETE FROM graph_nodes CASCADE");
    writer.push("DELETE FROM notifications CASCADE");
    writer.push("DELETE FROM sessions CASCADE");
    writer.push("DELETE FROM saved_searches CASCADE");
    writer.push("DELETE FROM document_versions CASCADE");
    writer.push("DELETE FROM attachments CASCADE");
    writer.push("DELETE FROM documents CASCADE");
    writer.push("DELETE FROM project_members CASCADE");
    writer.push("DELETE FROM components CASCADE");
    writer.push("DELETE FROM projects CASCADE");
    writer.push("DELETE FROM space_members CASCADE");
    writer.push("DELETE FROM spaces CASCADE");
    writer.push("DELETE FROM team_members CASCADE");
    writer.push("DELETE FROM teams CASCADE");
    writer.push("DELETE FROM organization_members CASCADE");
    writer.push("DELETE FROM organizations CASCADE");
    writer.push("DELETE FROM user_roles CASCADE");
    writer.push("DELETE FROM roles CASCADE");
    writer.push("DELETE FROM users CASCADE");

    writer.push("-- Seeding users");
    for i in 0..counts.users.min(3) {
        let (username, display_name, email, password_hash, totp_enabled) = data.gen_user(i);
        let role = if i == 0 { "admin" } else { "viewer" };
        writer.push(&format!(
            "INSERT INTO users (id, username, display_name, email, password_hash, role, user_type, is_active, totp_enabled) VALUES (gen_random_uuid(), '{}', '{}', '{}', '{}', '{}', 'regular', true, {})",
            username, display_name, email, password_hash, role, totp_enabled
        ));
    }
    if counts.users > 3 {
        writer.push(&format!("-- ... and {} more users", counts.users - 3));
    }

    writer.push("-- Seeding documents");
    for i in 0..counts.documents.min(2) {
        let (title, slug, status, visibility, _, _, _, content_type) = data.gen_document(i, 0);
        writer.push(&format!(
            "INSERT INTO documents (id, title, slug, author_id, visibility, status, content_type, word_count, character_count) VALUES (gen_random_uuid(), '{}', '{}', (SELECT id FROM users LIMIT 1), '{}', '{}', '{}', 150, 900)",
            title, slug, visibility, status, content_type
        ));
    }
    if counts.documents > 2 {
        writer.push(&format!(
            "-- ... and {} more documents",
            counts.documents - 2
        ));
    }

    writer.push("-- Seeding comments, saved_searches, notifications, sessions, graph_edges");
    writer.push(&format!(
        "-- {} comments, {} saved searches, {} notifications, {} sessions, {} graph edges",
        counts.comments,
        counts.saved_searches,
        counts.notifications,
        counts.sessions,
        counts.graph_edges
    ));

    writer.push("COMMIT");
}

// ============================================================================
// Stats
// ============================================================================

fn sanitize_identifier(name: &str) -> Result<&str, String> {
    if name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.is_empty() {
        Ok(name)
    } else {
        Err(format!("Invalid SQL identifier: {}", name))
    }
}

pub fn execute_stats(database_url: &str) -> CliResult<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::database(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async { execute_stats_async(database_url).await })
}

async fn execute_stats_async(database_url: &str) -> CliResult<()> {
    let pool = tachyon_database::DatabasePool::new(database_url)
        .await
        .map_err(|e| CliError::database(format!("Failed to connect: {}", e)))?;

    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| CliError::database(e.to_string()))?;

    let tables = [
        "users",
        "documents",
        "document_comments",
        "saved_searches",
        "notifications",
        "sessions",
        "organizations",
        "teams",
        "spaces",
        "projects",
        "repositories",
        "graph_nodes",
        "graph_edges",
        "search_index",
        "audit_log",
        "document_versions",
        "attachments",
    ];

    println!(
        "\n{:<25} {:>12} {:>18}",
        "Table", "Row Count", "Size (bytes)"
    );
    println!("{}", "-".repeat(57));

    for table in &tables {
        let safe_table = sanitize_identifier(table)
            .map_err(|e| CliError::database(e))?;
        let count_sql = format!("SELECT COUNT(*) as count FROM {}", safe_table);
        let size_sql = format!("SELECT pg_total_relation_size('{}') as size", safe_table);

        let (count, size): (i64, i64) =
            sqlx::query_as::<_, (i64, i64)>(&format!("SELECT ({}), ({})", count_sql, size_sql))
                .fetch_one(&mut *conn)
                .await
                .unwrap_or_default();

        let size_str = format_size(size as u64);
        println!("{:<25} {:>12} {:>18}", table, count, size_str);
    }

    let total_sql = "SELECT SUM(n_live_tup) as total_rows FROM pg_stat_user_tables";
    let total_size_sql = "SELECT SUM(pg_total_relation_size(schemaname || '.' || tablename)) as total_size FROM pg_tables WHERE schemaname = 'public'";

    let (total_rows, total_size): (Option<i64>, Option<i64>) = sqlx::query_as(&format!(
        "SELECT ({}) as total_rows, ({}) as total_size",
        total_sql, total_size_sql
    ))
    .fetch_one(&mut *conn)
    .await
    .unwrap_or_default();

    println!("{}", "-".repeat(57));
    println!(
        "{:<25} {:>12} {:>18}",
        "TOTAL",
        total_rows.unwrap_or(0),
        total_size.map_or("-".to_string(), |s| format_size(s as u64)),
    );
    println!();

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_scale_parse() {
        assert_eq!(SeedScale::parse("small"), Some(SeedScale::Small));
        assert_eq!(SeedScale::parse("medium"), Some(SeedScale::Medium));
        assert_eq!(SeedScale::parse("large"), Some(SeedScale::Large));
        assert_eq!(SeedScale::parse("production"), Some(SeedScale::Production));
        assert_eq!(SeedScale::parse("invalid"), None);
    }

    #[test]
    fn test_seed_scale_counts_small() {
        let counts = SeedScale::Small.counts();
        assert_eq!(counts.users, 10);
        assert_eq!(counts.spaces, 5);
        assert_eq!(counts.documents, 50);
        assert_eq!(counts.comments, 100);
    }

    #[test]
    fn test_seed_scale_counts_medium() {
        let counts = SeedScale::Medium.counts();
        assert_eq!(counts.users, 100);
        assert_eq!(counts.spaces, 20);
        assert_eq!(counts.documents, 1000);
        assert_eq!(counts.comments, 5000);
        assert_eq!(counts.saved_searches, 500);
    }

    #[test]
    fn test_seed_scale_counts_large() {
        let counts = SeedScale::Large.counts();
        assert_eq!(counts.users, 1000);
        assert_eq!(counts.spaces, 100);
        assert_eq!(counts.documents, 10000);
        assert_eq!(counts.comments, 50000);
    }

    #[test]
    fn test_seed_scale_counts_production() {
        let counts = SeedScale::Production.counts();
        assert_eq!(counts.users, 10000);
        assert_eq!(counts.spaces, 500);
        assert_eq!(counts.documents, 100000);
        assert_eq!(counts.comments, 500000);
    }

    #[test]
    fn test_fake_data_deterministic_users() {
        let mut data1 = FakeData::new();
        let mut data2 = FakeData::new();

        let user1 = data1.gen_user(0);
        let user2 = data2.gen_user(0);
        assert_eq!(user1.0, user2.0);
        assert_eq!(user1.1, user2.1);
        assert_eq!(user1.2, user2.2);
    }

    #[test]
    fn test_fake_data_unique_usernames() {
        let mut data = FakeData::new();
        let mut usernames: std::collections::HashSet<String> = std::collections::HashSet::new();
        for i in 0..100 {
            let user = data.gen_user(i);
            assert!(
                usernames.insert(user.0),
                "Duplicate username at index {}",
                i
            );
        }
    }

    #[test]
    fn test_fake_data_document_generation() {
        let mut data = FakeData::new();
        for i in 0..10 {
            let (title, slug, status, _, word_count, char_count, content, _) =
                data.gen_document(i, 0);
            assert!(!title.is_empty());
            assert!(!slug.is_empty());
            assert!(["draft", "published", "archived"].contains(&status.as_str()));
            assert!(word_count > 0);
            assert!(char_count > 0);
            assert!(!content.is_empty());
            assert!(content.starts_with('#'));
        }
    }

    #[test]
    fn test_fake_data_tags() {
        let mut data = FakeData::new();
        for _ in 0..50 {
            let tags = data.gen_tags();
            assert!(!tags.is_empty());
            assert!(tags.len() <= 5);
            let mut sorted = tags.clone();
            sorted.sort();
            assert_eq!(tags, sorted, "Tags should be sorted");
        }
    }

    #[test]
    fn test_fake_data_comments() {
        let mut data = FakeData::new();
        for i in 0..20 {
            let comment = data.gen_comment(i, 0);
            assert!(!comment.is_empty());
        }
    }

    #[test]
    fn test_fake_data_search_queries() {
        let mut data = FakeData::new();
        for i in 0..20 {
            let (name, query) = data.gen_search_query(i);
            assert!(!name.is_empty());
            assert!(!query.is_empty());
        }
    }

    #[test]
    fn test_fake_data_notifications() {
        let mut data = FakeData::new();
        for i in 0..20 {
            let (n_type, title, _body, _link) = data.gen_notification(i, 0);
            assert!(!n_type.is_empty());
            assert!(!title.is_empty());
        }
    }

    #[test]
    fn test_fake_data_sessions() {
        let mut data = FakeData::new();
        for _ in 0..10 {
            let (s_type, status, ua, device, ip, token) = data.gen_session(false);
            assert!(!s_type.is_empty());
            assert_eq!(status, "active");
            assert!(!ua.is_empty());
            assert!(!device.is_empty());
            assert!(ip.is_some());
            assert!(token.is_some());
        }

        let (_, expired_status, _, _, _, _) = data.gen_session(true);
        assert_eq!(expired_status, "expired");
    }

    #[test]
    fn test_dry_run_generation() {
        let counts = SeedScale::Small.counts();
        let mut writer = DryRunWriter::new();
        generate_seed_sql(&mut writer, &counts);
        assert!(!writer.statements.is_empty());
        assert!(writer.statements.iter().any(|s| s.contains("BEGIN")));
        assert!(writer.statements.iter().any(|s| s.contains("COMMIT")));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500.0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(0.0), "<1s");
        assert_eq!(format_eta(0.5), "<1s");
        assert_eq!(format_eta(5.0), "5s");
        assert_eq!(format_eta(90.0), "2m");
        assert_eq!(format_eta(7200.0), "2h");
    }

    #[test]
    fn test_seed_config_creation() {
        let config = SeedConfig {
            database_url: "postgres://localhost/test".to_string(),
            scale: SeedScale::Medium,
            clear: false,
            dry_run: true,
        };
        assert_eq!(config.scale, SeedScale::Medium);
        assert!(config.dry_run);
        assert!(!config.clear);
    }

    #[test]
    fn test_progress_reporter() {
        let reporter = ProgressReporter::new("test", 10);
        reporter.finish();
    }

    #[test]
    fn test_fake_data_org_names() {
        let mut data = FakeData::new();
        for i in 0..50 {
            let name = data.gen_org_name(i);
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_fake_data_space_names() {
        let mut data = FakeData::new();
        for i in 0..50 {
            let name = data.gen_space_name(i);
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_fake_data_team_names() {
        let mut data = FakeData::new();
        for i in 0..30 {
            let name = data.gen_team_name(i);
            assert!(!name.is_empty());
        }
    }
}
