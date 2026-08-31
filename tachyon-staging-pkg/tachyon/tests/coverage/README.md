# Test Coverage Configuration

This directory contains configuration for test coverage reporting.

## Setup

1. Install coverage tools:
```bash
cargo install cargo-tarpaulin
cargo install cargo-llvm-cov
```

2. Run coverage:
```bash
# Using tarpaulin (recommended for CI)
cargo tarpaulin --out Html --out Xml --out Lcov

# Using llvm-cov (faster, more accurate)
cargo llvm-cov --html --xml --lcov --output-path coverage
```

## Coverage Targets

| Module | Target Coverage |
|--------|----------------|
| tachyon-core | 70% |
| tachyon-database | 65% |
| tachyon-server | 60% |
| tachyon-rbac | 65% |
| tachyon-search | 60% |
| **Overall** | **60%** |

## Viewing Reports

After running coverage, open the generated HTML report:

```bash
open tarpaulin-report.html
# or
open coverage/index.html
```

## CI Integration

Coverage reports are automatically generated in CI and uploaded to Codecov:
https://codecov.io/gh/anomaly/tachyon

## Exclusions

The following are excluded from coverage:
- Test code (`tests/*`, `benches/*`)
- Error types (`**/error.rs`)
- Main entry points (`**/main.rs`)
- Auto-generated code

## Best Practices

1. **Write tests for new code**: All new code should have tests
2. **Maintain coverage**: Don't let coverage drop below 60%
3. **Test edge cases**: Include error paths and edge cases
4. **Use property testing**: For complex logic, use `proptest`
5. **Mock external dependencies**: Use `mockall` for external services

## Running Specific Coverage

```bash
# Coverage for specific package
cargo tarpaulin -p tachyon-database

# Coverage for specific test
cargo tarpaulin --test document_test

# Coverage with specific features
cargo tarpaulin --all-features
```

## Troubleshooting

### Low Coverage Warnings

If coverage is below threshold:
1. Check the HTML report for uncovered lines
2. Write tests for uncovered branches
3. Focus on critical business logic

### Coverage Tool Issues

If tarpaulin fails:
1. Try `cargo llvm-cov` instead
2. Ensure all dependencies are compiled
3. Check for race conditions in tests
