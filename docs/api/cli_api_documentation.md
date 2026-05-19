# TACHYON: CLI API DOCUMENTATION

**Document ID:** TACHYON-API-006-V1.0
**Date:** February 2026
**Status:** Approved for Execution
**Classification:** API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 8294:2021

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [CLI API Framework](#2-cli-api-framework)
3. [Command Structure](#3-command-structure)
4. [Global Options](#4-global-options)
5. [Document Commands](#5-document-commands)
6. [Workspace Commands](#6-workspace-commands)
7. [Git Commands](#7-git-commands)
8. [Server Commands](#8-server-commands)
9. [Plugin Commands](#9-plugin-commands)
10. [Configuration Commands](#10-configuration-commands)
11. [Output Formats](#11-output-formats)
12. [Error Handling](#12-error-handling)
13. [References](#13-references)

---

## 1. INTRODUCTION

### 1.1. Purpose and Scope

This document provides a comprehensive specification of the Tachyon Command Line Interface (CLI) API. The CLI serves as the primary interface for developers, DevOps engineers, and power users to interact with the Tachyon toolchain programmatically and efficiently.

The CLI API encompasses:
- Command invocation and execution framework
- Global configuration and option handling
- Document management operations
- Workspace management operations
- Git integration operations
- Server control and monitoring
- Plugin system interface
- Configuration management
- Output formatting and streaming
- Error handling and reporting

### 1.2. Design Philosophy

The Tachyon CLI API is designed with the following principles:

#### 1.2.1. Composability
Commands are designed to be composable, enabling users to chain operations together through piping and command substitution. Each command produces structured output that can be consumed by subsequent commands.

#### 1.2.2. Predictability
Command behavior follows consistent patterns across all operations. Flag naming, argument parsing, and output formatting adhere to established conventions, reducing cognitive load for users.

#### 1.2.3. Extensibility
The CLI architecture supports plugins and extensions through a well-defined plugin interface. Third-party developers can extend CLI functionality without modifying core code.

#### 1.2.4. Safety First
Destructive operations require explicit confirmation. The CLI provides dry-run modes for previewing operations before execution. All operations are validated before execution.

### 1.3. Technology Stack

The Tachyon CLI is implemented using:
- **Language:** Rust (per [ADR-001](.adrs/adr-001-three-tier-jit-compilation.md))
- **CLI Framework:** clap (Command Line Argument Parser)
- **Async Runtime:** Tokio (per [ADR-002](.adrs/adr-002-bm25-search-parameters.md))
- **Configuration:** serde for serialization/deserialization
- **Output Formatting:** termcolor and table formatting libraries

### 1.4. Related Documentation

This document references the following specifications:
- [TACHYON-STD-V1.0](.adrs/ - Coding and Documentation Standards
- [TACHYON-REQ-V1.0](.adrs/ - Requirements Specification
- [ADR-010](.adrs/adr-010-synchronization-primitives.md) - Security Architecture
- [TACHYON-TST-V1.0](.adrs/ - Test Plan

---

## 2. CLI API FRAMEWORK

### 2.1. Architecture Overview

The Tachyon CLI follows a hierarchical command structure organized into logical groups:

```
tachyon [GLOBAL_OPTIONS] <COMMAND> [SUBCOMMAND] [COMMAND_OPTIONS] [ARGUMENTS]
```

The framework consists of the following layers:

#### 2.1.1. Application Layer
The top-level application layer handles:
- Process initialization and signal handling
- Global option parsing and validation
- Configuration file loading and merging
- Logging infrastructure setup
- Plugin discovery and initialization
- Command routing and dispatch

#### 2.1.2. Command Layer
Each command implements the following traits:
- `Command` trait for command definition
- `Args` trait for argument parsing
- `Executor` trait for execution logic
- `Validator` trait for input validation

#### 2.1.3. Output Layer
The output layer provides:
- Structured data serialization (JSON, YAML, TOML)
- Human-readable formatting (tables, lists, trees)
- Progress indicators and spinners
- Color and formatting control
- Streaming output support for long-running operations

### 2.2. Command Definition Pattern

All commands follow a consistent definition pattern:

```rust
#[derive(Parser, Debug)]
#[command(
    name = "command-name",
    about = "Brief description of the command",
    long_about = "Detailed description explaining command purpose and usage"
)]
struct CommandName {
    /// Positional argument description
    #[arg(value_name = "arg-name", required = true)]
    arg_name: String,

    /// Optional flag description
    #[arg(short = 'f', long = "flag-name")]
    flag_name: bool,
}
```

### 2.3. Execution Model

The CLI supports multiple execution models:

#### 2.3.1. Synchronous Execution
For simple, fast operations that complete within milliseconds:
- Configuration queries
- Status checks
- Simple CRUD operations

#### 2.3.2. Asynchronous Execution
For I/O-bound and long-running operations:
- Network operations
- File system operations
- Git operations

#### 2.3.3. Streaming Execution
For operations that produce continuous output:
- Log tailing
- Real-time monitoring
- Build processes

### 2.4. Configuration Hierarchy

Configuration is resolved in the following precedence order (highest to lowest):

1. **Command-line arguments** - Direct user input
2. **Environment variables** - `TACHYON_*` prefixed variables
3. **Workspace configuration** - `.tachyon/config.toml` in current workspace
4. **User configuration** - `~/.config/tachyon/config.toml`
5. **System configuration** - `/etc/tachyon/config.toml`
6. **Default values** - Built-in defaults

### 2.5. Plugin System

The CLI plugin system enables runtime extension of functionality:

#### 2.5.1. Plugin Discovery
Plugins are discovered from:
- `~/.tachyon/plugins/` - User-local plugins
- `<workspace>/.tachyon/plugins/` - Workspace-local plugins
- System plugin directories - Configurable via installation

#### 2.5.2. Plugin Interface
Plugins must implement the `Plugin` trait:

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn commands(&self) -> Vec<Box<dyn Command>>;
    fn init(&mut self, config: &Config) -> Result<()>;
}
```

#### 2.5.3. Plugin Lifecycle
1. **Discovery** - Scan plugin directories during CLI initialization
2. **Validation** - Verify plugin signature and dependencies
3. **Registration** - Register plugin commands with the CLI
4. **Execution** - Execute plugin commands when invoked
5. **Cleanup** - Release plugin resources on CLI shutdown

---

## 3. COMMAND STRUCTURE

### 3.1. Command Syntax

The Tachyon CLI follows a hierarchical command structure with the following syntax:

```
tachyon [GLOBAL_OPTIONS] <COMMAND> [SUBCOMMAND] [COMMAND_OPTIONS] [ARGUMENTS]
```

#### 3.1.1. Command Naming Conventions

Commands follow these naming conventions:
- **Command names:** Use lowercase, hyphen-separated words (e.g., `create-workspace`, `list-documents`)
- **Short flags:** Single lowercase letter (e.g., `-v`, `-q`, `-f`)
- **Long flags:** Double hyphen prefix (e.g., `--verbose`, `--quiet`, `--force`)
- **Environment variables:** Uppercase with `TACHYON_` prefix (e.g., `TACHYON_CONFIG_DIR`)

### 3.2. Argument Types

The CLI supports the following argument types:

#### 3.2.1. Positional Arguments
Positional arguments are required and must be provided in order:

```bash
tachyon create-workspace <workspace-name> <workspace-path>
```

**Validation Rules:**
- Type checking based on argument definition
- Range validation for numeric arguments
- Path validation for file system arguments
- Enum validation for discrete choices

#### 3.2.2. Optional Flags
Optional flags modify command behavior:

```bash
tachyon list-documents --format json --recursive --limit 100
```

**Flag Categories:**
- **Output control:** `--format`, `--output`, `--quiet`, `--verbose`
- **Execution control:** `--dry-run`, `--force`, `--interactive`
- **Filtering:** `--filter`, `--exclude`, `--include`
- **Pagination:** `--limit`, `--offset`, `--page`

#### 3.2.3. Value Arguments
Value arguments accept key-value pairs:

```bash
tachyon config set <key> <value>
tachyon git checkout --branch <branch-name>
```

### 3.3. Subcommand Hierarchy

Commands are organized into logical hierarchies:

```
tachyon
├── document
│   ├── create
│   ├── list
│   ├── get
│   ├── update
│   └── delete
├── workspace
│   ├── init
│   ├── list
│   ├── switch
│   └── status
├── git
│   ├── status
│   ├── branch
│   ├── commit
│   ├── push
│   └── pull
├── server
│   ├── start
│   ├── stop
│   ├── status
│   └── logs
├── plugin
│   ├── list
│   ├── install
│   ├── uninstall
│   └── info
└── config
    ├── get
    ├── set
    ├── list
    └── reset
```

### 3.4. Command Execution Flow

Command execution follows this flow:

1. **Parse Arguments** - clap parses command-line arguments
2. **Load Configuration** - Configuration is loaded from all sources and merged
3. **Validate Input** - Validators check argument constraints
4. **Initialize Context** - Execution context is established
5. **Execute Command** - Command logic runs with async/sync execution model
6. **Format Output** - Results are formatted according to output options
7. **Handle Errors** - Errors are caught and formatted for display
8. **Exit** - Process exits with appropriate status code

### 3.5. Help System

The CLI provides comprehensive help at multiple levels:

#### 3.5.1. Global Help
```bash
tachyon --help
tachyon -h
```
Displays all available commands and global options.

#### 3.5.2. Command Help
```bash
tachyon <command> --help
tachyon <command> -h
```
Displays detailed help for specific command including:
- Command description
- All available subcommands
- Required and optional arguments
- Usage examples
- Related commands

#### 3.5.3. Auto-Completion
The CLI supports shell auto-completion for:
- Command names
- Subcommands
- Argument values
- File paths

Enable with:
```bash
# For bash
eval "$(tachyon completion bash)"

# For zsh
eval "$(tachyon completion zsh)"

# For fish
tachyon completion fish | source
```

---

## 4. GLOBAL OPTIONS

### 4.1. Overview

Global options apply to all commands and control CLI-wide behavior. These options are parsed before command execution and influence the execution environment.

### 4.2. Configuration Options

#### 4.2.1. Configuration Path
```bash
--config <path>
-c <path>
```
Specifies custom configuration file location. Overrides default configuration search paths.

**Parameters:**
- `<path>`: Path to configuration file (TOML, YAML, or JSON format)
- Type: File path
- Default: Automatic search in standard locations

**Related Configuration:** `TACHYON_CONFIG_FILE`

#### 4.2.2. Workspace Path
```bash
--workspace <path>
-w <path>
```
Specifies the workspace root directory. All relative paths are resolved against this directory.

**Parameters:**
- `<path>`: Path to workspace directory
- Type: Directory path
- Default: Current working directory

**Related Configuration:** `TACHYON_WORKSPACE`

#### 4.2.3. Verbosity Level
```bash
--verbose
-v
```
Controls output verbosity. Multiple levels increase detail.

**Levels:**
- No flag: Standard output (errors and warnings)
- `-v`: Info level output
- `-vv`: Debug level output
- `-vvv`: Trace level output

**Related Configuration:** `TACHYON_VERBOSITY`

#### 4.2.4. Quiet Mode
```bash
--quiet
-q
```
Suppresses all non-error output. Only critical messages are displayed.

**Behavior:**
- Suppresses info messages
- Suppresses warning messages
- Only errors are displayed
- Overrides verbosity when both specified

**Related Configuration:** `TACHYON_QUIET`

#### 4.2.5. Output Format
```bash
--format <format>
-f <format>
```
Specifies output format for structured data.

**Supported Formats:**
- `json`: JSON format
- `yaml`: YAML format
- `toml`: TOML format
- `table`: Human-readable table
- `plain`: Plain text

**Related Configuration:** `TACHYON_OUTPUT_FORMAT`

#### 4.2.6. Color Output
```bash
--color <when>
```
Controls colored output.

**Values:**
- `always`: Always use colors
- `auto`: Use colors if terminal supports (default)
- `never`: Never use colors

**Related Configuration:** `TACHYON_COLOR`

#### 4.2.7. Dry Run
```bash
--dry-run
-n
```
Preview operations without making changes.

**Behavior:**
- Validates all inputs
- Shows what would be executed
- No side effects (file changes, network calls)
- Returns exit code 0 if validation succeeds

**Related Configuration:** `TACHYON_DRY_RUN`

#### 4.2.8. Force Operation
```bash
--force
-f
```
Bypasses safety checks and confirmations.

**Behavior:**
- Skips confirmation prompts
- Overwrites existing files
- Bypasses validation warnings
- Use with caution

**Related Configuration:** `TACHYON_FORCE`

#### 4.2.9. Interactive Mode
```bash
--interactive
-i
```
Enables interactive prompts for operations.

**Behavior:**
- Prompts for missing required arguments
- Confirms destructive operations
- Allows selection from multiple options
- Disables non-interactive mode

**Related Configuration:** `TACHYON_INTERACTIVE`

#### 4.2.10. No-Pager
```bash
--no-pager
-P
```
Disables output paging.

**Behavior:**
- Outputs all content immediately
- Useful for scripting and automation
- Overrides `PAGER` environment variable

**Related Configuration:** `TACHYON_PAGER`

#### 4.2.11. Help and Version
```bash
--help
-h
--version
-V
```
Display help and version information.

**Behavior:**
- `--help`/`-h`: Display help message
- `--version`/`-V`: Display version information
- Exits after displaying information

### 4.3. Environment Variables

The CLI respects the following environment variables:

#### 4.3.1. Configuration Variables
```bash
TACHYON_CONFIG_DIR=<path>
TACHYON_CONFIG_FILE=<path>
TACHYON_WORKSPACE=<path>
```
Override configuration discovery and file locations.

#### 4.3.2. Behavior Variables
```bash
TACHYON_VERBOSITY=<level>
TACHYON_QUIET=<true|false>
TACHYON_COLOR=<always|auto|never>
TACHYON_DRY_RUN=<true|false>
TACHYON_FORCE=<true|false>
TACHYON_INTERACTIVE=<true|false>
TACHYON_OUTPUT_FORMAT=<format>
TACHYON_PAGER=<command>
```
Control CLI behavior without command-line flags.

#### 4.3.3. Authentication Variables
```bash
TACHYON_TOKEN=<token>
TACHYON_API_KEY=<key>
TACHYON_API_ENDPOINT=<url>
```
Configure authentication for server operations.

**Security Note:** These variables should be set in environment or `.env` files, not in shell history.

### 4.4. Option Precedence

Options are resolved in the following precedence order (highest to lowest):

1. **Command-line flags** - Direct user input
2. **Environment variables** - `TACHYON_*` prefixed variables
3. **Configuration file** - Values from config file
4. **Default values** - Built-in defaults

Later options override earlier options in the same category.

### 4.5. Option Validation

All global options undergo validation:

#### 4.5.1. Path Validation
- File paths must exist or be creatable
- Directory paths must be accessible
- Relative paths are resolved against workspace

#### 4.5.2. Value Validation
- Enum values must match allowed options
- Numeric values must be in valid range
- Boolean flags accept true/false, yes/no, 1/0

#### 4.5.3. Conflict Detection
- Conflicting options are detected and reported
- Last specified option takes precedence
- Warnings are issued for ambiguous combinations

---

## 5. DOCUMENT COMMANDS

Document commands manage content within Tachyon workspaces, including creation, retrieval, modification, and deletion of documents.

### 5.1. Document Create

#### 5.1.1. Syntax
```bash
tachyon document create <title> [OPTIONS]
```

#### 5.1.2. Parameters
- `title` (required): Document title
- `--type <type>` (optional): Document type (default: markdown)
- `--template <template>` (optional): Template to use
- `--path <path>` (optional): Parent directory (default: current directory)
- `--workspace <workspace>` (optional): Target workspace (default: current)

#### 5.1.3. Options
- `--dry-run`: Preview creation without changes
- `--force`: Overwrite existing document
- `--editor <editor>`: Open in specified editor
- `--no-git`: Skip git commit after creation

#### 5.1.4. Examples
```bash
# Create a new markdown document
tachyon document create "Getting Started Guide" --type markdown

# Create from template
tachyon document create "API Spec" --template api-template.md

# Create in specific directory
tachyon document create "README" --path docs/

# Preview without committing
tachyon document create "Draft" --dry-run
```

#### 5.1.5. Output
Returns document metadata in JSON format:
```json
{
  "id": "doc-123",
  "title": "Document Title",
  "type": "markdown",
  "path": "/path/to/document.md",
  "created_at": "2026-02-07T17:28:00Z",
  "workspace": "workspace-name"
}
```

### 5.2. Document List

#### 5.2.1. Syntax
```bash
tachyon document list [OPTIONS]
```

#### 5.2.2. Parameters
- `--workspace <workspace>` (optional): Filter by workspace (default: current)
- `--type <type>` (optional): Filter by document type
- `--format <format>` (optional): Output format (default: table)
- `--recursive` (optional): Include subdirectories
- `--limit <n>` (optional): Maximum number of results

#### 5.2.3. Options
- `--json`: Output in JSON format
- `--yaml`: Output in YAML format
- `--verbose`: Include detailed metadata
- `--sort <field>`: Sort by field (name, created, modified)

#### 5.2.4. Examples
```bash
# List all documents
tachyon document list

# List in JSON format
tachyon document list --format json

# List markdown documents only
tachyon document list --type markdown

# List with limit
tachyon document list --limit 10

# List recursively
tachyon document list --recursive
```

#### 5.2.5. Output
Table format (default):
```
+------------+----------------+----------------+----------------+----------------+
| ID         | Title           | Type    | Created              | Modified             |
+------------+----------------+----------------+----------------+----------------+
| doc-001    | Introduction     | markdown | 2026-02-07T10:00 | 2026-02-07T15:00 |
| doc-002    | API Reference    | markdown | 2026-02-07T11:00 | 2026-02-07T16:00 |
+------------+----------------+----------------+----------------+----------------+
```

JSON format:
```json
[
  {
    "id": "doc-001",
    "title": "Introduction",
    "type": "markdown",
    "path": "docs/introduction.md",
    "created_at": "2026-02-07T10:00:00Z",
    "modified_at": "2026-02-07T15:00:00Z",
    "workspace": "default"
  }
]
```

### 5.3. Document Get

#### 5.3.1. Syntax
```bash
tachyon document get <document-id> [OPTIONS]
```

#### 5.3.2. Parameters
- `document-id` (required): Document identifier
- `--format <format>` (optional): Output format (default: table)
- `--output <path>` (optional): Write to file instead of stdout

#### 5.3.3. Options
- `--raw`: Output raw content without formatting
- `--metadata`: Include metadata only
- `--content`: Include content only

#### 5.3.4. Examples
```bash
# Get document by ID
tachyon document get doc-001

# Get and save to file
tachyon document get doc-001 --output document.md

# Get metadata only
tachyon document get doc-001 --metadata

# Get raw content
tachyon document get doc-001 --raw
```

#### 5.3.5. Output
Returns document details in specified format.

### 5.4. Document Update

#### 5.4.1. Syntax
```bash
tachyon document update <document-id> [OPTIONS]
```

#### 5.4.2. Parameters
- `document-id` (required): Document identifier
- `--title <title>` (optional): New title
- `--content <path>` (optional): New content file
- `--message <msg>` (optional): Commit message

#### 5.4.3. Options
- `--dry-run`: Preview changes without committing
- `--force`: Bypass validation
- `--no-git`: Skip git commit after update
- `--append`: Append to existing content

#### 5.4.4. Examples
```bash
# Update document title
tachyon document update doc-001 --title "Updated Introduction"

# Update from file
tachyon document update doc-001 --content new-content.md

# Preview changes
tachyon document update doc-001 --dry-run

# Update with custom message
tachyon document update doc-001 --message "Updated for v1.0"
```

#### 5.4.5. Output
Returns update confirmation in JSON format:
```json
{
  "id": "doc-001",
  "success": true,
  "updated_at": "2026-02-07T17:30:00Z",
  "changes": {
    "title": "Updated Introduction",
    "content_modified": true
  }
}
```

### 5.5. Document Delete

#### 5.5.1. Syntax
```bash
tachyon document delete <document-id> [OPTIONS]
```

#### 5.5.2. Parameters
- `document-id` (required): Document identifier
- `--force`: Bypass confirmation
- `--no-git`: Skip git commit after deletion

#### 5.5.3. Options
- `--dry-run`: Preview deletion without changes
- `--keep-git-history`: Keep git history

#### 5.5.4. Examples
```bash
# Delete document
tachyon document delete doc-001

# Preview deletion
tachyon document delete doc-001 --dry-run

# Force delete without confirmation
tachyon document delete doc-001 --force

# Delete but keep git history
tachyon document delete doc-001 --no-git
```

#### 5.5.5. Output
Returns deletion confirmation in JSON format:
```json
{
  "id": "doc-001",
  "deleted": true,
  "deleted_at": "2026-02-07T17:30:00Z",
  "git_commit": false
}
```

### 5.6. Document Search

#### 5.6.1. Syntax
```bash
tachyon document search <query> [OPTIONS]
```

#### 5.6.2. Parameters
- `query` (required): Search query string
- `--type <type>` (optional): Filter by document type
- `--workspace <workspace>` (optional): Search specific workspace
- `--limit <n>` (optional): Maximum results

#### 5.6.3. Options
- `--format <format>` (optional): Output format (default: table)
- `--json`: Output in JSON format
- `--case-sensitive`: Enable case-sensitive search

#### 5.6.4. Examples
```bash
# Search for documents
tachyon document search "API"

# Search in specific workspace
tachyon document search "guide" --workspace docs

# Case-sensitive search
tachyon document search "API" --case-sensitive

# Limit results
tachyon document search "getting started" --limit 5
```

#### 5.6.5. Output
Returns matching documents in specified format.

---

## 6. WORKSPACE COMMANDS

Workspace commands manage Tachyon workspaces, including initialization, switching, and status operations.

### 6.1. Workspace Init

#### 6.1.1. Syntax
```bash
tachyon workspace init <workspace-name> [OPTIONS]
```

#### 6.1.2. Parameters
- `workspace-name` (required): Name for new workspace
- `--path <path>` (optional): Workspace location (default: current directory)
- `--template <template>` (optional): Workspace template to use
- `--description <desc>` (optional): Workspace description

#### 6.1.3. Options
- `--dry-run`: Preview creation without changes
- `--force`: Overwrite existing workspace
- `--git`: Initialize git repository

#### 6.1.4. Examples
```bash
# Initialize new workspace
tachyon workspace init my-project

# Initialize with custom path
tachyon workspace init my-project --path ~/projects/

# Initialize with template
tachyon workspace init my-project --template minimal

# Preview initialization
tachyon workspace init my-project --dry-run

# Initialize with git
tachyon workspace init my-project --git --description "My Tachyon Workspace"
```

#### 6.1.5. Output
Returns workspace initialization result:
```json
{
  "workspace_name": "my-project",
  "workspace_path": "/home/user/projects/my-project",
  "git_initialized": true,
  "created_at": "2026-02-07T17:30:00Z"
}
```

### 6.2. Workspace List

#### 6.2.1. Syntax
```bash
tachyon workspace list [OPTIONS]
```

#### 6.2.2. Parameters
- `--path <path>` (optional): List workspaces in specific directory
- `--format <format>` (optional): Output format (default: table)

#### 6.2.3. Options
- `--json`: Output in JSON format
- `--verbose`: Include detailed information

#### 6.2.4. Examples
```bash
# List all workspaces
tachyon workspace list

# List in JSON format
tachyon workspace list --json

# List workspaces in specific directory
tachyon workspace list --path ~/projects/
```

#### 6.2.5. Output
Table format (default):
```
+----------------+----------------+----------------+----------------+
| Workspace Name | Path           | Git     | Created              |
+----------------+----------------+----------------+----------------+
| my-project    | ~/projects/my-project | true     | 2026-02-07T10:00 |
| docs          | ~/docs             | false    | 2026-02-07T11:00 |
+----------------+----------------+----------------+----------------+
```

JSON format:
```json
[
  {
    "name": "my-project",
    "path": "/home/user/projects/my-project",
    "git_initialized": true,
    "created_at": "2026-02-07T10:00:00Z"
  },
  {
    "name": "docs",
    "path": "/home/user/docs",
    "git_initialized": false,
    "created_at": "2026-02-07T11:00:00Z"
  }
]
```

### 6.3. Workspace Switch

#### 6.3.1. Syntax
```bash
tachyon workspace switch <workspace-name> [OPTIONS]
```

#### 6.3.2. Parameters
- `workspace-name` (required): Name of workspace to switch to

#### 6.3.3. Options
- `--dry-run`: Preview switch without changes
- `--persist`: Save as default workspace

#### 6.3.4. Examples
```bash
# Switch to different workspace
tachyon workspace switch docs

# Preview switch
tachyon workspace switch docs --dry-run

# Switch and save as default
tachyon workspace switch docs --persist
```

#### 6.3.5. Output
Returns switch confirmation:
```json
{
  "previous_workspace": "my-project",
  "new_workspace": "docs",
  "switched_at": "2026-02-07T17:30:00Z",
  "persisted": true
}
```

### 6.4. Workspace Status

#### 6.4.1. Syntax
```bash
tachyon workspace status [OPTIONS]
```

#### 6.4.2. Parameters
- `--workspace <workspace>` (optional): Status of specific workspace (default: current)

#### 6.4.3. Options
- `--json`: Output in JSON format
- `--verbose`: Include detailed status

#### 6.4.4. Examples
```bash
# Get current workspace status
tachyon workspace status

# Get status of specific workspace
tachyon workspace status --workspace my-project

# Status in JSON format
tachyon workspace status --json
```

#### 6.4.5. Output
Returns workspace status information:
```json
{
  "current_workspace": "my-project",
  "workspace_path": "/home/user/projects/my-project",
  "git_status": "clean",
  "branch": "main",
  "last_commit": "abc1234",
  "modified_at": "2026-02-07T17:25:00Z"
}
```

### 6.5. Workspace Delete

#### 6.5.1. Syntax
```bash
tachyon workspace delete <workspace-name> [OPTIONS]
```

#### 6.5.2. Parameters
- `workspace-name` (required): Name of workspace to delete
- `--force`: Bypass confirmation

#### 6.5.3. Options
- `--dry-run`: Preview deletion without changes
- `--remove-files`: Also delete workspace files

#### 6.5.4. Examples
```bash
# Delete workspace
tachyon workspace delete my-project

# Preview deletion
tachyon workspace delete my-project --dry-run

# Force delete
tachyon workspace delete my-project --force

# Delete workspace and files
tachyon workspace delete my-project --remove-files
```

#### 6.5.5. Output
Returns deletion confirmation:
```json
{
  "workspace_name": "my-project",
  "deleted": true,
  "files_removed": false,
  "deleted_at": "2026-02-07T17:30:00Z"
}
```

---

## 7. GIT COMMANDS

Git commands provide integration with Git repositories for version control of content and workspace configuration.

### 7.1. Git Status

#### 7.1.1. Syntax
```bash
tachyon git status [OPTIONS]
```

#### 7.1.2. Parameters
- `--workspace <workspace>` (optional): Status for specific workspace (default: current)
- `--format <format>` (optional): Output format (default: table)
- `--json`: Output in JSON format
- `--verbose`: Include detailed git information

#### 7.1.3. Options
- `--short`: Show abbreviated commit hashes
- `--branch`: Show tracking information
- `--porcelain`: Machine-readable output

#### 7.1.4. Examples
```bash
# Get current workspace git status
tachyon git status

# Get status for specific workspace
tachyon git status --workspace my-project

# Status in JSON format
tachyon git status --json

# Status with short hashes
tachyon git status --short

# Status with branch info
tachyon git status --branch
```

#### 7.1.5. Output
Returns git repository status information:
```json
{
  "branch": "main",
  "head": "abc1234",
  "is_clean": true,
  "untracked_files": [],
  "modified_files": ["docs/introduction.md"],
  "staged_files": ["docs/api_spec.md"],
  "ahead": 0,
  "behind": 0
}
```

### 7.2. Git Branch

#### 7.2.1. Syntax
```bash
tachyon git branch [OPTIONS]
```

#### 7.2.2. Parameters
- `<branch-name>` (required): Name of new branch
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--base <branch>` (optional): Starting branch
- `--track`: Set upstream tracking branch

#### 7.2.3. Options
- `--force`: Create branch even if exists
- `--dry-run`: Preview branch creation
- `--no-track`: Don't set tracking branch
- `--set-upstream`: Set upstream branch explicitly

#### 7.2.4. Examples
```bash
# Create new branch
tachyon git branch feature/new-feature

# Create from specific base
tachyon git branch feature/new-feature --base develop

# Create with tracking
tachyon git branch feature/new-feature --track origin/feature/new-feature

# Preview branch creation
tachyon git branch feature/new-feature --dry-run

# Force branch creation
tachyon git branch main --force
```

#### 7.2.5. Output
Returns branch creation confirmation:
```json
{
  "branch": "feature/new-feature",
  "created": true,
  "base": "develop",
  "tracking": "origin/feature/new-feature"
}
```

### 7.3. Git Checkout

#### 7.3.1. Syntax
```bash
tachyon git checkout <branch-name> [OPTIONS]
```

#### 7.3.2. Parameters
- `<branch-name>` (required): Branch to checkout
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--force`: Discard local changes
- `--detach`: Detach HEAD
- `--orphan`: Create orphan branch

#### 7.3.3. Options
- `--b <branch>`: Create new branch and checkout
- `--track`: Set tracking branch
- `--guess`: Fallback to merge strategy
- `--conflict <style>`: Conflict resolution (merge, diff3, ours, theirs)

#### 7.3.4. Examples
```bash
# Checkout existing branch
tachyon git checkout develop

# Checkout and create new branch
tachyon git checkout -b feature/new-feature

# Force checkout (discard changes)
tachyon git checkout main --force

# Detach HEAD
tachyon git checkout --detach HEAD
```

#### 7.3.5. Output
Returns checkout confirmation:
```json
{
  "previous_branch": "main",
  "current_branch": "feature/new-feature",
  "checkout_type": "existing",
  "forced": false
}
```

### 7.4. Git Commit

#### 7.4.1. Syntax
```bash
tachyon git commit [OPTIONS]
```

#### 7.4.2. Parameters
- `-m <message>` (required): Commit message
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--all`: Commit all staged changes
- `--amend`: Amend previous commit
- `--allow-empty`: Allow empty commits
- `--no-verify`: Skip pre-commit hooks

#### 7.4.3. Options
- `--dry-run`: Preview commit without changes
- `--no-edit`: Don't open editor
- `--signoff`: Add Signed-off-by line

#### 7.4.4. Examples
```bash
# Commit with message
tachyon git commit -m "Add new API endpoint"

# Commit with multi-line message
tachyon git commit -m "Add new API endpoint

This implements the new REST endpoint for user management
with improved error handling and validation."

# Commit all staged changes
tachyon git commit --all

# Amend previous commit
tachyon git commit --amend -m "Fix typo in previous commit"

# Allow empty commit
tachyon git commit --allow-empty -m "Initial commit"

# Skip pre-commit hooks
tachyon git commit --no-verify -m "Bypass validation for CI"

# Don't open editor
tachyon git commit -m "Automated commit" --no-edit

# Add signed-off
tachyon git commit -m "Release v1.0.0" --signoff
```

#### 7.4.5. Output
Returns commit information:
```json
{
  "commit_hash": "abc1234",
  "message": "Add new API endpoint",
  "author": "John Doe <john@example.com>",
  "timestamp": "2026-02-07T17:30:00Z",
  "files_changed": ["src/api/user.rs", "tests/user_test.rs"],
  "lines_added": 15,
  "lines_deleted": 2,
  "amended": false
}
```

### 7.5. Git Push

#### 7.5.1. Syntax
```bash
tachyon git push [OPTIONS]
```

#### 7.5.2. Parameters
- `<remote>` (optional): Remote to push to (default: origin)
- `<branch>` (optional): Branch to push (default: current)
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--force`: Force push even if not fast-forward
- `--dry-run`: Preview push without changes
- `--set-upstream`: Set upstream branch

#### 7.5.3. Options
- `--all`: Push all branches
- `--tags`: Push all tags
- `--atomic`: Use atomic push
- `--prune`: Remove remote-tracking references

#### 7.5.4. Examples
```bash
# Push current branch
tachyon git push

# Push specific branch
tachyon git push origin feature/new-feature

# Push to specific workspace
tachyon git push origin feature/new-feature --workspace my-project

# Force push
tachyon git push origin main --force

# Push all branches
tachyon git push --all

# Push with tags
tachyon git push --tags

# Preview push
tachyon git push origin main --dry-run
```

#### 7.5.5. Output
Returns push result:
```json
{
  "remote": "origin",
  "branch": "main",
  "pushed": true,
  "forced": false,
  "ahead": 0,
  "new_commits": 5
}
```

### 7.6. Git Pull

#### 7.6.1. Syntax
```bash
tachyon git pull [OPTIONS]
```

#### 7.6.2. Parameters
- `<remote>` (optional): Remote to pull from (default: origin)
- `<branch>` (optional): Branch to pull (default: current)
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--rebase`: Rebase local commits
- `--no-ff`: Require fast-forward
- `--all`: Fetch all remotes
- `--prune`: Remove remote-tracking references

#### 7.6.3. Options
- `--dry-run`: Preview pull without changes
- `--stat`: Show diffstat
- `--log`: Show commit log
- `--depth <n>`: Limit fetch depth

#### 7.6.4. Examples
```bash
# Pull current branch
tachyon git pull

# Pull specific branch
tachyon git pull origin develop

# Pull with rebase
tachyon git pull --rebase

# Require fast-forward
tachyon git pull --no-ff

# Pull with log
tachyon git pull --log --depth 10

# Preview pull
tachyon git pull --dry-run
```

#### 7.6.5. Output
Returns pull result:
```json
{
  "remote": "origin",
  "branch": "main",
  "pulled": true,
  "commits_fetched": 3,
  "commits_merged": 2,
  "rebased": false,
  "fast_forward": true
}
```

### 7.7. Git Log

#### 7.7.1. Syntax
```bash
tachyon git log [OPTIONS]
```

#### 7.7.2. Parameters
- `<commit-ish>` (optional): Commit reference (default: HEAD)
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--max-count <n>` (optional): Limit number of commits
- `--since <date>` (optional): Show commits after date
- `--until <date>` (optional): Show commits before date
- `--author <pattern>` (optional): Filter by author
- `--grep <pattern>` (optional): Filter by message pattern
- `--format <format>` (optional): Output format
- `--oneline`: One line per commit
- `--graph`: ASCII graph

#### 7.7.3. Options
- `--pretty`: Custom format string
- `--abbrev-commit`: Show abbreviated hashes
- `--date`: Show date in format
- `--decorate`: Show branch/tag info
- `--no-walk`: Don't walk parents

#### 7.7.4. Format Specifiers
Format specifiers for `--pretty` option:
- `%H`: Commit hash
- `%h`: Abbreviated commit hash
- `%an`: Author name
- `%ae`: Author email
- `%ad`: Author date (RFC 2822)
- `%ar`: Author date (ISO 8601)
- `%s`: Subject (first line)
- `%b`: Body
- `%B`: Raw body
- `%n`: Newline
- `--format` options can be combined

#### 7.7.5. Examples
```bash
# Show last 10 commits
tachyon git log --max-count 10

# Show commits since specific date
tachyon git log --since "2026-01-01"

# Show commits in pretty format
tachyon git log --pretty=format:"%h - %an (%ae): %s" --abbrev-commit

# Show graph
tachyon git log --graph --decorate

# One line per commit
tachyon git log --oneline

# Custom format
tachyon git log --pretty=format:"%Cred %an: %s%n%n%b"
```

#### 7.7.6. Output
Returns commit history in specified format:
```
abc1234 (2026-02-07T10:00:00Z) John Doe <john@example.com>
    Add new API endpoint

abc1235 (2026-02-07T11:30:00Z) Jane Smith <jane@example.com>
    Fix typo in previous commit

abc1236 (2026-02-07T12:15:00Z) John Doe <john@example.com>
    Update documentation
```

JSON format:
```json
[
  {
    "hash": "abc1234",
    "message": "Add new API endpoint",
    "author": "John Doe <john@example.com>",
    "date": "2026-02-07T10:00:00Z"
  },
  {
    "hash": "abc1235",
    "message": "Fix typo in previous commit",
    "author": "Jane Smith <jane@example.com>",
    "date": "2026-02-07T11:30:00Z"
  }
]
```

---

## 8. SERVER COMMANDS

Server commands control the Tachyon server component, including startup, shutdown, monitoring, and configuration.

### 8.1. Server Start

#### 8.1.1. Syntax
```bash
tachyon server start [OPTIONS]
```

#### 8.1.2. Parameters
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--environment <env>` (optional): Environment (default: development)
- `--port <port>` (optional): Server port (default: 8080)
- `--host <host>` (optional): Bind address (default: 127.0.0.1)
- `--workers <n>` (optional): Number of workers (default: CPU count)

#### 8.1.3. Options
- `--daemon`: Run as daemon process
- `--reload`: Reload configuration on SIGHUP
- `--log-level <level>`: Logging level (error, warn, info, debug)
- `--config <path>`: Custom configuration file

#### 8.1.4. Examples
```bash
# Start server with defaults
tachyon server start

# Start in production mode
tachyon server start --environment production

# Start with custom port
tachyon server start --port 9000

# Start with specific workers
tachyon server start --workers 4

# Run as daemon
tachyon server start --daemon --log-level info

# Start with custom config
tachyon server start --config /etc/tachyon/server.toml
```

#### 8.1.5. Output
Returns server startup information:
```json
{
  "server_started": true,
  "environment": "development",
  "port": 8080,
  "host": "127.0.0.1",
  "workers": 8,
  "pid": 12345,
  "started_at": "2026-02-07T17:30:00Z"
}
```

### 8.2. Server Stop

#### 8.2.1. Syntax
```bash
tachyon server stop [OPTIONS]
```

#### 8.2.2. Parameters
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--graceful <seconds>` (optional): Grace period before force stop (default: 30)
- `--force`: Force immediate stop

#### 8.2.3. Options
- `--wait`: Wait for connections to close
- `--timeout <seconds>`: Maximum wait time (default: 60)

#### 8.2.4. Examples
```bash
# Stop server gracefully
tachyon server stop

# Force stop immediately
tachyon server stop --force

# Stop with grace period
tachyon server stop --graceful 60

# Wait for connections
tachyon server stop --wait --timeout 120
```

#### 8.2.5. Output
Returns stop confirmation:
```json
{
  "server_stopped": true,
  "graceful": true,
  "connections_closed": 5,
  "stopped_at": "2026-02-07T17:35:00Z"
}
```

### 8.3. Server Status

#### 8.3.1. Syntax
```bash
tachyon server status [OPTIONS]
```

#### 8.3.2. Parameters
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--format <format>` (optional): Output format (default: table)
- `--json`: Output in JSON format
- `--verbose`: Include detailed metrics

#### 8.3.3. Options
- `--metrics`: Include performance metrics
- `--connections`: Show active connections
- `--requests`: Show request statistics

#### 8.3.4. Examples
```bash
# Get server status
tachyon server status

# Status in JSON format
tachyon server status --json

# Status with metrics
tachyon server status --metrics --connections --requests

# Status for specific workspace
tachyon server status --workspace my-project
```

#### 8.3.5. Output
Table format (default):
```
+----------------+----------------+----------------+----------------+----------------+
| Status   | Uptime  | Connections | Requests | Memory   | CPU      |
+----------------+----------------+----------------+----------------+----------------+
| Running   | 2h 34m  | 15        | 1,234   | 2.1 GB  | 45%     |
+----------------+----------------+----------------+----------------+----------------+
```

JSON format:
```json
{
  "status": "running",
  "uptime_seconds": 9204,
  "active_connections": 15,
  "total_requests": 1234,
  "memory_mb": 2048,
  "memory_percent": 45,
  "cpu_percent": 45,
  "started_at": "2026-02-07T15:00:00Z"
}
```

### 8.4. Server Restart

#### 8.4.1. Syntax
```bash
tachyon server restart [OPTIONS]
```

#### 8.4.2. Parameters
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--graceful <seconds>` (optional): Grace period (default: 30)
- `--timeout <seconds>` (optional): Maximum wait time (default: 60)

#### 8.4.3. Options
- `--reload`: Reload configuration after restart
- `--zero-downtime`: Attempt zero-downtime restart

#### 8.4.4. Examples
```bash
# Restart server
tachyon server restart

# Graceful restart
tachyon server restart --graceful 60

# Restart with zero downtime attempt
tachyon server restart --zero-downtime --timeout 120
```

#### 8.4.5. Output
Returns restart confirmation:
```json
{
  "restarted": true,
  "graceful": true,
  "zero_downtime": false,
  "restarted_at": "2026-02-07T17:40:00Z"
}
```

### 8.5. Server Logs

#### 8.5.1. Syntax
```bash
tachyon server logs [OPTIONS]
```

#### 8.5.2. Parameters
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--lines <n>` (optional): Number of lines (default: 100)
- `--follow`: Follow log file (like tail -f)
- `--level <level>` (optional): Filter by log level (error, warn, info, debug)
- `--since <date>` (optional): Show logs since date
- `--until <date>` (optional): Show logs until date

#### 8.5.3. Options
- `--json`: Output in JSON format
- `--timestamps`: Include timestamps
- `--no-pager`: Disable paging

#### 8.5.4. Examples
```bash
# Show last 100 log lines
tachyon server logs --lines 100

# Follow log output
tachyon server logs --follow

# Show errors only
tachyon server logs --level error

# Show logs since specific time
tachyon server logs --since "2026-02-07T10:00:00Z"

# Logs in JSON format
tachyon server logs --json --timestamps --lines 50
```

#### 8.5.5. Output
Returns log entries in specified format:
```
2026-02-07T17:40:15.123Z [INFO] Server started on port 8080
2026-02-07T17:40:22.456Z [WARN] High memory usage detected: 85%
2026-02-07T17:40:30.789Z [ERROR] Connection refused: Connection timeout
```

JSON format:
```json
[
  {
    "timestamp": "2026-02-07T17:40:15.123Z",
    "level": "INFO",
    "message": "Server started on port 8080"
  },
  {
    "timestamp": "2026-02-07T17:40:22.456Z",
    "level": "WARN",
    "message": "High memory usage detected: 85%"
  }
]
```

### 8.6. Server Config

#### 8.6.1. Syntax
```bash
tachyon server config [OPTIONS]
```

#### 8.6.2. Parameters
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--get <key>`: Get specific configuration value
- `--set <key> <value>`: Set configuration value
- `--list`: List all configuration
- `--reset`: Reset to defaults

#### 8.6.3. Options
- `--json`: Output in JSON format
- `--show-secrets`: Include secret values (use with caution)

#### 8.6.4. Examples
```bash
# List all configuration
tachyon server config --list

# Get specific configuration value
tachyon server config --get server.port

# Set configuration value
tachyon server config --set log.level debug

# Reset to defaults
tachyon server config --reset
```

#### 8.6.5. Output
Returns configuration information:
```json
{
  "server_port": 8080,
  "log_level": "info",
  "max_connections": 100,
  "timeout_seconds": 60,
  "workers": 4
}
```

---

## 9. PLUGIN COMMANDS

Plugin commands manage the Tachyon plugin system, including listing, installing, uninstalling, and querying plugins.

### 9.1. Plugin List

#### 9.1.1. Syntax
```bash
tachyon plugin list [OPTIONS]
```

#### 9.1.2. Parameters
- `--workspace <workspace>` (optional): List plugins in specific workspace (default: current)
- `--installed`: Show only installed plugins
- `--format <format>` (optional): Output format (default: table)
- `--json`: Output in JSON format
- `--verbose`: Include detailed plugin information

#### 9.1.3. Options
- `--all`: Include system and user plugins
- `--system`: Show system plugins only

#### 9.1.4. Examples
```bash
# List all plugins
tachyon plugin list

# List installed plugins only
tachyon plugin list --installed

# List in JSON format
tachyon plugin list --json

# List for specific workspace
tachyon plugin list --workspace my-project

# Verbose output
tachyon plugin list --verbose
```

#### 9.1.5. Output
Table format (default):
```
+----------------+----------------+----------------+----------------+
| Plugin Name | Version    | Type   | Status    | Author              | Description           |
+----------------+----------------+----------------+----------------+
| tachyon-git | 1.2.3     | system  | active    | Tachyon Core Team   | Git integration plugin |
| markdown-preview| 0.5.0      | user    | active    | Markdown preview      | Live markdown rendering |
| api-validator | 2.1.0      | user    | active    | API validation tool   | Validate API responses |
+----------------+----------------+----------------+----------------+
```

JSON format:
```json
[
  {
    "name": "tachyon-git",
    "version": "1.2.3",
    "type": "system",
    "status": "active",
    "author": "Tachyon Core Team",
    "description": "Git integration plugin",
    "installed_at": "2026-02-07T10:00:00Z",
    "enabled": true
  },
  {
    "name": "markdown-preview",
    "version": "0.5.0",
    "type": "user",
    "status": "active",
    "author": "John Doe",
    "description": "Live markdown rendering",
    "installed_at": "2026-02-07T12:00:00Z",
    "enabled": true
  }
]
```

### 9.2. Plugin Install

#### 9.2.1. Syntax
```bash
tachyon plugin install <plugin-name> [OPTIONS]
```

#### 9.2.2. Parameters
- `<plugin-name>` (required): Name of plugin to install
- `--version <version>` (optional): Specific version to install
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--system`: Install to system plugins directory
- `--force`: Reinstall if already exists
- `--dry-run`: Preview installation without changes

#### 9.2.3. Options
- `--from <path>` (optional): Install from local file
- `--no-verify`: Skip plugin signature verification

#### 9.2.4. Examples
```bash
# Install plugin from registry
tachyon plugin install api-validator

# Install specific version
tachyon plugin install markdown-preview --version 0.4.5

# Install from local file
tachyon plugin install my-plugin --from ./my-plugin.tar.gz

# Install to system directory
tachyon plugin install tachyon-git --system

# Force reinstall
tachyon plugin install api-validator --force

# Preview installation
tachyon plugin install api-validator --dry-run
```

#### 9.2.5. Output
Returns installation result:
```json
{
  "plugin_name": "api-validator",
  "version": "2.1.0",
  "installed": true,
  "type": "user",
  "installed_at": "2026-02-07T17:30:00Z",
  "location": "/home/user/.tachyon/plugins/",
  "verified": true
}
```

### 9.3. Plugin Uninstall

#### 9.3.1. Syntax
```bash
tachyon plugin uninstall <plugin-name> [OPTIONS]
```

#### 9.3.2. Parameters
- `<plugin-name>` (required): Name of plugin to uninstall
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--system`: Remove from system plugins directory
- `--force`: Remove even if in use
- `--purge`: Remove all plugin data

#### 9.3.3. Options
- `--dry-run`: Preview uninstallation without changes
- `--keep-config`: Keep plugin configuration files

#### 9.3.4. Examples
```bash
# Uninstall plugin
tachyon plugin uninstall api-validator

# Uninstall from system directory
tachyon plugin uninstall tachyon-git --system

# Force uninstall
tachyon plugin uninstall api-validator --force

# Uninstall and remove all data
tachyon plugin uninstall markdown-preview --purge

# Preview uninstallation
tachyon plugin uninstall api-validator --dry-run

# Keep configuration files
tachyon plugin uninstall api-validator --keep-config
```

#### 9.3.5. Output
Returns uninstallation result:
```json
{
  "plugin_name": "api-validator",
  "uninstalled": true,
  "data_removed": false,
  "config_kept": false,
  "uninstalled_at": "2026-02-07T17:30:00Z"
}
```

### 9.4. Plugin Info

#### 9.4.1. Syntax
```bash
tachyon plugin info <plugin-name> [OPTIONS]
```

#### 9.4.2. Parameters
- `<plugin-name>` (required): Name of plugin to query
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--json`: Output in JSON format

#### 9.4.3. Options
- `--commands`: List commands provided by plugin
- `--dependencies`: Show plugin dependencies

#### 9.4.4. Examples
```bash
# Get plugin information
tachyon plugin info api-validator

# Get info in JSON format
tachyon plugin info api-validator --json

# Show plugin commands
tachyon plugin info api-validator --commands

# Show dependencies
tachyon plugin info api-validator --dependencies
```

#### 9.4.5. Output
Returns detailed plugin information:
```json
{
  "name": "api-validator",
  "version": "2.1.0",
  "type": "user",
  "status": "active",
  "author": "Tachyon Core Team",
  "description": "API validation tool",
  "installed_at": "2026-02-07T10:00:00Z",
  "enabled": true,
  "location": "/home/user/.tachyon/plugins/",
  "commands": [
    {
      "name": "validate",
      "description": "Validate API responses against schema"
    },
    {
      "name": "check",
      "description": "Check API documentation completeness"
    }
  ],
  "dependencies": [
    {
      "name": "serde",
      "version": "1.0.193",
      "type": "runtime"
    },
    {
      "name": "tokio",
      "version": "1.35.0",
      "type": "runtime"
    }
  ]
}
```

### 9.5. Plugin Enable/Disable

#### 9.5.1. Syntax
```bash
tachyon plugin enable <plugin-name> [OPTIONS]
tachyon plugin disable <plugin-name> [OPTIONS]
```

#### 9.5.2. Parameters
- `<plugin-name>` (required): Name of plugin to enable/disable
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--persist`: Save enabled state across restarts

#### 9.5.3. Options
- `--temporary`: Enable for current session only

#### 9.5.4. Examples
```bash
# Enable plugin
tachyon plugin enable api-validator

# Enable for current session only
tachyon plugin enable api-validator --temporary

# Enable and persist
tachyon plugin enable markdown-preview --persist

# Disable plugin
tachyon plugin disable api-validator

# Disable plugin for current session
tachyon plugin disable api-validator --temporary
```

#### 9.5.5. Output
Returns enable/disable result:
```json
{
  "plugin_name": "api-validator",
  "enabled": true,
  "persisted": false,
  "temporary": false,
  "changed_at": "2026-02-07T17:30:00Z"
}
```

---

## 10. CONFIGURATION COMMANDS

Configuration commands manage Tachyon CLI configuration, including viewing, setting, and resetting configuration values.

### 10.1. Config Get

#### 10.1.1. Syntax
```bash
tachyon config get <key> [OPTIONS]
```

#### 10.1.2. Parameters
- `<key>` (required): Configuration key to retrieve
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--format <format>` (optional): Output format (default: table)
- `--json`: Output in JSON format
- `--show-secrets`: Include secret values (use with caution)

#### 10.1.3. Options
- `--default`: Show default value if key not set
- `--all`: Show all configuration keys
- `--verbose`: Include detailed information

#### 10.1.4. Examples
```bash
# Get specific configuration value
tachyon config get server.port

# Get value with default fallback
tachyon config get log.level --default

# Get in JSON format
tachyon config get server.port --json

# Show secrets (use with caution)
tachyon config get api.token --show-secrets

# Get all configuration
tachyon config get --all --verbose
```

#### 10.1.5. Output
Returns configuration value in specified format:
```json
{
  "key": "server.port",
  "value": "8080",
  "source": "default",
  "type": "integer"
  "description": "Server port number"
}
```

### 10.2. Config Set

#### 10.2.1. Syntax
```bash
tachyon config set <key> <value> [OPTIONS]
```

#### 10.2.2. Parameters
- `<key>` (required): Configuration key to set
- `<value>` (required): Configuration value to assign
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--persist`: Save to configuration file

#### 10.2.3. Options
- `--type <type>` (optional): Type annotation for value
- `--dry-run`: Preview change without saving
- `--force`: Overwrite existing value

#### 10.2.4. Examples
```bash
# Set server port
tachyon config set server.port 9000

# Set log level
tachyon config set log.level debug

# Set with type annotation
tachyon config set cache.size 1024 --type integer

# Persist to file
tachyon config set workspace.name my-project --persist

# Preview change
tachyon config set api.token new-token --dry-run

# Force overwrite
tachyon config set server.force true --force
```

#### 10.2.5. Output
Returns set confirmation:
```json
{
  "key": "server.port",
  "old_value": "8080",
  "new_value": "9000",
  "persisted": true,
  "changed_at": "2026-02-07T17:30:00Z"
}
```

### 10.3. Config List

#### 10.3.1. Syntax
```bash
tachyon config list [OPTIONS]
```

#### 10.3.2. Parameters
- `--workspace <workspace>` (optional): List configuration for specific workspace (default: current)
- `--format <format>` (optional): Output format (default: table)
- `--json`: Output in JSON format
- `--verbose`: Include values and sources

#### 10.3.3. Options
- `--all`: Show all configuration including defaults
- `--show-sources`: Show configuration source for each key

#### 10.3.4. Examples
```bash
# List all configuration
tachyon config list

# List in JSON format
tachyon config list --json

# List with sources
tachyon config list --show-sources --verbose

# List for specific workspace
tachyon config list --workspace my-project
```

#### 10.3.5. Output
Table format (default):
```
+----------------+----------------+----------------+----------------+----------------+
| Key         | Value    | Type   | Source    | Workspace |
+----------------+----------------+----------------+----------------+
| server.port   | 8080    | integer | default   | default   |
| log.level     | info     | string  | user     | my-project |
| workspace.name | my-project | string  | user     | my-project |
+----------------+----------------+----------------+----------------+
```

JSON format:
```json
[
  {
    "key": "server.port",
    "value": "8080",
    "type": "integer",
    "source": "default",
    "workspace": "default"
  },
  {
    "key": "log.level",
    "value": "info",
    "type": "string",
    "source": "user",
    "workspace": "my-project"
  }
]
```

### 10.4. Config Reset

#### 10.4.1. Syntax
```bash
tachyon config reset [OPTIONS]
```

#### 10.4.2. Parameters
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--key <key>` (optional): Reset specific key only
- `--all`: Reset all configuration to defaults
- `--dry-run`: Preview reset without changes

#### 10.4.3. Options
- `--force`: Reset even if key not set
- `--keep-workspace`: Preserve workspace-specific values

#### 10.4.4. Examples
```bash
# Reset all configuration to defaults
tachyon config reset --all

# Reset specific key
tachyon config reset server.port --force

# Preview reset
tachyon config reset log.level --dry-run

# Reset but keep workspace values
tachyon config reset workspace.name --keep-workspace
```

#### 10.4.5. Output
Returns reset confirmation:
```json
{
  "reset_keys": ["server.port", "log.level", "workspace.name"],
  "preserved_keys": ["api.token", "cache.size"],
  "reset_at": "2026-02-07T17:30:00Z"
}
```

### 10.5. Config Validate

#### 10.5.1. Syntax
```bash
tachyon config validate [OPTIONS]
```

#### 10.5.2. Parameters
- `--workspace <workspace>` (optional): Target workspace (default: current)
- `--format <format>` (optional): Output format (default: table)
- `--json`: Output in JSON format

#### 10.5.3. Options
- `--all`: Validate all configuration
- `--show-secrets`: Include secrets in output

#### 10.5.4. Examples
```bash
# Validate all configuration
tachyon config validate --all

# Validate in JSON format
tachyon config validate --json

# Validate and show secrets
tachyon config validate --show-secrets
```

#### 10.5.5. Output
Returns validation results:
```json
{
  "valid": true,
  "errors": [],
  "warnings": [
    {
      "key": "api.token",
      "message": "API token not set, using default value"
    }
  ],
  "configuration": {
    "server.port": {
      "value": 8080,
      "valid": true,
      "type": "integer"
    },
    "log.level": {
      "value": "info",
      "valid": true,
      "type": "string",
      "in_range": ["error", "warn", "info", "debug"]
    }
  }
}
```

---

## 13. REFERENCES

This section provides references to related documentation, specifications, and standards referenced throughout the CLI API documentation.

### 13.1. Documentation Standards

- **[TACHYON-STD-V1.0](.adrs/ - Coding and Documentation Standards
  Defines conventions for all Tachyon documentation including formatting, structure, and quality requirements

### 13.2. Requirements Specification

- **[TACHYON-REQ-V1.0](.adrs/ - Requirements Specification
  Contains all functional and non-functional requirements for the Tachyon toolchain

### 13.3. Architectural Decisions

- **[ADR-001](.adrs/adr-001-three-tier-jit-compilation.md) - Rust Language Decision
  Rationale for selecting Rust as the implementation language for Tachyon CLI

- **[ADR-002](.adrs/adr-002-bm25-search-parameters.md) - Tokio Runtime Decision
  Rationale for using Tokio for asynchronous operations in the CLI

- **[ADR-010](.adrs/adr-010-synchronization-primitives.md) - Security Architecture
  Defines security considerations for CLI implementation including authentication and authorization

### 13.4. API Documentation

- **[TACHYON-API-001-V1.0](.adrs/ - API Overview and Conventions
  Establishes API design principles and conventions used across all Tachyon APIs

- **[TACHYON-API-008-V1.0](.adrs/ - Desktop API Specification
  Documents the Tauri-based desktop application API that CLI interacts with

- **[TACHYON-API-009-V1.0](.adrs/ - Server HTTP API Specification
  Documents the Axum-based server HTTP API that CLI can interact with

- **[TACHYON-API-010-V1.0](.adrs/ - Server WebSocket API Specification
  Documents the WebSocket API for real-time communication

### 13.5. Design Documents

- **[DSN-001](.adrs/ - System Architecture Design
  Provides overall system architecture including CLI integration points

- **[DSN-011](.adrs/ - API Design
  Provides detailed API design specifications

### 13.6. Test Plan

- **[TACHYON-TST-V1.0](.adrs/ - Test Plan
  Defines testing strategy and acceptance criteria for CLI commands

### 13.7. Technology Documentation

- **[clap](https://docs.rs/clap/) - clap Documentation
  Official documentation for the clap command-line argument parser used by Tachyon CLI

- **[Tokio](https://tokio.rs/) - Tokio Documentation
  Official documentation for the Tokio asynchronous runtime used by Tachyon CLI

- **[serde](https://serde.rs/) - serde Documentation
  Official documentation for the serde serialization/deserialization library used by Tachyon CLI

### 13.8. External Standards

- **[ISO/IEC 26514:2021](https://www.iso.org/standard/26514) - Systems and Software Engineering — Requirements for Documentation
  International standard for documentation quality and structure

- **[IEEE 8294:2021](https://standards.ieee.org/) - IEEE Standard for Software Documentation
  Industry standard for software engineering documentation

### 13.9. Related Tools

- **[Rust](https://www.rust-lang.org/) - Rust Programming Language
  Official Rust language documentation and tools

- **[Cargo](https://doc.rust-lang.org/cargo/) - Cargo Package Manager
  Official Rust package manager documentation

### 13.10. Version History

- **TACHYON-CHG-V1.0](.adrs/ - Change History and Versioning
  Documents version history and change management procedures for Tachyon

---

## APPENDICES

### Appendix A: Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success | Command completed successfully |
| 1 | General Error | Error occurred during command execution |
| 2 | Usage Error | Invalid command usage |
| 3 | Validation Error | Input validation failed |
| 4 | Configuration Error | Configuration error |
| 5 | Network Error | Network communication failure |
| 6 | File System Error | File system operation failed |
| 7 | Permission Error | Insufficient permissions |
| 8 | Plugin Error | Plugin operation failed |
| 9 | Git Error | Git operation failed |
| 10 | Server Error | Server operation failed |
| 11 | Timeout | Operation timed out |
| 12 | Interrupted | Operation cancelled by user |
| 13 | Unknown | Unknown error |

### Appendix B: Configuration Keys

| Key | Type | Default | Description | Valid Values |
|------|--------|----------|-------------|
| `server.port` | integer | 8080 | Server HTTP port (1024-65535) | 1024-65535 |
| `log.level` | string | info | Logging level (error, warn, info, debug) | error, warn, info, debug |
| `workspace.name` | string | default | Workspace name | Any valid string |
| `api.token` | string | (auto-generated) | API authentication token | Any valid string |
| `cache.size` | integer | 1024 | Cache size in MB | 256-8192 | 256-8192 |
| `server.host` | string | 127.0.0.1 | Server bind address | Any valid IP address |
| `workers` | integer | auto | Number of workers | 1-64 | 1-64 |
| `timeout.seconds` | integer | 60 | Request timeout in seconds | 1-3600 | 1-3600 |

### Appendix C: Environment Variables

| Variable | Type | Description |
|------|----------|-------------|
| `TACHYON_CONFIG_DIR` | string | Configuration directory path | `~/.config/tachyon/` |
| `TACHYON_CONFIG_FILE` | string | Configuration file path | `config.toml` |
| `TACHYON_WORKSPACE` | string | Workspace directory path | Current working directory |
| `TACHYON_VERBOSITY` | string | Logging level | `info` | error, warn, info, debug |
| `TACHYON_QUIET` | boolean | Quiet mode | `false` |
| `TACHYON_COLOR` | string | Color output | `auto` | always, auto, never |
| `TACHYON_OUTPUT_FORMAT` | string | Output format | `table` | table, json, yaml, toml, plain |
| `TACHYON_PAGER` | string | Pager command | System default | `less` |
| `TACHYON_DRY_RUN` | boolean | Dry run mode | `false` |
| `TACHYON_FORCE` | boolean | Force mode | `false` |
| `TACHYON_INTERACTIVE` | boolean | Interactive mode | `false` |
| `TACHYON_TOKEN` | string | API authentication token | (from config) |
| `TACHYON_API_KEY` | string | API key | (from config) |
| `TACHYON_API_ENDPOINT` | string | API endpoint | (from config) |

### Appendix D: Command Aliases

| Alias | Command | Description |
|------|----------|-------------|
| `h` | help | Display help information |
| `v` | verbose | Enable verbose output |
| `q` | quiet | Suppress non-error output |
| `n` | dry-run | Preview without changes |
| `f` | force | Bypass confirmations |
| `i` | interactive | Enable interactive prompts |

---

**Document Control:**

This document is version **TACHYON-API-006-V1.0**.

**Last Updated:** 2026-02-07T17:35:00Z
**Status:** Approved for Execution
**Classification:** API Specification
**Compliance Level:** ISO/IEC 26514:2021, IEEE 8294:2021
