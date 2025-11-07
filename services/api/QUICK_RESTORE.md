# Quick Restore — RapidFab API (M0)

**Status:** Auth + Users baseline complete (M0 Phases 1-2).

---

## What's Here

Email/password auth (Argon2) + session-based user profiles. 4 endpoints: `/auth/register`, `/auth/login`, `/auth/logout`, `/users/me`.

---

## Key Files

- `src/app/auth/` — Register/Login/Logout (service, routes, repository)
- `src/app/users/` — GET /users/me (protected endpoint)
- `src/middleware/session.rs` — Bearer token validation
- `migrations/` — SQL schema (users, sessions tables)
- `docs/` — Full micro-docs (see below)

---

## Docs

All in `docs/`:
- **INDEX.md** — Overview + Quick Start + endpoint table
- **AUTH.md** — Endpoints, flow, security (Argon2, 30-day TTL)
- **USERS.md** — Protected profile endpoint, middleware
- **DATABASE.md** — SQL schema, indexes, pool config
- **ARCHITECTURE.md** — Directory structure, data flow, stack
- **TEST_NOTES.md** — Integration test cases, coverage

---

## Tech Stack

| Component | Choice |
|-----------|--------|
| Framework | Axum (async Rust) |
| Database | PostgreSQL + sqlx |
| Password | Argon2 |
| Sessions | UUID v4, 30-day TTL |
| Logging | tracing JSON |

---

## Quick Commands

```bash
# Test
cargo test

# Run
make run

# Build
cargo build --release
```

---

## Current Gaps

- [ ] `/healthz`, `/readyz`, `/metrics` endpoints
- [ ] File upload / S3 integration
- [ ] Worker queue (Redis/NATS)
- [ ] Health checks, rate limiting

Next: File upload + pricing worker (M1).
