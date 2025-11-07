# RapidFab API Service

Axum-based REST API for RapidFab.xyz platform.

## Quick Start

1. Copy `.env.example` to `.env` and adjust values
2. Start PostgreSQL: `docker-compose up -d` (from root)
3. Run migrations: `cargo run` (automatic on startup)
4. Start API: `make run`

## API Endpoints

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| GET | `/health/healthz` | Health check | No |
| GET | `/health/readyz` | Readiness check (DB) | No |
| GET | `/metrics` | Prometheus metrics | No |
| POST | `/auth/register` | Register user | No |
| POST | `/auth/login` | Login user | No |
| POST | `/auth/logout` | Logout user | Yes |
| GET | `/users/me` | Get current user | Yes |

**Auth:** Bearer token in `Authorization: Bearer <token>` header

## Development

- `make fmt` - Format code
- `make lint` - Run clippy
- `make test` - Run all tests
- `make test-unit` - Unit tests only
- `make test-integration` - Integration tests only
- `make build` - Build release binary

## Architecture

- `src/main.rs` - Entry point and server setup
- `src/config.rs` - Environment configuration
- `src/db.rs` - Database pool and migrations
- `src/error.rs` - Error types and responses
- `src/app/` - Domain modules (auth, users, etc.)
- `src/middleware/` - Custom middleware (session auth, etc.)

## Database Migrations

Migrations are stored in `migrations/` and run automatically on startup using `sqlx migrate`.

## Testing

Integration tests require a running PostgreSQL instance. Use `docker-compose up -d` from root.
