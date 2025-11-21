# SunBay SoftPOS Backend Tests

This directory contains the test suite for the SunBay SoftPOS backend application.

## Test Structure

```
tests/
├── unit/                          # Unit tests
│   ├── models/                    # Model tests
│   │   └── device_test.rs        # Device model tests
│   └── security/                  # Security module tests
│       └── dukpt_test.rs         # DUKPT encryption tests
├── integration/                   # Integration tests
│   ├── api/                       # API endpoint tests
│   │   └── device_api_test.rs    # Device API tests
│   └── services/                  # Service layer tests
│       └── transaction_service_test.rs  # Transaction service tests
└── lib.rs                         # Test library root
```

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Unit Tests Only
```bash
cargo test --test unit
```

### Run Integration Tests Only
```bash
cargo test --test integration
```

### Run Specific Test File
```bash
cargo test --test device_test
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests in Parallel
```bash
cargo test -- --test-threads=4
```

## Test Database Setup

Integration tests require a test database. Set up the test database:

1. Create test database:
```bash
createdb softpos_test
```

2. Run migrations:
```bash
DATABASE_URL=postgres://postgres:postgres@localhost/softpos_test sqlx migrate run
```

3. Set environment variable:
```bash
export TEST_DATABASE_URL=postgres://postgres:postgres@localhost/softpos_test
```

## Test Categories

### Unit Tests
- **Models**: Test data structures, validation, and business logic
- **Security**: Test encryption, DUKPT, JWT, and crypto functions
- **Utils**: Test utility functions and helpers

### Integration Tests
- **API**: Test HTTP endpoints, request/response handling
- **Services**: Test service layer with database interactions
- **Repositories**: Test database operations

## Writing New Tests

### Unit Test Template
```rust
#[cfg(test)]
mod my_module_tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        let input = "test";
        
        // Act
        let result = my_function(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Integration Test Template
```rust
#[cfg(test)]
mod my_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_api_endpoint() {
        // Setup
        let pool = setup_test_db().await;
        let app = setup_test_app(pool).await;
        
        // Execute
        let req = test::TestRequest::get()
            .uri("/api/endpoint")
            .to_request();
        let resp = test::call_service(&app, req).await;
        
        // Verify
        assert!(resp.status().is_success());
    }
}
```

## Test Coverage

Generate test coverage report:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage
```

## Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Clean up test data after tests
3. **Naming**: Use descriptive test names (test_feature_scenario_expected)
4. **Arrange-Act-Assert**: Follow AAA pattern
5. **Edge Cases**: Test boundary conditions and error cases
6. **Performance**: Keep tests fast and focused

## Continuous Integration

Tests are automatically run on:
- Every push to main branch
- Every pull request
- Scheduled nightly builds

See `.github/workflows/ci.yml` for CI configuration.

## Troubleshooting

### Database Connection Issues
```bash
# Check PostgreSQL is running
pg_isready

# Verify connection string
psql $TEST_DATABASE_URL
```

### Test Failures
```bash
# Run with verbose output
cargo test -- --nocapture --test-threads=1

# Run specific failing test
cargo test test_name -- --exact
```

### Clean Test Database
```bash
# Drop and recreate
dropdb softpos_test
createdb softpos_test
DATABASE_URL=$TEST_DATABASE_URL sqlx migrate run
```
