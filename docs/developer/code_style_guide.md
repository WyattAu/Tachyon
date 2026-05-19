# TACHYON: CODE STYLE GUIDE

**Document ID:** TACHYON-DEV-008-V1.0
**Date:** February 2026
**Status:** Approved for Implementation
**Classification:** Developer Documentation
**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [Style Framework](#2-style-framework)
3. [Rust Style Guidelines](#3-rust-style-guidelines)
4. [TypeScript Style Guidelines](#4-typescript-style-guidelines)
5. [Naming Conventions](#5-naming-conventions)
6. [Formatting Rules](#6-formatting-rules)
7. [Documentation Style](#7-documentation-style)
8. [Error Handling Style](#8-error-handling-style)
9. [References](#9-references)

---

## 1. INTRODUCTION

### 1.1. Purpose and Scope

This document establishes the comprehensive code style guidelines governing the Tachyon toolchain project. These guidelines are mandatory for all contributors and are designed to ensure consistency, maintainability, and compliance with international standards at the PhD thesis level of rigor.

The Tachyon project encompasses:
- A Rust-based core engine with Tokio asynchronous runtime
- A Tauri-based desktop application wrapper
- An Axum-based HTTP/2 server component
- A TypeScript/JavaScript frontend using Leptos and TailwindCSS
- Git-based content storage and management

### 1.2. Applicability

These guidelines apply to:
1. All source code artifacts (Rust, TypeScript, JavaScript)
2. All test code artifacts
3. All configuration files (TOML, JSON, YAML)
4. All build scripts and tooling configurations
5. All inline documentation and comments

### 1.3. Rationale

The establishment of rigorous code style guidelines is justified by:
- **Deterministic Quality:** Ensuring consistent code quality across all contributors
- **Maintainability:** Facilitating long-term maintenance and evolution
- **Readability:** Enhancing code comprehension and reducing cognitive load
- **Tooling Compatibility:** Ensuring compatibility with automated formatters and linters
- **Academic Rigor:** Maintaining PhD thesis level precision in all code artifacts

---

## 2. STYLE FRAMEWORK

### 2.1. Guiding Principles

The Tachyon code style framework is founded upon the following principles:

#### 2.1.1. Clarity Over Cleverness

**Principle:** Code shall prioritize clarity and readability over cleverness or conciseness.

**Rationale:** Clear code is easier to understand, maintain, and debug. Clever code that sacrifices clarity increases cognitive load and introduces maintenance risks.

**Examples:**
- [PASS] Use descriptive variable names that explain their purpose
- [PASS] Write straightforward logic that can be understood at a glance
- [FAIL] Avoid obscure language features that require deep knowledge
- [FAIL] Avoid code golf or excessive one-liners

#### 2.1.2. Consistency Over Convention

**Principle:** Consistency within the codebase takes precedence over external conventions or personal preferences.

**Rationale:** Consistent code reduces cognitive load when navigating the codebase. Readers should not encounter different styles for similar constructs.

**Examples:**
- [PASS] Follow existing patterns in the codebase
- [PASS] Use the same naming conventions across modules
- [FAIL] Avoid introducing new conventions without consensus
- [FAIL] Avoid mixing different styles for similar constructs

#### 2.1.3. Explicit Over Implicit

**Principle:** Code shall make intent explicit rather than relying on implicit behavior.

**Rationale:** Explicit code is self-documenting and reduces the cognitive load required to understand behavior. Implicit behavior often requires knowledge of language-specific rules.

**Examples:**
- [PASS] Use explicit type annotations for public interfaces
- [PASS] Use explicit error handling with clear error messages
- [FAIL] Avoid relying on implicit type conversions
- [FAIL] Avoid relying on implicit default values

#### 2.1.4. Safety Over Performance

**Principle:** Code shall prioritize safety and correctness over micro-optimizations.

**Rationale:** Safety and correctness are fundamental requirements. Performance optimizations should only be applied when necessary and after profiling.

**Examples:**
- [PASS] Use safe Rust code unless unsafe is absolutely necessary
- [PASS] Use validated input handling
- [FAIL] Avoid premature optimization
- [FAIL] Avoid unsafe code without rigorous justification

#### 2.1.5. Documentation Over Assumption

**Principle:** Code shall be documented to explain intent, rationale, and usage.

**Rationale:** Documentation captures knowledge that cannot be expressed in code alone. It explains why code exists, not just what it does.

**Examples:**
- [PASS] Document public APIs with clear descriptions
- [PASS] Explain non-obvious algorithms or design decisions
- [PASS] Document invariants and preconditions
- [FAIL] Avoid assuming code is self-explanatory
- [FAIL] Avoid omitting documentation for complex logic

### 2.2. Tooling Integration

The Tachyon project integrates automated tooling to enforce code style guidelines:

#### 2.2.1. Rust Tooling

**rustfmt:** Automatic code formatter for Rust code
- Configuration: `rustfmt.toml` in project root
- Enforcement: Pre-commit hooks and CI pipeline
- Coverage: All `.rs` files

**Clippy:** Linting tool for Rust code
- Configuration: `clippy.toml` in project root
- Enforcement: Pre-commit hooks and CI pipeline
- Coverage: All `.rs` files

**rust-analyzer:** Language server for IDE support
- Provides real-time feedback on code style
- Enforces style guidelines during development

#### 2.2.2. TypeScript Tooling

**ESLint:** Linting tool for TypeScript/JavaScript code
- Configuration: `.eslintrc.json` in project root
- Enforcement: Pre-commit hooks and CI pipeline
- Coverage: All `.ts` and `.js` files

**Prettier:** Code formatter for TypeScript/JavaScript code
- Configuration: `.prettierrc` in project root
- Enforcement: Pre-commit hooks and CI pipeline
- Coverage: All `.ts`, `.js`, `.json`, and `.md` files

**TypeScript Compiler:** Type checking and style enforcement
- Configuration: `tsconfig.json` in project root
- Enforcement: Pre-commit hooks and CI pipeline
- Coverage: All `.ts` files

### 2.3. Enforcement Mechanisms

#### 2.3.1. Pre-Commit Hooks

Pre-commit hooks enforce code style guidelines before code is committed to the repository.

**Hook Triggers:**
- File modification or creation
- Staging of files for commit

**Hook Actions:**
- Run formatters (rustfmt, Prettier)
- Run linters (Clippy, ESLint)
- Run type checkers (rustc, tsc)
- Fail commit if style violations are detected

#### 2.3.2. Continuous Integration Pipeline

The CI pipeline enforces code style guidelines on all pull requests.

**Pipeline Stages:**
1. **Style Check:** Run formatters and linters
2. **Type Check:** Verify type safety
3. **Build:** Ensure code compiles successfully
4. **Test:** Verify functionality

**Failure Conditions:**
- Style violations detected by linters
- Type errors detected by compilers
- Build failures
- Test failures

#### 2.3.3. Code Review Process

Code reviews verify adherence to code style guidelines.

**Review Checklist:**
- [ ] Code follows style guidelines
- [ ] Code is formatted correctly
- [ ] Code passes all linters
- [ ] Code is documented appropriately
- [ ] Code is clear and readable

**Review Outcomes:**
- **Approved:** Code meets all style guidelines
- **Request Changes:** Code has style violations that must be addressed
- **Commented:** Code has minor style suggestions (non-blocking)

---

## 3. RUST STYLE GUIDELINES

### 3.1. Rust Edition and Version

**Standard:** All Rust code shall use Rust Edition 2024 with minimum supported Rust version (MSRV) of 1.77.2 for desktop component and 1.80+ for server component.

**Rationale:** Rust Edition 2024 provides the latest language features and improvements while maintaining compatibility with stable Rust toolchain. The MSRV ensures consistent behavior across development environments.

**Reference:** [ADR-001: Rust as Primary Language](../../.adrs/adr-001-three-tier-jit-compilation.md)

### 3.2. Type Annotations

#### 3.2.1. Public Interface Annotations

**Standard:** All public functions, methods, and struct fields shall have explicit type annotations.

**Rationale:** Explicit type annotations improve code documentation, enable better IDE support, and make the contract of public interfaces clear.

**Examples:**
```rust
// [PASS] Correct: Explicit type annotation for public function
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// [FAIL] Incorrect: Missing type annotation for public function
pub fn calculate_area(width, height) {
    width * height
}

// [PASS] Correct: Explicit type annotation for public struct field
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

// [FAIL] Incorrect: Missing type annotation for public struct field
pub struct Rectangle {
    pub width,
    pub height,
}
```

#### 3.2.2. Local Variable Annotations

**Standard:** Local variables shall omit type annotations when the type can be inferred by the compiler.

**Rationale:** Type inference reduces verbosity while maintaining type safety. The compiler verifies type correctness.

**Examples:**
```rust
// [PASS] Correct: Type inferred from literal
let count = 42;

// [FAIL] Incorrect: Redundant type annotation
let count: i32 = 42;

// [PASS] Correct: Type inferred from function return
let result = calculate_area(10.0, 20.0);

// [FAIL] Incorrect: Redundant type annotation
let result: f64 = calculate_area(10.0, 20.0);
```

#### 3.2.3. Complex Type Annotations

**Standard:** Complex types (closures, iterators, trait objects) shall use explicit type annotations when inference is unclear.

**Rationale:** Explicit annotations for complex types improve readability and make the code's intent clear.

**Examples:**
```rust
// [PASS] Correct: Explicit type for complex closure
let filter_fn: Box<dyn Fn(&i32) -> bool> = Box::new(|x| x > 0);

// [PASS] Correct: Explicit type for complex iterator
let numbers: Vec<i32> = (0..100).filter(|x| x % 2 == 0).collect();

// [PASS] Correct: Explicit type for trait object
let writer: Box<dyn std::io::Write> = Box::new(std::io::stdout());
```

### 3.3. Ownership and Borrowing

#### 3.3.1. Borrowing Rules

**Standard:** Code shall adhere to Rust's borrowing rules: multiple immutable references OR one mutable reference.

**Rationale:** The borrowing rules prevent data races at compile time, ensuring memory safety without runtime overhead.

**Examples:**
```rust
// [PASS] Correct: Multiple immutable references
fn process_values(values: &[i32]) {
    let first = &values[0];
    let second = &values[1];
    // Both references are valid here
}

// [PASS] Correct: Single mutable reference
fn modify_values(values: &mut [i32]) {
    values[0] = 42;
}

// [FAIL] Incorrect: Multiple mutable references
fn invalid_borrow(values: &mut [i32]) {
    let first = &mut values[0];
    let second = &mut values[1];  // Compile-time error
}
```

#### 3.3.2. Lifetime Annotations

**Standard:** Lifetime annotations shall be explicit when the compiler cannot infer lifetimes.

**Rationale:** Explicit lifetime annotations document the relationship between references and prevent dangling references.

**Examples:**
```rust
// [PASS] Correct: Explicit lifetime annotation
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// [FAIL] Incorrect: Missing lifetime annotation
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

#### 3.3.3. Lifetime Elision

**Standard:** Lifetime elision rules shall be leveraged when lifetimes can be inferred.

**Rationale:** Lifetime elision reduces verbosity while maintaining safety.

**Examples:**
```rust
// [PASS] Correct: Lifetime elision applies (single input, output is reference)
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

// [PASS] Correct: Lifetime elision applies (method with self)
impl String {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}
```

### 3.4. Error Handling

#### 3.4.1. Result Type Usage

**Standard:** Functions that can fail shall return `Result<T, E>` rather than panicking.

**Rationale:** The `Result` type makes error handling explicit and forces callers to handle errors.

**Examples:**
```rust
// [PASS] Correct: Return Result for fallible operation
pub fn parse_number(s: &str) -> Result<i32, ParseIntError> {
    s.parse::<i32>()
}

// [FAIL] Incorrect: Panic on error
pub fn parse_number(s: &str) -> i32 {
    s.parse::<i32>().unwrap()
}
```

#### 3.4.2. Option Type Usage

**Standard:** Functions that may not return a value shall return `Option<T>` rather than panicking.

**Rationale:** The `Option` type makes the absence of a value explicit and forces callers to handle it.

**Examples:**
```rust
// [PASS] Correct: Return Option for optional value
pub fn find_first(items: &[i32], target: i32) -> Option<usize> {
    items.iter().position(|&x| x == target)
}

// [FAIL] Incorrect: Panic on absence
pub fn find_first(items: &[i32], target: i32) -> usize {
    items.iter().position(|&x| x == target).unwrap()
}
```

#### 3.4.3. Error Propagation

**Standard:** Errors shall be propagated using the `?` operator when appropriate.

**Rationale:** The `?` operator provides concise error propagation while maintaining explicit error handling.

**Examples:**
```rust
// [PASS] Correct: Use ? operator for error propagation
pub fn read_config(path: &str) -> Result<Config, IoError> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

// [FAIL] Incorrect: Manual error propagation
pub fn read_config(path: &str) -> Result<Config, IoError> {
    let content = fs::read_to_string(path).map_err(|e| IoError::Read(e))?;
    let config: Config = serde_json::from_str(&content).map_err(|e| IoError::Parse(e))?;
    Ok(config)
}
```

### 3.5. Pattern Matching

#### 3.5.1. Exhaustive Matching

**Standard:** All `match` expressions shall be exhaustive, covering all possible variants.

**Rationale:** Exhaustive matching ensures all cases are handled, preventing runtime panics from unmatched patterns.

**Examples:**
```rust
// [PASS] Correct: Exhaustive match on enum
enum Color {
    Red,
    Green,
    Blue,
}

fn color_name(color: Color) -> &'static str {
    match color {
        Color::Red => "red",
        Color::Green => "green",
        Color::Blue => "blue",
    }
}

// [FAIL] Incorrect: Non-exhaustive match
fn color_name(color: Color) -> &'static str {
    match color {
        Color::Red => "red",
        Color::Green => "green",
        // Missing Color::Blue case
    }
}
```

#### 3.5.2. Wildcard Patterns

**Standard:** Wildcard patterns (`_`) shall be used with caution and documented.

**Rationale:** Wildcard patterns can hide bugs by silently ignoring cases. Documentation explains the intent.

**Examples:**
```rust
// [PASS] Correct: Documented wildcard pattern
fn process_result(result: Result<i32, Error>) {
    match result {
        Ok(value) => println!("Success: {}", value),
        Err(_) => println!("Error occurred"),  // Ignoring specific error
    }
}

// [PASS] Correct: Wildcard with binding for logging
fn process_result(result: Result<i32, Error>) {
    match result {
        Ok(value) => println!("Success: {}", value),
        Err(e) => eprintln!("Error: {}", e),  // Binding for logging
    }
}
```

#### 3.5.3. Guard Clauses

**Standard:** Match guards (`if` clauses) shall be used for complex pattern conditions.

**Rationale:** Match guards enable more expressive pattern matching without introducing additional enum variants.

**Examples:**
```rust
// [PASS] Correct: Match guard for complex condition
fn classify_number(n: i32) -> &'static str {
    match n {
        x if x < 0 => "negative",
        0 => "zero",
        x if x > 0 => "positive",
        _ => unreachable!(),
    }
}

// [FAIL] Incorrect: Additional enum variant for condition
enum Number {
    Negative,
    Zero,
    Positive,
}

fn classify_number(n: i32) -> Number {
    if n < 0 {
        Number::Negative
    } else if n == 0 {
        Number::Zero
    } else {
        Number::Positive
    }
}
```

### 3.6. Async/Await

#### 3.6.1. Async Function Signature

**Standard:** Async functions shall use the `async` keyword and return types shall be inferred.

**Rationale:** The `async` keyword provides clear indication of asynchronous behavior. Inferred return types reduce verbosity.

**Examples:**
```rust
// [PASS] Correct: Async function with inferred return type
pub async fn fetch_document(id: &str) -> Result<Document, Error> {
    let response = http_client.get(format!("/documents/{}", id)).send().await?;
    Ok(response.json().await?)
}

// [FAIL] Incorrect: Explicit Future return type
pub async fn fetch_document(id: &str) -> Pin<Box<dyn Future<Output = Result<Document, Error>> + Send>> {
    // Unnecessary complexity
}
```

#### 3.6.2. Await Usage

**Standard:** The `.await` keyword shall be used for all async operations.

**Rationale:** The `.await` keyword makes suspension points explicit, improving code readability.

**Examples:**
```rust
// [PASS] Correct: Explicit await for async operations
pub async fn process_documents() -> Result<(), Error> {
    let docs = fetch_all_documents().await?;
    for doc in docs {
        process_document(&doc).await?;
    }
    Ok(())
}

// [FAIL] Incorrect: Blocking on async operation
pub fn process_documents() -> Result<(), Error> {
    let docs = tokio::runtime::Runtime::new().unwrap().block_on(fetch_all_documents())?;
    for doc in docs {
        tokio::runtime::Runtime::new().unwrap().block_on(process_document(&doc))?;
    }
    Ok(())
}
```

#### 3.6.3. Tokio Runtime

**Standard:** Tokio shall be used as the async runtime with the `multi-threaded` scheduler.

**Rationale:** Tokio provides the de facto async runtime for Rust with mature ecosystem support. Multi-threaded scheduler enables efficient parallel processing.

**Reference:** [ADR-007: Tokio for Async Runtime](../../.adrs/adr-007-thread-safety-strategy.md)

**Examples:**
```rust
// [PASS] Correct: Tokio multi-threaded runtime
#[tokio::main(flavor = "multi_threaded", worker_threads = 4)]
async fn main() -> Result<(), Error> {
    // Application code
    Ok(())
}

// [FAIL] Incorrect: Single-threaded runtime
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // Application code
    Ok(())
}

---

## 4. TYPESCRIPT STYLE GUIDELINES

### 4.1. TypeScript Configuration

**Standard:** All TypeScript code shall use strict mode with `noImplicitAny`, `strictNullChecks`, and `strictFunctionTypes` enabled.

**Rationale:** Strict mode enables the TypeScript compiler to catch more errors at compile time, improving type safety and reducing runtime errors.

**Reference:** [ADR-004: Leptos for Web Frontend](../../.adrs/adr-004-debounce-window.md)

**Configuration Example:**
```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true
  }
}
```

### 4.2. Type Annotations

#### 4.2.1. Function Parameter Annotations

**Standard:** All function parameters shall have explicit type annotations.

**Rationale:** Explicit parameter types improve code documentation and enable better IDE support.

**Examples:**
```typescript
// [PASS] Correct: Explicit type annotation for parameter
function calculateArea(width: number, height: number): number {
  return width * height;
}

// [FAIL] Incorrect: Missing type annotation
function calculateArea(width, height) {
  return width * height;
}

// [PASS] Correct: Arrow function with explicit types
const calculateArea = (width: number, height: number): number => {
  return width * height;
};
```

#### 4.2.2. Function Return Type Annotations

**Standard:** All functions shall have explicit return type annotations.

**Rationale:** Explicit return types document the function contract and enable the compiler to verify correctness.

**Examples:**
```typescript
// [PASS] Correct: Explicit return type annotation
function calculateArea(width: number, height: number): number {
  return width * height;
}

// [FAIL] Incorrect: Missing return type annotation
function calculateArea(width: number, height: number) {
  return width * height;
}

// [PASS] Correct: Arrow function with explicit return type
const calculateArea = (width: number, height: number): number => {
  return width * height;
};
```

#### 4.2.3. Variable Type Annotations

**Standard:** Variable type annotations shall be explicit when the type cannot be inferred or when clarity is needed.

**Rationale:** Explicit annotations improve readability and document intent when inference is unclear.

**Examples:**
```typescript
// [PASS] Correct: Explicit type for complex object
const config: {
  apiUrl: string;
  timeout: number;
  retries: number;
} = {
  apiUrl: 'https://api.example.com',
  timeout: 5000,
  retries: 3
};

// [PASS] Correct: Type inferred from literal
const count = 42;

// [PASS] Correct: Explicit type for readability
const userId: string = getUserSession().userId;
```

### 4.3. Interface and Type Definitions

#### 4.3.1. Interface Usage

**Standard:** Interfaces shall be used for object shapes that may be extended or implemented.

**Rationale:** Interfaces support declaration merging and are more extensible than type aliases.

**Examples:**
```typescript
// [PASS] Correct: Interface for extensible object shape
interface User {
  id: string;
  name: string;
  email: string;
}

// Extension through declaration merging
interface User {
  createdAt: Date;
}

// [PASS] Correct: Interface for class implementation
interface Serializable {
  serialize(): string;
}

class Document implements Serializable {
  serialize(): string {
    return JSON.stringify(this);
  }
}
```

#### 4.3.2. Type Alias Usage

**Standard:** Type aliases shall be used for union types, intersection types, and complex type expressions.

**Rationale:** Type aliases provide readable names for complex type expressions.

**Examples:**
```typescript
// [PASS] Correct: Type alias for union type
type Status = 'pending' | 'active' | 'completed' | 'failed';

// [PASS] Correct: Type alias for intersection type
type Timestamped<T> = T & {
  createdAt: Date;
  updatedAt: Date;
};

// [PASS] Correct: Type alias for complex type expression
type ApiResponse<T> = {
  data: T;
  status: number;
  message: string;
};
```

#### 4.3.3. Generic Type Parameters

**Standard:** Generic type parameters shall use descriptive single-letter names (T, U, V) or descriptive multi-letter names when clarity is needed.

**Rationale:** Descriptive generic parameter names improve readability and document intent.

**Examples:**
```typescript
// [PASS] Correct: Single-letter generic parameter
function identity<T>(value: T): T {
  return value;
}

// [PASS] Correct: Descriptive multi-letter generic parameter
function map<TInput, TOutput>(
  items: TInput[],
  mapper: (item: TInput) => TOutput
): TOutput[] {
  return items.map(mapper);
}

// [PASS] Correct: Constrained generic parameter
function length<T extends { length: number }>(value: T): number {
  return value.length;
}
```

### 4.4. Null and Undefined Handling

#### 4.4.1. Strict Null Checks

**Standard:** Strict null checks shall be enabled, and null/undefined shall be handled explicitly.

**Rationale:** Explicit null/undefined handling prevents runtime errors from null/undefined values.

**Examples:**
```typescript
// [PASS] Correct: Explicit null check
function getUserName(user: User | null): string {
  if (user === null) {
    return 'Anonymous';
  }
  return user.name;
}

// [PASS] Correct: Optional chaining with nullish coalescing
function getUserName(user: User | null): string {
  return user?.name ?? 'Anonymous';
}

// [FAIL] Incorrect: Implicit null check
function getUserName(user: User | null): string {
  return user.name;  // Runtime error if user is null
}
```

#### 4.4.2. Optional Chaining

**Standard:** Optional chaining (`?.`) shall be used for safe property access on potentially null/undefined values.

**Rationale:** Optional chaining provides concise syntax for safe property access.

**Examples:**
```typescript
// [PASS] Correct: Optional chaining
const city = user?.address?.city;

// [FAIL] Incorrect: Nested property access without null check
const city = user.address.city;  // Runtime error if user or address is null
```

#### 4.4.3. Nullish Coalescing

**Standard:** Nullish coalescing (`??`) shall be used for providing default values for null/undefined.

**Rationale:** Nullish coalescing distinguishes between null/undefined and falsy values.

**Examples:**
```typescript
// [PASS] Correct: Nullish coalescing
const timeout = config?.timeout ?? 5000;

// [FAIL] Incorrect: Logical OR operator
const timeout = config?.timeout || 5000;  // Uses default if timeout is 0
```

### 4.5. Async/Await

#### 4.5.1. Async Function Declaration

**Standard:** Async functions shall use the `async` keyword and return `Promise<T>`.

**Rationale:** The `async` keyword provides clear indication of asynchronous behavior.

**Examples:**
```typescript
// [PASS] Correct: Async function with Promise return type
async function fetchDocument(id: string): Promise<Document> {
  const response = await fetch(`/api/documents/${id}`);
  return response.json();
}

// [FAIL] Incorrect: Explicit Promise return type without async
function fetchDocument(id: string): Promise<Document> {
  return fetch(`/api/documents/${id}`).then(response => response.json());
}
```

#### 4.5.2. Await Usage

**Standard:** The `await` keyword shall be used for all Promise-based operations.

**Rationale:** The `await` keyword makes asynchronous code read like synchronous code, improving readability.

**Examples:**
```typescript
// [PASS] Correct: Explicit await for async operations
async function processDocuments(): Promise<void> {
  const docs = await fetchAllDocuments();
  for (const doc of docs) {
    await processDocument(doc);
  }
}

// [FAIL] Incorrect: Promise chaining
function processDocuments(): Promise<void> {
  return fetchAllDocuments().then(docs => {
    return Promise.all(docs.map(doc => processDocument(doc)));
  });
}
```

#### 4.5.3. Error Handling in Async Functions

**Standard:** Async functions shall use try/catch blocks for error handling.

**Rationale:** Try/catch blocks provide explicit error handling for async operations.

**Examples:**
```typescript
// [PASS] Correct: Try/catch for error handling
async function fetchDocument(id: string): Promise<Document | null> {
  try {
    const response = await fetch(`/api/documents/${id}`);
    return await response.json();
  } catch (error) {
    console.error(`Failed to fetch document ${id}:`, error);
    return null;
  }
}

// [FAIL] Incorrect: No error handling
async function fetchDocument(id: string): Promise<Document> {
  const response = await fetch(`/api/documents/${id}`);
  return response.json();  // Unhandled rejection if fetch fails
}
```

### 4.6. Leptos Framework Guidelines

#### 4.6.1. Component Structure

**Standard:** Leptos components shall use the `#[component]` macro and follow the component lifecycle pattern.

**Rationale:** The `#[component]` macro provides the standard Leptos component structure.

**Reference:** [ADR-004: Leptos for Web Frontend](../../.adrs/adr-004-debounce-window.md)

**Examples:**
```rust
// [PASS] Correct: Leptos component structure
#[component]
pub fn DocumentViewer(
    document_id: String,
    #[prop(optional)] show_metadata: bool,
) -> impl IntoView {
    let document = use_resource(move || async move {
        fetch_document(&document_id).await
    });

    view! {
        <div class="document-viewer">
            {match &*document.read() {
                Some(doc) => view! {
                    <h1>{&doc.title}</h1>
                    <div class="content">{&doc.content}</div>
                    {show_metadata.then(|| view! {
                        <div class="metadata">
                            <p>"Created: " {&doc.created_at}</p>
                        </div>
                    })}
                },
                None => view! {
                    <p>"Loading..."</p>
                },
            }}
        </div>
    }
}
```

#### 4.6.2. Signal Usage

**Standard:** Signals shall be used for reactive state management.

**Rationale:** Signals provide reactive state management that automatically updates the UI when changed.

**Examples:**
```rust
// [PASS] Correct: Signal for reactive state
#[component]
pub fn Counter() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div>
            <p>"Count: " {count}</p>
            <button on:click=move |_| set_count.update(|n| *n + 1)>"Increment"</button>
            <button on:click=move |_| set_count.update(|n| *n - 1)>"Decrement"</button>
        </div>
    }
}
```

#### 4.6.3. Resource Usage

**Standard:** Resources shall be used for async data fetching with loading and error states.

**Rationale:** Resources provide built-in loading and error state management for async operations.

**Examples:**
```rust
// [PASS] Correct: Resource for async data fetching
#[component]
pub fn UserProfile(user_id: String) -> impl IntoView {
    let user = use_resource(move || async move {
        fetch_user(&user_id).await
    });

    view! {
        <div class="user-profile">
            {match &*user.read() {
                Some(Ok(user)) => view! {
                    <h1>{&user.name}</h1>
                    <p>{&user.email}</p>
                },
                Some(Err(error)) => view! {
                    <p class="error">"Error: " {error.to_string()}</p>
                },
                None => view! {
                    <p>"Loading..."</p>
                },
            }}
        </div>
    }
}
```

---

## 5. NAMING CONVENTIONS

### 5.1. General Principles

**Standard:** Names shall be descriptive, pronounceable, and searchable. Abbreviations shall be avoided unless widely recognized.

**Rationale:** Descriptive names improve code readability and reduce cognitive load. Pronounceable names improve communication. Searchable names enable efficient code navigation.

**Examples:**
- [PASS] `calculate_area`, `user_id`, `document_title`
- [FAIL] `calc`, `uid`, `dt`

### 5.2. Rust Naming Conventions

#### 5.2.1. Struct and Enum Names

**Standard:** Struct and enum names shall use `PascalCase` (also known as `UpperCamelCase`).

**Rationale:** PascalCase is the standard convention for types in Rust, distinguishing types from values.

**Examples:**
```rust
// [PASS] Correct: PascalCase for struct
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
}

// [PASS] Correct: PascalCase for enum
pub enum DocumentStatus {
    Draft,
    Published,
    Archived,
}

// [FAIL] Incorrect: snake_case for struct
pub struct document {
    pub id: String,
}

// [FAIL] Incorrect: SCREAMING_SNAKE_CASE for enum
pub enum DOCUMENT_STATUS {
    DRAFT,
    PUBLISHED,
}
```

#### 5.2.2. Function and Method Names

**Standard:** Function and method names shall use `snake_case`.

**Rationale:** SnakeCase is the standard convention for functions in Rust, following Rust's naming conventions.

**Examples:**
```rust
// [PASS] Correct: snake_case for function
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// [PASS] Correct: snake_case for method
impl Document {
    pub fn get_title(&self) -> &str {
        &self.title
    }
}

// [FAIL] Incorrect: camelCase for function
pub fn calculateArea(width: f64, height: f64) -> f64 {
    width * height
}

// [FAIL] Incorrect: PascalCase for method
impl Document {
    pub fn GetTitle(&self) -> &str {
        &self.title
    }
}
```

#### 5.2.3. Variable and Field Names

**Standard:** Variable and struct field names shall use `snake_case`.

**Rationale:** SnakeCase is the standard convention for variables in Rust, following Rust's naming conventions.

**Examples:**
```rust
// [PASS] Correct: snake_case for variable
let user_id = get_user_id();

// [PASS] Correct: snake_case for struct field
pub struct User {
    pub user_id: String,
    pub user_name: String,
}

// [FAIL] Incorrect: camelCase for variable
let userId = get_user_id();

// [FAIL] Incorrect: SCREAMING_SNAKE_CASE for field
pub struct User {
    pub USER_ID: String,
}
```

#### 5.2.4. Constant Names

**Standard:** Constant names shall use `SCREAMING_SNAKE_CASE`.

**Rationale:** SCREAMING_SNAKE_CASE distinguishes constants from variables, following Rust's naming conventions.

**Examples:**
```rust
// [PASS] Correct: SCREAMING_SNAKE_CASE for constant
pub const MAX_DOCUMENT_SIZE: usize = 10_000_000;
pub const DEFAULT_TIMEOUT_MS: u64 = 5000;

// [FAIL] Incorrect: snake_case for constant
pub const max_document_size: usize = 10_000_000;

// [FAIL] Incorrect: PascalCase for constant
pub const MaxDocumentSize: usize = 10_000_000;
```

#### 5.2.5. Type Parameter Names

**Standard:** Type parameter names shall use single uppercase letters (`T`, `U`, `V`) or descriptive names when clarity is needed.

**Rationale:** Single uppercase letters are the standard convention for type parameters in Rust.

**Examples:**
```rust
// [PASS] Correct: Single uppercase letter for type parameter
pub fn identity<T>(value: T) -> T {
    value
}

// [PASS] Correct: Descriptive type parameter when clarity is needed
pub fn map<TInput, TOutput>(
    items: Vec<TInput>,
    mapper: fn(TInput) -> TOutput,
) -> Vec<TOutput> {
    items.into_iter().map(mapper).collect()
}

// [FAIL] Incorrect: lowercase for type parameter
pub fn identity<t>(value: t) -> t {
    value
}
```

#### 5.2.6. Module Names

**Standard:** Module names shall use `snake_case`.

**Rationale:** SnakeCase is the standard convention for modules in Rust, following Rust's naming conventions.

**Examples:**
```rust
// [PASS] Correct: snake_case for module
mod document_manager {
    pub fn create_document() -> Document {
        // Implementation
    }
}

// [FAIL] Incorrect: PascalCase for module
mod DocumentManager {
    pub fn CreateDocument() -> Document {
        // Implementation
    }
}
```

### 5.3. TypeScript Naming Conventions

#### 5.3.1. Interface and Type Alias Names

**Standard:** Interface and type alias names shall use `PascalCase`.

**Rationale:** PascalCase is the standard convention for types in TypeScript, distinguishing types from values.

**Examples:**
```typescript
// [PASS] Correct: PascalCase for interface
interface User {
  id: string;
  name: string;
  email: string;
}

// [PASS] Correct: PascalCase for type alias
type DocumentStatus = 'draft' | 'published' | 'archived';

// [FAIL] Incorrect: camelCase for interface
interface user {
  id: string;
}

// [FAIL] Incorrect: snake_case for type alias
type document_status = 'draft' | 'published' | 'archived';
```

#### 5.3.2. Function and Method Names

**Standard:** Function and method names shall use `camelCase`.

**Rationale:** camelCase is the standard convention for functions in JavaScript/TypeScript, following language conventions.

**Examples:**
```typescript
// [PASS] Correct: camelCase for function
function calculateArea(width: number, height: number): number {
  return width * height;
}

// [PASS] Correct: camelCase for method
class Document {
  getTitle(): string {
    return this.title;
  }
}

// [FAIL] Incorrect: snake_case for function
function calculate_area(width: number, height: number): number {
  return width * height;
}

// [FAIL] Incorrect: PascalCase for method
class Document {
  GetTitle(): string {
    return this.title;
  }
}
```

#### 5.3.3. Variable and Property Names

**Standard:** Variable and property names shall use `camelCase`.

**Rationale:** camelCase is the standard convention for variables in JavaScript/TypeScript, following language conventions.

**Examples:**
```typescript
// [PASS] Correct: camelCase for variable
const userId = getUserId();

// [PASS] Correct: camelCase for property
interface User {
  userId: string;
  userName: string;
}

// [FAIL] Incorrect: snake_case for variable
const user_id = getUserId();

// [FAIL] Incorrect: SCREAMING_SNAKE_CASE for property
interface User {
  USER_ID: string;
}
```

#### 5.3.4. Constant Names

**Standard:** Constant names shall use `SCREAMING_SNAKE_CASE`.

**Rationale:** SCREAMING_SNAKE_CASE distinguishes constants from variables, following JavaScript conventions.

**Examples:**
```typescript
// [PASS] Correct: SCREAMING_SNAKE_CASE for constant
const MAX_DOCUMENT_SIZE = 10_000_000;
const DEFAULT_TIMEOUT_MS = 5000;

// [FAIL] Incorrect: camelCase for constant
const maxDocumentSize = 10_000_000;

// [FAIL] Incorrect: PascalCase for constant
const MaxDocumentSize = 10_000_000;
```

#### 5.3.5. Class Names

**Standard:** Class names shall use `PascalCase`.

**Rationale:** PascalCase is the standard convention for classes in JavaScript/TypeScript, following language conventions.

**Examples:**
```typescript
// [PASS] Correct: PascalCase for class
class DocumentManager {
  createDocument(): Document {
    // Implementation
  }
}

// [FAIL] Incorrect: camelCase for class
class documentManager {
  createDocument(): Document {
    // Implementation
  }
}
```

#### 5.3.6. Generic Type Parameter Names

**Standard:** Generic type parameter names shall use single uppercase letters (`T`, `U`, `V`) or descriptive names when clarity is needed.

**Rationale:** Single uppercase letters are the standard convention for type parameters in TypeScript.

**Examples:**
```typescript
// [PASS] Correct: Single uppercase letter for type parameter
function identity<T>(value: T): T {
  return value;
}

// [PASS] Correct: Descriptive type parameter when clarity is needed
function map<TInput, TOutput>(
  items: TInput[],
  mapper: (item: TInput) => TOutput,
): TOutput[] {
  return items.map(mapper);
}

// [FAIL] Incorrect: lowercase for type parameter
function identity<t>(value: t): t {
  return value;
}
```

### 5.4. File and Directory Naming

#### 5.4.1. Rust File Names

**Standard:** Rust source file names shall use `snake_case`.

**Rationale:** SnakeCase matches Rust's module naming convention and is case-insensitive on some file systems.

**Examples:**
- [PASS] `document_manager.rs`, `user_service.rs`, `api_handler.rs`
- [FAIL] `DocumentManager.rs`, `UserService.rs`, `api-handler.rs`

#### 5.4.2. TypeScript File Names

**Standard:** TypeScript source file names shall use `camelCase` for components and `kebab-case` for utilities.

**Rationale:** camelCase matches component naming conventions. kebab-case is readable for utility files.

**Examples:**
- [PASS] `DocumentViewer.tsx`, `UserProfile.tsx`, `api-client.ts`
- [FAIL] `document-viewer.tsx`, `user_profile.tsx`, `ApiClient.ts`

#### 5.4.3. Directory Names

**Standard:** Directory names shall use `kebab-case`.

**Rationale:** kebab-case is readable and avoids case-sensitivity issues across different file systems.

**Examples:**
- [PASS] `document-manager/`, `user-service/`, `api-handler/`
- [FAIL] `document_manager/`, `UserService/`, `apiHandler/`
```

---

## 6. FORMATTING RULES

### 6.1. Indentation

#### 6.1.1. Rust Indentation

**Standard:** Rust code shall use 4 spaces for indentation. Tabs shall not be used.

**Rationale:** 4 spaces provide consistent indentation across different editors and environments. Tabs can cause inconsistent display.

**Examples:**
```rust
// [PASS] Correct: 4 spaces for indentation
pub fn calculate_area(width: f64, height: f64) -> f64 {
    if width > 0 && height > 0 {
        let area = width * height;
        area
    } else {
        0.0
    }
}

// [FAIL] Incorrect: Tabs for indentation
pub fn calculate_area(width: f64, height: f64) -> f64 {
	if width > 0 && height > 0 {
		let area = width * height;
		area
	} else {
		0.0
	}
}
```

#### 6.1.2. TypeScript Indentation

**Standard:** TypeScript code shall use 2 spaces for indentation. Tabs shall not be used.

**Rationale:** 2 spaces are the standard convention for JavaScript/TypeScript, following language conventions.

**Examples:**
```typescript
// [PASS] Correct: 2 spaces for indentation
function calculateArea(width: number, height: number): number {
  if (width > 0 && height > 0) {
    const area = width * height;
    return area;
  } else {
    return 0.0;
  }
}

// [FAIL] Incorrect: Tabs for indentation
function calculateArea(width: number, height: number): number {
	if (width > 0 && height > 0) {
		const area = width * height;
		return area;
	} else {
		return 0.0;
	}
}
```

### 6.2. Line Length

#### 6.2.1. Maximum Line Length

**Standard:** Lines shall not exceed 100 characters. Longer lines shall be split across multiple lines.

**Rationale:** Short lines improve readability and enable side-by-side code comparison.

**Examples:**
```rust
// [PASS] Correct: Line under 100 characters
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// [PASS] Correct: Long line split across multiple lines
pub fn calculate_area(width: f64, height: f64) -> f64 {
    let area = width * height;
    if area > 0.0 {
        area
    } else {
        0.0
    }
}

// [FAIL] Incorrect: Line exceeds 100 characters
pub fn calculate_area_of_rectangle_given_width_and_height(width: f64, height: f64) -> f64 {
    width * height
}
```

#### 6.2.2. Line Breaking Strategy

**Standard:** Lines shall be broken at logical boundaries (operators, commas, function calls).

**Rationale:** Breaking at logical boundaries improves readability and maintains code structure.

**Examples:**
```rust
// [PASS] Correct: Break at logical boundary
let result = function_name(
    long_parameter_name_1,
    long_parameter_name_2,
    long_parameter_name_3,
);

// [FAIL] Incorrect: Arbitrary line break
let result =
    function_name(long_parameter_name_1, long_parameter_name_2, long_parameter_name_3);
```

### 6.3. Blank Lines

#### 6.3.1. Function Separation

**Standard:** Functions shall be separated by one blank line.

**Rationale:** Blank lines improve readability by visually separating functions.

**Examples:**
```rust
// [PASS] Correct: One blank line between functions
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

pub fn calculate_perimeter(width: f64, height: f64) -> f64 {
    2.0 * (width + height)
}

// [FAIL] Incorrect: No blank line between functions
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}
pub fn calculate_perimeter(width: f64, height: f64) -> f64 {
    2.0 * (width + height)
}
```

#### 6.3.2. Logical Grouping

**Standard:** Blank lines shall be used to group related code.

**Rationale:** Logical grouping improves code organization and readability.

**Examples:**
```rust
// [PASS] Correct: Blank lines for logical grouping
pub fn process_document(document: &Document) -> Result<ProcessedDocument, Error> {
    // Validate document
    if document.content.is_empty() {
        return Err(Error::EmptyContent);
    }

    // Process content
    let processed_content = process_content(&document.content)?;

    // Create processed document
    Ok(ProcessedDocument {
        id: document.id.clone(),
        content: processed_content,
    })
}

// [FAIL] Incorrect: No logical grouping
pub fn process_document(document: &Document) -> Result<ProcessedDocument, Error> {
    if document.content.is_empty() {
        return Err(Error::EmptyContent);
    }
    let processed_content = process_content(&document.content)?;
    Ok(ProcessedDocument {
        id: document.id.clone(),
        content: processed_content,
    })
}
```

### 6.4. Bracing Style

#### 6.4.1. Rust Bracing Style

**Standard:** Rust code shall use K&R bracing style (opening brace on same line).

**Rationale:** K&R style is the standard convention for Rust, following Rust's formatting guidelines.

**Examples:**
```rust
// [PASS] Correct: K&R bracing style
pub fn calculate_area(width: f64, height: f64) -> f64 {
    if width > 0 && height > 0 {
        let area = width * height;
        area
    } else {
        0.0
    }
}

// [FAIL] Incorrect: Allman bracing style
pub fn calculate_area(width: f64, height: f64) -> f64
{
    if width > 0 && height > 0
    {
        let area = width * height;
        area
    }
    else
    {
        0.0
    }
}
```

#### 6.4.2. TypeScript Bracing Style

**Standard:** TypeScript code shall use K&R bracing style (opening brace on same line).

**Rationale:** K&R style is the standard convention for JavaScript/TypeScript, following language conventions.

**Examples:**
```typescript
// [PASS] Correct: K&R bracing style
function calculateArea(width: number, height: number): number {
  if (width > 0 && height > 0) {
    const area = width * height;
    return area;
  } else {
    return 0.0;
  }
}

// [FAIL] Incorrect: Allman bracing style
function calculateArea(width: number, height: number): number
{
  if (width > 0 && height > 0)
  {
    const area = width * height;
    return area;
  }
  else
  {
    return 0.0;
  }
}
```

### 6.5. Spacing

#### 6.5.1. Operator Spacing

**Standard:** Spaces shall be used around binary operators. No spaces shall be used around unary operators.

**Rationale:** Spacing around operators improves readability and distinguishes binary from unary operators.

**Examples:**
```rust
// [PASS] Correct: Spaces around binary operators
let result = a + b * c;
let is_valid = x > 0 && x < 100;

// [PASS] Correct: No spaces around unary operators
let negated = -value;
let incremented = value++;

// [FAIL] Incorrect: No spaces around binary operators
let result = a+b*c;

// [FAIL] Incorrect: Spaces around unary operators
let negated = - value;
```

#### 6.5.2. Comma Spacing

**Standard:** Spaces shall be used after commas in lists and function calls.

**Rationale:** Spacing after commas improves readability and follows language conventions.

**Examples:**
```rust
// [PASS] Correct: Space after comma
let result = function_name(arg1, arg2, arg3);

// [FAIL] Incorrect: No space after comma
let result = function_name(arg1,arg2,arg3);
```

#### 6.5.3. Parenthesis Spacing

**Standard:** No spaces shall be used inside parentheses.

**Rationale:** No spaces inside parentheses improves readability and follows language conventions.

**Examples:**
```rust
// [PASS] Correct: No spaces inside parentheses
let result = function_name(arg1, arg2);

// [FAIL] Incorrect: Spaces inside parentheses
let result = function_name( arg1, arg2 );
```

### 6.6. Trailing Whitespace

**Standard:** Trailing whitespace shall not be present in any file.

**Rationale:** Trailing whitespace causes unnecessary diff noise and can cause issues with some tools.

**Examples:**
```rust
// [PASS] Correct: No trailing whitespace
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// [FAIL] Incorrect: Trailing whitespace (shown as ·)
pub fn calculate_area(width: f64, height: f64) -> f64 {··
    width * height··
}
```

---

## 7. DOCUMENTATION STYLE

### 7.1. Rust Documentation

#### 7.1.1. Documentation Comments

**Standard:** Public functions, structs, enums, and traits shall be documented using `///` doc comments.

**Rationale:** Documentation comments enable automatic documentation generation with `cargo doc`.

**Examples:**
```rust
// [PASS] Correct: Documentation comment for function
/// Calculates the area of a rectangle.
///
/// # Arguments
///
/// * `width` - The width of the rectangle
/// * `height` - The height of the rectangle
///
/// # Returns
///
/// The area of the rectangle in square units.
///
/// # Examples
///
/// ```
/// let area = calculate_area(10.0, 20.0);
/// assert_eq!(area, 200.0);
/// ```
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// [FAIL] Incorrect: Regular comment instead of documentation comment
// Calculates the area of a rectangle
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}
```

#### 7.1.2. Module Documentation

**Standard:** Modules shall be documented using `//!` module-level doc comments.

**Rationale:** Module documentation provides context for all items within the module.

**Examples:**
```rust
// [PASS] Correct: Module-level documentation
//! Document management module.
//!
//! This module provides functionality for creating, reading, updating,
//! and deleting documents in the Tachyon system.

pub mod document_manager {
    // Module implementation
}

// [FAIL] Incorrect: No module documentation
pub mod document_manager {
    // Module implementation
}
```

#### 7.1.3. Documentation Sections

**Standard:** Documentation shall include sections for Arguments, Returns, Errors, and Examples when applicable.

**Rationale:** Structured documentation sections improve readability and completeness.

**Examples:**
```rust
// [PASS] Correct: Complete documentation with sections
/// Fetches a document from the database.
///
/// # Arguments
///
/// * `id` - The unique identifier of the document to fetch
///
/// # Returns
///
/// Returns `Ok(Document)` if the document exists, `Err(DatabaseError)` otherwise.
///
/// # Errors
///
/// This function will return an error if:
/// - The database connection fails
/// - The document does not exist
/// - The document data is corrupted
///
/// # Examples
///
/// ```
/// let document = fetch_document("doc-123").await?;
/// println!("Document title: {}", document.title);
/// ```
pub async fn fetch_document(id: &str) -> Result<Document, DatabaseError> {
    // Implementation
}

// [FAIL] Incorrect: Incomplete documentation
/// Fetches a document from the database.
pub async fn fetch_document(id: &str) -> Result<Document, DatabaseError> {
    // Implementation
}
```

### 7.2. TypeScript Documentation

#### 7.2.1. JSDoc Comments

**Standard:** Public functions, classes, and interfaces shall be documented using JSDoc comments (`/** */`).

**Rationale:** JSDoc comments enable automatic documentation generation and provide IDE support.

**Examples:**
```typescript
// [PASS] Correct: JSDoc comment for function
/**
 * Calculates the area of a rectangle.
 *
 * @param width - The width of the rectangle
 * @param height - The height of the rectangle
 * @returns The area of the rectangle in square units
 *
 * @example
 * ```typescript
 * const area = calculateArea(10, 20);
 * console.log(area); // 200
 * ```
 */
function calculateArea(width: number, height: number): number {
  return width * height;
}

// [FAIL] Incorrect: Regular comment instead of JSDoc comment
// Calculates the area of a rectangle
function calculateArea(width: number, height: number): number {
  return width * height;
}
```

#### 7.2.2. Interface Documentation

**Standard:** Interfaces shall be documented with descriptions of their purpose and usage.

**Rationale:** Interface documentation explains the contract and expected behavior.

**Examples:**
```typescript
// [PASS] Correct: Complete interface documentation
/**
 * Represents a user in the Tachyon system.
 *
 * A user has a unique identifier, name, and email address.
 * Users can create, read, update, and delete documents.
 *
 * @interface
 */
interface User {
  /** The unique identifier of the user */
  id: string;
  
  /** The display name of the user */
  name: string;
  
  /** The email address of the user */
  email: string;
}

// [FAIL] Incorrect: Incomplete interface documentation
interface User {
  id: string;
  name: string;
  email: string;
}
```

#### 7.2.3. Class Documentation

**Standard:** Classes shall be documented with descriptions of their purpose and usage.

**Rationale:** Class documentation explains the class's responsibility and how to use it.

**Examples:**
```typescript
// [PASS] Correct: Complete class documentation
/**
 * Manages documents in the Tachyon system.
 *
 * The DocumentManager provides methods for creating, reading, updating,
 * and deleting documents. It handles database operations and caching.
 *
 * @example
 * ```typescript
 * const manager = new DocumentManager();
 * const doc = await manager.createDocument({ title: "My Document" });
 * ```
 */
class DocumentManager {
  // Class implementation
}

// [FAIL] Incorrect: No class documentation
class DocumentManager {
  // Class implementation
}
```

### 7.3. Inline Comments

#### 7.3.1. Comment Purpose

**Standard:** Inline comments shall explain "why" rather than "what".

**Rationale:** Code already explains "what". Comments should explain the rationale behind non-obvious decisions.

**Examples:**
```rust
// [PASS] Correct: Comment explains "why"
// Use exponential backoff to avoid overwhelming the server during retries
let retry_delay = calculate_backoff(attempt);

// [FAIL] Incorrect: Comment explains "what" (redundant)
// Calculate the retry delay
let retry_delay = calculate_backoff(attempt);
```

#### 7.3.2. Comment Placement

**Standard:** Comments shall be placed above the code they describe.

**Rationale:** Comments above code are more readable and follow standard conventions.

**Examples:**
```rust
// [PASS] Correct: Comment above code
// Validate input before processing
if input.is_empty() {
    return Err(Error::InvalidInput);
}

// [FAIL] Incorrect: Comment after code
if input.is_empty() {
    return Err(Error::InvalidInput);
} // Validate input before processing
```

#### 7.3.3. TODO Comments

**Standard:** TODO comments shall include a reference to an issue or task.

**Rationale:** References enable tracking and resolution of TODO items.

**Examples:**
```rust
// [PASS] Correct: TODO with issue reference
// TODO: Implement caching for frequently accessed documents
// Reference: https://github.com/tachyon/tachyon/issues/123

// [FAIL] Incorrect: TODO without reference
// TODO: Implement caching
```

### 7.4. Documentation Quality

#### 7.4.1. Completeness

**Standard:** Documentation shall be complete and cover all public interfaces.

**Rationale:** Complete documentation ensures users understand how to use the code.

**Examples:**
```rust
// [PASS] Correct: All public functions documented
/// Calculates the area of a rectangle.
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

/// Calculates the perimeter of a rectangle.
pub fn calculate_perimeter(width: f64, height: f64) -> f64 {
    2.0 * (width + height)
}

// [FAIL] Incorrect: Incomplete documentation
/// Calculates the area of a rectangle.
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

pub fn calculate_perimeter(width: f64, height: f64) -> f64 {
    2.0 * (width + height)
}
```

#### 7.4.2. Accuracy

**Standard:** Documentation shall be accurate and kept in sync with code changes.

**Rationale:** Inaccurate documentation misleads users and causes confusion.

**Examples:**
```rust
// [PASS] Correct: Documentation matches implementation
/// Calculates the area of a rectangle.
///
/// # Arguments
///
/// * `width` - The width of the rectangle
/// * `height` - The height of the rectangle
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// [FAIL] Incorrect: Documentation does not match implementation
/// Calculates the perimeter of a rectangle.
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}
```

#### 7.4.3. Examples

**Standard:** Documentation shall include executable examples when applicable.

**Rationale:** Examples demonstrate usage and serve as tests with `cargo test --doc`.

**Examples:**
```rust
// [PASS] Correct: Documentation includes example
/// Calculates the area of a rectangle.
///
/// # Examples
///
/// ```
/// let area = calculate_area(10.0, 20.0);
/// assert_eq!(area, 200.0);
/// ```
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// [FAIL] Incorrect: No example in documentation
/// Calculates the area of a rectangle.
pub fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}
```

---

## 8. ERROR HANDLING STYLE

### 8.1. Rust Error Handling

#### 8.1.1. Result Type Usage

**Standard:** Functions that can fail shall return `Result<T, E>` rather than panicking.

**Rationale:** The `Result` type makes error handling explicit and forces callers to handle errors.

**Reference:** [ADR-010: Security Architecture](../../.adrs/adr-010-synchronization-primitives.md) - Fail-Safe Error Handling

**Examples:**
\`\`\`rust
// [PASS] Correct: Return Result for fallible operation
pub fn parse_number(s: &str) -> Result<i32, ParseIntError> {
    s.parse::<i32>()
}

// [FAIL] Incorrect: Panic on error
pub fn parse_number(s: &str) -> i32 {
    s.parse::<i32>().unwrap()
}
\`\`\`

#### 8.1.2. Error Type Design

**Standard:** Error types shall use `thiserror` for application errors and `anyhow` for library errors.

**Rationale:** `thiserror` provides ergonomic error handling with custom error types. `anyhow` provides flexible error handling for libraries.

**Examples:**
\`\`\`rust
// [PASS] Correct: Custom error type with thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Document not found: {0}")]
    NotFound(String),
    #[error("Invalid document format: {0}")]
    InvalidFormat(String),
    #[error("Permission denied")]
    PermissionDenied,
}

// [FAIL] Incorrect: Using String for errors
pub fn fetch_document(id: &str) -> Result<Document, String> {
    // Implementation
}
\`\`\`

#### 8.1.3. Error Propagation

**Standard:** Errors shall be propagated using the `?` operator when appropriate.

**Rationale:** The `?` operator provides concise error propagation while maintaining explicit error handling.

**Examples:**
\`\`\`rust
// [PASS] Correct: Use ? operator for error propagation
pub fn read_config(path: &str) -> Result<Config, IoError> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

// [FAIL] Incorrect: Manual error propagation
pub fn read_config(path: &str) -> Result<Config, IoError> {
    let content = fs::read_to_string(path).map_err(|e| IoError::Read(e))?;
    let config: Config = serde_json::from_str(&content).map_err(|e| IoError::Parse(e))?;
    Ok(config)
}
\`\`\`

#### 8.1.4. Error Context

**Standard:** Errors shall include context about the operation that failed.

**Rationale:** Error context improves debuggability and helps identify the source of errors.

**Examples:**
\`\`\`rust
// [PASS] Correct: Error with context
pub fn fetch_document(id: &str) -> Result<Document, DocumentError> {
    let path = format!("/documents/{}.json", id);
    let content = fs::read_to_string(&path)
        .map_err(|e| DocumentError::NotFound(format!("Failed to read {}: {}", path, e)))?;
    serde_json::from_str(&content)
        .map_err(|e| DocumentError::InvalidFormat(format!("Failed to parse {}: {}", path, e)))
}

// [FAIL] Incorrect: Error without context
pub fn fetch_document(id: &str) -> Result<Document, DocumentError> {
    let content = fs::read_to_string(&path)?;
    serde_json::from_str(&content)?
}
\`\`\`

#### 8.1.5. Panic Usage

**Standard:** Panics shall only be used for unrecoverable errors or programming errors.

**Rationale:** Panics terminate the program and should only be used when recovery is impossible.

**Examples:**
\`\`\`rust
// [PASS] Correct: Panic for unrecoverable error
pub fn get_document(id: &str) -> Document {
    DOCUMENTS.get(id)
        .expect("Document cache not initialized")
        .get(id)
        .expect(&format!("Document {} not found", id))
        .clone()
}

// [FAIL] Incorrect: Panic for recoverable error
pub fn fetch_document(id: &str) -> Document {
    let content = fs::read_to_string(&path).expect("Failed to read document");
    serde_json::from_str(&content).expect("Failed to parse document")
}
\`\`\`

### 8.2. TypeScript Error Handling

#### 8.2.1. Error Types

**Standard:** Errors shall be represented using custom error classes or discriminated unions.

**Rationale:** Custom error types provide type safety and enable pattern matching on errors.

**Examples:**
\`\`\`typescript
// [PASS] Correct: Custom error class
class DocumentError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly context?: unknown
  ) {
    super(message);
    this.name = 'DocumentError';
  }
}

// [FAIL] Incorrect: Using string for errors
function fetchDocument(id: string): Promise<Document> {
  throw new Error('Failed to fetch document');
}
\`\`\`

#### 8.2.2. Error Propagation

**Standard:** Errors shall be propagated using `throw` and `try/catch` blocks.

**Rationale:** Explicit error propagation ensures errors are not silently ignored.

**Examples:**
\`\`\`typescript
// [PASS] Correct: Explicit error propagation
async function readConfig(path: string): Promise<Config> {
  try {
    const content = await fs.readFile(path, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    throw new ConfigError('Failed to read config', { path, error });
  }
}

// [FAIL] Incorrect: Silent error handling
async function readConfig(path: string): Promise<Config> {
  try {
    const content = await fs.readFile(path, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    return null as any;
  }
}
\`\`\`

#### 8.2.3. Error Context

**Standard:** Errors shall include context about the operation that failed.

**Rationale:** Error context improves debuggability and helps identify the source of errors.

**Examples:**
\`\`\`typescript
// [PASS] Correct: Error with context
async function fetchDocument(id: string): Promise<Document> {
  try {
    const path = \`/documents/\${id}.json\`;
    const content = await fs.readFile(path, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    throw new DocumentError('Failed to fetch document', 'FETCH_ERROR', {
      id,
      path: \`/documents/\${id}.json\`,
      error,
    });
  }
}

// [FAIL] Incorrect: Error without context
async function fetchDocument(id: string): Promise<Document> {
  try {
    const content = await fs.readFile(path, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    throw new Error('Failed to fetch document');
  }
}
\`\`\`

#### 8.2.4. Never Type Usage

**Standard:** The `never` type shall be used for unreachable code paths after throwing errors.

**Rationale:** The `never` type ensures type safety for unreachable code paths.

**Examples:**
\`\`\`typescript
// [PASS] Correct: Never type for unreachable code
function handleError(error: Error): never {
  console.error('Error:', error);
  throw error;
}

// [FAIL] Incorrect: No type annotation for unreachable code
function handleError(error: Error): void {
  console.error('Error:', error);
  throw error;
}
\`\`\`

### 8.3. Error Handling Best Practices

#### 8.3.1. Early Return

**Standard:** Errors shall be handled as early as possible using early return patterns.

**Rationale:** Early return reduces nesting and improves code readability.

**Examples:**
\`\`\`rust
// [PASS] Correct: Early return for error handling
pub fn process_document(document: &Document) -> Result<ProcessedDocument, Error> {
    if document.content.is_empty() {
        return Err(Error::EmptyContent);
    }
    
    if document.content.len() > MAX_SIZE {
        return Err(Error::ContentTooLarge);
    }
    
    let processed = process_content(&document.content)?;
    Ok(ProcessedDocument::new(document.id.clone(), processed))
}

// [FAIL] Incorrect: Deep nesting for error handling
pub fn process_document(document: &Document) -> Result<ProcessedDocument, Error> {
    if !document.content.is_empty() {
        if document.content.len() <= MAX_SIZE {
            let processed = process_content(&document.content)?;
            Ok(ProcessedDocument::new(document.id.clone(), processed))
        } else {
            Err(Error::ContentTooLarge)
        }
    } else {
        Err(Error::EmptyContent)
    }
}
\`\`\`

#### 8.3.2. Error Logging

**Standard:** Errors shall be logged before being returned or thrown.

**Rationale:** Error logging provides visibility into failures and aids debugging.

**Examples:**
\`\`\`rust
// [PASS] Correct: Error logging before return
pub fn fetch_document(id: &str) -> Result<Document, DocumentError> {
    let path = format!("/documents/{}.json", id);
    
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read document {}: {}", path, e);
            return Err(DocumentError::NotFound(id.to_string()));
        }
    };
    
    serde_json::from_str(&content).map_err(|e| {
        error!("Failed to parse document {}: {}", path, e);
        DocumentError::InvalidFormat(e.to_string())
    })
}

// [FAIL] Incorrect: No error logging
pub fn fetch_document(id: &str) -> Result<Document, DocumentError> {
    let content = fs::read_to_string(&path)?;
    serde_json::from_str(&content)?
}
\`\`\`

#### 8.3.3. User-Facing Error Messages

**Standard:** User-facing error messages shall be clear, actionable, and avoid technical jargon.

**Rationale:** Clear error messages help users understand and resolve issues.

**Examples:**
\`\`\`rust
// [PASS] Correct: User-friendly error message
pub fn validate_document_title(title: &str) -> Result<(), ValidationError> {
    if title.is_empty() {
        return Err(ValidationError::new(
            "Document title cannot be empty"
        ));
    }
    
    if title.len() > 100 {
        return Err(ValidationError::new(
            "Document title must be less than 100 characters"
        ));
    }
    
    Ok(())
}

// [FAIL] Incorrect: Technical error message
pub fn validate_document_title(title: &str) -> Result<(), ValidationError> {
    if title.is_empty() {
        return Err(ValidationError::new(
            "Title validation failed: length is 0"
        ));
    }
    
    if title.len() > 100 {
        return Err(ValidationError::new(
            "Title validation failed: length exceeds maximum allowed value"
        ));
    }
    
    Ok(())
}
\`\`\`

---

## 9. REFERENCES

### 9.1. Internal References

#### 9.1.1. Standards Documents

- [TACHYON-STD-V1.0](../../.adrs/ - Coding and Documentation Standards

#### 9.1.2. Architecture Decision Records

- [ADR-001: Rust as Primary Language](../../.adrs/adr-001-three-tier-jit-compilation.md) - Rust language selection and rationale
- [ADR-002: Tauri for Desktop Application](../../.adrs/adr-002-bm25-search-parameters.md) - Desktop application framework selection
- [ADR-003: Axum for HTTP/2 Server](../../.adrs/adr-003-lru-cache-target.md) - HTTP/2 server framework selection
- [ADR-004: Leptos for Web Frontend](../../.adrs/adr-004-debounce-window.md) - Web frontend framework selection
- [ADR-005: Bun for JavaScript Runtime](../../.adrs/adr-005-last-write-wins-conflict-resolution.md) - JavaScript runtime selection
- [ADR-006: Nix Flakes for Build System](../../.adrs/adr-006-direct-libgit2-integration.md) - Build system selection
- [ADR-007: Tokio for Async Runtime](../../.adrs/adr-007-thread-safety-strategy.md) - Async runtime selection
- [ADR-008: Workspace Structure for Rust Crates](../../.adrs/adr-008-deadlock-prevention.md) - Rust workspace organization
- [ADR-009: IPC Communication Architecture](../../.adrs/adr-009-race-condition-mitigation.md) - Inter-process communication design
- [ADR-010: Security Architecture](../../.adrs/adr-010-synchronization-primitives.md) - Security architecture and controls

#### 9.1.3. Requirements Documents

- [REQ-DOC-019: Code Style Guide](../../.adrs/ - Documentation requirements

### 9.2. External References

#### 9.2.1. Rust References

[1] The Rust Programming Language, "The Rust Reference," Online. Available: https://doc.rust-lang.org/reference/. [Accessed: 01-Feb-2026].

[2] The Rust Project, "Rust Edition 2024," Online. Available: https://doc.rust-lang.org/edition-guide/rust-2024/index.html. [Accessed: 01-Feb-2026].

[3] The Rust Project, "The Rustonomicon: The Unsafe Book," Online. Available: https://doc.rust-lang.org/nomicon/. [Accessed: 01-Feb-2026].

[4] The Rust Project, "The Rust Book," Online. Available: https://doc.rust-lang.org/book/. [Accessed: 01-Feb-2026].

[5] The Rust Project, "Rust API Guidelines," Online. Available: https://rust-lang.github.io/api-guidelines/. [Accessed: 01-Feb-2026].

[6] The Rust Project, "Rust Style Guide," Online. Available: https://rust-lang.github.io/rustfmt/. [Accessed: 01-Feb-2026].

[7] The Rust Project, "Clippy Lints," Online. Available: https://rust-lang.github.io/rust-clippy/. [Accessed: 01-Feb-2026].

[8] Tokio Contributors, "Tokio: Asynchronous runtime for the Rust programming language," Online. Available: https://tokio.rs/. [Accessed: 01-Feb-2026].

[9] crates.io, "Rust Package Registry," Online. Available: https://crates.io/. [Accessed: 01-Feb-2026].

#### 9.2.2. TypeScript References

[10] Microsoft, "TypeScript Handbook," Online. Available: https://www.typescriptlang.org/docs/handbook/intro.html. [Accessed: 01-Feb-2026].

[11] Microsoft, "TypeScript Deep Dive," Online. Available: https://basarat.gitbook.io/typescript-deep-dive/. [Accessed: 01-Feb-2026].

[12] Microsoft, "TypeScript Style Guide," Online. Available: https://typescript-eslint.io/rules/. [Accessed: 01-Feb-2026].

[13] ESLint, "ESLint - The pluggable linting utility for JavaScript and JSX," Online. Available: https://eslint.org/. [Accessed: 01-Feb-2026].

[14] Prettier, "Prettier - Code formatter using prettier," Online. Available: https://prettier.io/. [Accessed: 01-Feb-2026].

[15] Leptos, "Leptos - Build fast web applications with Rust," Online. Available: https://leptos.rs/. [Accessed: 01-Feb-2026].

#### 9.2.3. Standards References

[16] ISO/IEC 26514:2021, "Systems and software engineering — Requirements for designers and developers of user documentation," International Organization for Standardization, 2021.

[17] ISO/IEC 12207:2017, "Systems and software engineering — Software life cycle processes," International Organization for Standardization, 2017.

[18] ISO/IEC 25010:2011, "Systems and software engineering — Systems and software Quality Requirements and Evaluation (SQuaRE) — System and software quality models," International Organization for Standardization, 2011.

[19] IEEE 829-2008, "IEEE Standard for Software Test Documentation," IEEE Computer Society, 2008.

[20] IEEE 1063-2001, "IEEE Standard for Software User Documentation," IEEE Computer Society, 2001.

[21] IEEE 1016-2009, "IEEE Standard for Information Technology—System Design—Software Design Descriptions," IEEE Computer Society, 2009.

### 9.3. Tooling References

#### 9.3.1. Rust Tooling

[22] The Rust Project, "rustfmt - Format Rust code," Online. Available: https://github.com/rust-lang/rustfmt. [Accessed: 01-Feb-2026].

[23] The Rust Project, "Clippy - A bunch of lints to catch common mistakes and improve your Rust code," Online. Available: https://github.com/rust-lang/rust-clippy. [Accessed: 01-Feb-2026].

[24] The Rust Analyzer Project, "rust-analyzer - A Rust compiler front-end for IDEs," Online. Available: https://github.com/rust-analyzer/rust-analyzer. [Accessed: 01-Feb-2026].

[25] The Rust Project, "Cargo - The Rust package manager," Online. Available: https://doc.rust-lang.org/cargo/. [Accessed: 01-Feb-2026].

#### 9.3.2. TypeScript Tooling

[26] Microsoft, "TypeScript Compiler (tsc)," Online. Available: https://www.typescriptlang.org/docs/handbook/compiler-options.html. [Accessed: 01-Feb-2026].

[27] ESLint, "ESLint - Find and fix problems in your JavaScript code," Online. Available: https://eslint.org/. [Accessed: 01-Feb-2026].

[28] Prettier, "Prettier - Opinionated Code Formatter," Online. Available: https://prettier.io/. [Accessed: 01-Feb-2026].

[29] Bun, "Bun - Incredibly fast JavaScript runtime, bundler, test runner, and package manager," Online. Available: https://bun.sh/. [Accessed: 01-Feb-2026].

---

**Document Control**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| V1.0 | February 2026 | Technical Writer | Initial creation |

---

**Document Status:** Approved for Implementation

**Document Classification:** Developer Documentation

**Compliance Level:** ISO/IEC 26514:2021, IEEE 1063:2001
