# How to Use This Template

This template provides a minimal starting point for new microservices in RapidFab.xyz.

## Quick Start (For Agents)

1. **Copy the template:**
   ```bash
   cp -r services/_template services/your-service-name
   cd services/your-service-name
   ```

2. **Find and replace these placeholders:**
   - `SERVICE_NAME` → your service binary name (e.g., `pricing-service`)
   - `template-service` → your service name in package format
   - All `TODO:` comments

3. **Update files:**
   - `Cargo.toml`: name, dependencies
   - `Containerfile`: binary names (3 places)
   - `Makefile`: SERVICE_NAME variable
   - `src/main.rs`: service name in health response
   - `README.md`: service description, endpoints, env vars
   - `docs/INDEX.md`: architecture, API contracts, configuration

4. **Generate Cargo.lock:**
   ```bash
   cargo build
   ```

5. **Verify setup:**
   ```bash
   make lint
   make test
   make docker-build
   ```

## What's Included

### Core Files
- `Cargo.toml` - Minimal Rust dependencies (axum, tokio, serde, tracing)
- `src/main.rs` - Basic Axum server with /health endpoint
- `Containerfile` - Multi-stage Docker build with layer caching
- `Makefile` - Standard targets (fmt, lint, test, build, run, docker-*)

### Documentation
- `README.md` - Quick start guide
- `docs/INDEX.md` - Architecture and API contracts
- `TEMPLATE_USAGE.md` - This file (delete after setup)

### Structure
- `tests/` - Integration tests directory (empty)
- `src/` - Source code

## Customization Examples

### Adding a new endpoint
```rust
// In src/main.rs
async fn my_handler() -> Json<MyResponse> {
    // ...
}

let app = Router::new()
    .route("/health", get(health))
    .route("/api/my-endpoint", post(my_handler));  // Add this
```

### Adding database support
```toml
# In Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
```

### Adding environment variables
```rust
// In src/main.rs
let database_url = std::env::var("DATABASE_URL")?;
```

## File Size Target

Keep total lines minimal:
- `src/main.rs`: < 100 lines for simple services
- `Cargo.toml`: < 50 lines
- `README.md`: < 80 lines
- `docs/INDEX.md`: < 150 lines

## Next Steps

1. Delete `TEMPLATE_USAGE.md` (this file) after setup
2. Implement your service logic in `src/main.rs`
3. Add integration tests in `tests/`
4. Document endpoints in `README.md` and `docs/INDEX.md`
5. Run `task ci` from project root to verify everything works

---

**Template Version:** 1.0
