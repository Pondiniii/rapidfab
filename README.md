# RapidFab.xyz

Minimalist on-demand manufacturing platform (Xometry-like) built with Rust, Axum, and PostgreSQL.

## Quick Start

```bash
# Start all services (first time takes ~5 min to build)
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f api
```

That's it! 🚀

## Services

| Service    | URL                         | Description                    |
|------------|-----------------------------|--------------------------------|
| API        | http://localhost:8080       | RapidFab API (Axum/Rust)       |
| PGWEB      | http://localhost:8081       | PostgreSQL Web UI              |
| Grafana    | http://localhost:3000       | Dashboards (admin/admin)       |
| Prometheus | http://localhost:9090       | Metrics                        |
| Loki       | http://localhost:3100       | Logs                           |
| PostgreSQL | localhost:5432              | Database                       |
| Redis      | localhost:6379              | Cache/Queue                    |

## API Endpoints

```bash
# Health check
curl http://localhost:8080/health/healthz

# Register user
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret123","full_name":"John Doe"}'

# Login
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret123"}'

# Get profile (use token from login response)
curl http://localhost:8080/users/me \
  -H "Authorization: Bearer <TOKEN>"

# Metrics (Prometheus format)
curl http://localhost:8080/metrics
```

## Testing

```bash
# Run E2E tests
./tests/e2e/auth_flow_test.sh

# Expected output:
# Test 1: Health check... ✅ PASS
# Test 2: Register user... ✅ PASS
# Test 3: Get user profile... ✅ PASS
# Test 4: Logout... ✅ PASS
# Test 5: Profile access after logout... ✅ PASS
# Test 6: Login with credentials... ✅ PASS
```

## Development

```bash
# Local development (without Docker)
cd services/api

# Setup env
cp .env.example .env

# Start database
docker-compose up -d postgres redis

# Run API
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       v
┌─────────────┐     ┌──────────────┐
│  API (Axum) │────▶│  PostgreSQL  │
└──────┬──────┘     └──────────────┘
       │
       ├──────▶ Loki (Logs)
       ├──────▶ Prometheus (Metrics)
       └──────▶ Redis (Queue/Cache)
```

## Features

- ✅ **Auth**: Register/Login/Logout with Argon2 password hashing
- ✅ **Sessions**: 30-day session tokens with secure cleanup
- ✅ **Health Checks**: `/healthz`, `/readyz` for monitoring
- ✅ **Metrics**: Prometheus metrics on `/metrics`
- ✅ **Observability**: Full stack (Loki + Prometheus + Grafana)
- ✅ **Database UI**: PGWEB for easy database inspection
- ✅ **CI/CD**: GitHub Actions (6 jobs)
- ✅ **Clean Architecture**: Repository pattern, SOLID principles

## Tech Stack

**Backend:**
- Rust 2021 (Axum web framework)
- PostgreSQL 15 (with sqlx)
- Redis 7 (cache/queue)

**Observability:**
- Loki (log aggregation)
- Promtail (log collection)
- Prometheus (metrics)
- Grafana (visualization)

**Security:**
- Argon2 password hashing
- Session-based authentication
- SQL injection protection (sqlx compile-time checks)
- No stack traces in error responses

## Project Structure

```
rapidfab.xyz/
├── services/
│   └── api/              # Axum API (Rust)
│       ├── src/
│       │   ├── app/      # Domain modules (auth, users, health, metrics)
│       │   ├── middleware/
│       │   └── main.rs
│       ├── migrations/   # SQL migrations
│       ├── tests/        # Integration tests
│       └── docs/         # API documentation
├── infra/
│   └── docker/           # Observability configs (Loki, Prometheus, Grafana)
├── tests/
│   └── e2e/              # End-to-end tests
├── plan/                 # ADR documents
├── docker-compose.yml    # All services
└── Makefile              # Build targets
```

## Documentation

- [API Documentation](services/api/docs/INDEX.md)
- [Auth Module](services/api/docs/AUTH.md)
- [Database Schema](services/api/docs/DATABASE.md)
- [Architecture](services/api/docs/ARCHITECTURE.md)
- [Testing Strategy](plan/PRD-002-testing-strategy.md)

## Roadmap

- [x] **M0**: Skeleton + Auth + Observability ✅
- [ ] **M1**: Pricing FDM (OrcaSlicer) + Upload flow
- [ ] **M2**: Orders + Stripe + Email service
- [ ] **M3**: Admin panel
- [ ] **M4**: Frontend (Svelte) + Optimizations

## License

MIT

## Contributing

See [CLAUDE.md](CLAUDE.md) for development philosophy and coding standards.

---

🤖 Built with [Claude Code](https://claude.com/claude-code)
