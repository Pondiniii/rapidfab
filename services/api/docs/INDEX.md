# RapidFab API Documentation Index

M0 Implementation — Auth + Users baseline.

---

## Quick Start

```bash
# Setup
export DATABASE_URL=postgresql://user:pass@localhost/rapidfab
cargo build
cargo test

# Run
make run

# Test endpoints
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@test.com","password":"pass","full_name":"Test"}'

curl -X GET http://localhost:3000/users/me \
  -H "Authorization: Bearer <token>"
```

---

## Modules

- [AUTH.md](./AUTH.md) — Register / Login / Logout endpoints, Argon2 hashing, session mgmt
- [USERS.md](./USERS.md) — GET /users/me, auth middleware
- [DATABASE.md](./DATABASE.md) — SQL schema, tables (users, sessions), indexes
- [ARCHITECTURE.md](./ARCHITECTURE.md) — Directory structure, data flow, components
- [TEST_NOTES.md](./TEST_NOTES.md) — Test coverage, integration test cases

---

## Endpoints Summary

| Method | Path | Auth | Response |
|--------|------|------|----------|
| POST | `/auth/register` | No | `{token, user_id}` |
| POST | `/auth/login` | No | `{token, user_id}` |
| POST | `/auth/logout` | Bearer | 204 No Content |
| GET | `/users/me` | Bearer | `{id, email, full_name, created_at}` |

---

## Error Responses

All errors return JSON:
```json
{"error": "Error message"}
```

Status codes:
- 400 — Bad Request / Validation error
- 401 — Unauthorized / Invalid token
- 409 — Conflict (e.g., user already exists)
- 500 — Internal Server Error

---

## Architecture Notes

- **Framework:** Axum 0.7 (async Rust web framework)
- **DB:** PostgreSQL + sqlx (type-safe queries)
- **Auth:** Argon2 (password hashing) + UUID sessions
- **Logging:** tracing + JSON output
- **Testing:** cargo test (unit + integration)
