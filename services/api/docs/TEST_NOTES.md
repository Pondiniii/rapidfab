# Test Coverage — Auth + Users (M0)

## Test Commands

```bash
# Unit tests only
cargo test --lib

# Integration tests (requires Postgres)
cargo test --test '*'

# All tests
cargo test

# With output
cargo test -- --nocapture
```

---

## Unit Tests

Located in respective module files (e.g., `src/app/auth/service.rs#[cfg(test)]`).

**Coverage:**
- [x] Password hashing (Argon2)
- [x] Token generation (UUID v4)
- [x] Error types and conversions

---

## Integration Tests

Located in `tests/integration_test.rs`.

**Setup:**
- Requires PostgreSQL running (use `docker-compose up -d` from root)
- Cleans up test data after each test

**Test Cases:**
- [x] Register user → create user + session
- [x] Register duplicate user → 409 conflict
- [x] Login valid user → create new session
- [x] Login invalid email → 401 unauthorized
- [x] Login invalid password → 401 unauthorized
- [x] Logout valid token → delete session
- [x] Logout invalid token → 401 unauthorized
- [x] GET /users/me with valid token → return user
- [x] GET /users/me without token → 401 unauthorized
- [x] GET /users/me with expired token → 401 unauthorized

---

## Assumptions & Edge Cases

**Assumptions:**
1. PostgreSQL is running and accessible via `DATABASE_URL`
2. Migrations run automatically on startup
3. Password minimum length is not enforced at API level (handled in client)

**Edge Cases:**
- Email validation is minimal (format not validated server-side)
- Concurrent registrations with same email — DB constraint prevents race
- Session expiry checked on middleware level (no cleanup task)

---

## Known Gaps (TODO for next phase)

- [ ] Load tests (concurrency)
- [ ] Rate limiting tests
- [ ] CORS tests
- [ ] Health check endpoint tests (`/healthz`, `/readyz`)
- [ ] Database connection pool exhaustion tests
