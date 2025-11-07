# Auth Module

## Overview
User authentication via email/password. Sessions stored in DB with 30-day TTL.

## Endpoints

### POST /auth/register
Register new user.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "secure_password",
  "full_name": "John Doe"  // optional
}
```

**Response (201):**
```json
{
  "token": "uuid-v4-string",
  "user_id": "uuid-string"
}
```

**Errors:**
- 409: User already exists

---

### POST /auth/login
Authenticate user and create session.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "secure_password"
}
```

**Response (200):**
```json
{
  "token": "uuid-v4-string",
  "user_id": "uuid-string"
}
```

**Errors:**
- 401: Invalid credentials

---

### POST /auth/logout
Invalidate session token.

**Headers:**
```
Authorization: Bearer <token>
```

**Response (204):** No content

**Errors:**
- 401: Missing/invalid token

---

## Flow

1. **Register** → Hash pwd (Argon2) → Create user → Create session → Return token
2. **Login** → Find user by email → Verify pwd → Create new session → Return token
3. **Logout** → Delete session from DB

---

## Security

- **Password Hashing:** Argon2 (default params)
- **Session Token:** UUID v4 (random, cryptographically secure)
- **Token Storage:** Plaintext in DB (indexed on `token` column)
- **Token Expiry:** 30 days (checked at middleware level)
- **Protected Routes:** All `/users/*` require valid, non-expired session

---

## Code Structure

- `service.rs` - Business logic (hashing, validation)
- `repository.rs` - DB queries (create_user, create_session, delete_session)
- `routes.rs` - HTTP handlers
- `dto.rs` - Request/Response types
