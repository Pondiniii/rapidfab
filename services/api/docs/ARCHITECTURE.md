# API Architecture — M0 Baseline

## Overview
Minimal Axum HTTP API with PostgreSQL backend. Modular structure: each domain gets `app/<domain>/` with routes/dto/service/repository.

---

## Directory Structure

```
services/api/
├── src/
│   ├── main.rs              # Server setup, router creation
│   ├── lib.rs               # Crate exports
│   ├── config.rs            # Env config loading
│   ├── db.rs                # PgPool, migrations
│   ├── error.rs             # AppError enum, JSON responses
│   ├── app/
│   │   ├── mod.rs           # Module exports
│   │   ├── auth/            # Auth module
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs    # HTTP handlers
│   │   │   ├── service.rs   # Business logic
│   │   │   ├── repository.rs # DB queries
│   │   │   └── dto.rs       # Request/Response types
│   │   └── users/           # Users module
│   │       ├── mod.rs
│   │       ├── routes.rs
│   │       └── dto.rs
│   └── middleware/
│       ├── mod.rs
│       └── session.rs       # Auth middleware (Bearer token validation)
├── migrations/
│   ├── 001_create_users_table.sql
│   └── 002_create_sessions_table.sql
├── tests/
│   └── integration_test.rs
├── Makefile
├── Cargo.toml
├── README.md
└── docs/                    # This folder
    ├── INDEX.md
    ├── AUTH.md
    ├── USERS.md
    ├── DATABASE.md
    └── ARCHITECTURE.md
```

---

## Data Flow

### Registration
```
POST /auth/register
  ↓
routes::register()
  ↓
service::register()
  ├─ Check user exists (repository)
  ├─ Hash password (Argon2)
  ├─ Create user (repository)
  ├─ Create session (repository)
  └─ Return {token, user_id}
```

### Login
```
POST /auth/login
  ↓
routes::login()
  ↓
service::login()
  ├─ Find user by email (repository)
  ├─ Verify password (Argon2)
  ├─ Create session (repository)
  └─ Return {token, user_id}
```

### Protected Request
```
GET /users/me + Bearer token
  ↓
middleware::require_auth()
  ├─ Extract token from header
  ├─ Find valid session (repository)
  ├─ Inject user_id into Extensions
  └─ Continue or reject (401)
  ↓
routes::get_me()
  ├─ Extract user_id from Extensions
  ├─ Query user by ID (repository)
  └─ Return {id, email, full_name, created_at}
```

---

## Key Components

### Error Handling
`src/error.rs` — Single error enum that converts to JSON:
```rust
#[serde(crate = "serde")]
pub enum AppError {
    UserAlreadyExists,    // → 409
    InvalidCredentials,   // → 401
    Unauthorized,         // → 401
    NotFound,            // → 404
    Internal,            // → 500
    // ...
}
```

Auto-implements Axum's `IntoResponse` → JSON response + status code.

### Database
`src/db.rs`:
- Create PgPool (max 10 connections)
- Run migrations from `./migrations/`
- All queries use `sqlx::query_as!()` for type safety

### Middleware
`src/middleware/session.rs`:
- `require_auth` — checks Bearer token + expiry
- Extends request with user_id
- Can be composed with any router

---

## Technology Stack

| Component | Choice | Why |
|-----------|--------|-----|
| Web Framework | Axum | Minimal, composable, fast |
| Database | PostgreSQL | Reliable, ACID, good Rust support |
| ORM/Queries | sqlx | Type-safe, compile-time checked |
| Password Hash | Argon2 | Modern, secure, resistant to GPU attacks |
| Session Token | UUID v4 | Simple, unpredictable, easy to index |
| Logging | tracing | Structured logs, JSON output |
| Runtime | Tokio | Async, multi-threaded |

---

## Current Limitations (TODO)

- [ ] No refresh token mechanism
- [ ] No password reset endpoint
- [ ] No email verification
- [ ] No cleanup of expired sessions (periodic task)
- [ ] No rate limiting
- [ ] No CORS configured
