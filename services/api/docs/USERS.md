# Users Module

## Overview
Read-only endpoint for authenticated users to fetch their profile.

---

## Endpoints

### GET /users/me
Get current user profile. **Requires auth.**

**Headers:**
```
Authorization: Bearer <token>
```

**Response (200):**
```json
{
  "id": "uuid-string",
  "email": "user@example.com",
  "full_name": "John Doe",
  "created_at": "2025-01-15T10:30:00Z"
}
```

**Errors:**
- 401: Missing/invalid/expired token

---

## Flow

1. Extract Bearer token from `Authorization` header
2. Middleware validates token existence + expiry
3. Query `users` table by user_id from session
4. Return user record

---

## Authentication Middleware

Located in `src/middleware/session.rs`. Applied at router level via `.layer(middleware::from_fn(require_auth))`.

**Steps:**
1. Extract token from header
2. Query `sessions` table: `token = ? AND expires_at > NOW()`
3. If valid: inject `user_id` into Extensions
4. If invalid: return 401

---

## Code Structure

- `routes.rs` - HTTP handler (get_me)
- `dto.rs` - Response types (UserResponse)
