# Database Schema

## Overview
PostgreSQL with `sqlx` for type-safe queries. Migrations auto-run on startup.

---

## Tables

### `users`

Core user data. Email is unique.

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
```

| Column | Type | Notes |
|--------|------|-------|
| `id` | UUID | PK, auto-generated |
| `email` | VARCHAR(255) | UNIQUE, used for login |
| `password_hash` | VARCHAR(255) | Argon2 hash |
| `full_name` | VARCHAR(255) | Optional |
| `created_at` | TIMESTAMPTZ | Auto NOW() |
| `updated_at` | TIMESTAMPTZ | Auto NOW() |

---

### `sessions`

User sessions with expiry. Foreign key to `users`.

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
```

| Column | Type | Notes |
|--------|------|-------|
| `id` | UUID | PK, auto-generated |
| `user_id` | UUID | FK to `users.id`, CASCADE delete |
| `token` | VARCHAR(255) | UNIQUE, session identifier |
| `expires_at` | TIMESTAMPTZ | Checked by middleware |
| `created_at` | TIMESTAMPTZ | Auto NOW() |

**Constraints:**
- `ON DELETE CASCADE` — deleting user removes all sessions

---

## Indexes

| Table | Column(s) | Purpose |
|-------|-----------|---------|
| `users` | `email` | Fast lookup by email (login) |
| `sessions` | `token` | Fast session validation |
| `sessions` | `user_id` | Find sessions by user |
| `sessions` | `expires_at` | Cleanup expired sessions |

---

## Migrations

Located in `migrations/`:

1. `001_create_users_table.sql` - Users table + email index
2. `002_create_sessions_table.sql` - Sessions table + indexes

Run automatically on startup via `sqlx migrate!()`.

---

## Maintenance

- **Expired Sessions:** No cleanup task yet (TODO: add periodic cleanup)
- **Connections:** Pool: 10 max, 3s acquire timeout
