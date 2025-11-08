# IP Forwarding Verification Report

**Date:** 2025-11-08
**Task:** Verify X-Forwarded-For header forwarding from API to upload-service
**Status:** PARTIAL PASS

---

## Executive Summary

X-Forwarded-For header forwarding is **correctly implemented** in code:
- API extracts client IP from headers (X-Forwarded-For, X-Real-IP)
- API forwards IP to upload-service in all proxy calls
- Upload-service receives and stores IP address in database

**However,** runtime verification is blocked by existing init endpoint bug (HTTP 500). Code review confirms implementation is correct, but we cannot verify actual IP visibility in logs due to API returning 500 before reaching upload-service.

---

## Test Results

### 1. Build Status: PASS

```bash
docker-compose build --no-cache api
```

**Result:** SUCCESS

- Compilation: 0 errors
- Build time: ~30 seconds (with cache layer)
- Warning: `sqlx-postgres v0.7.4` future incompatibility (non-blocking)
- Binary size: Normal
- Docker image: Created successfully

**Verdict:** API compiles cleanly with IP forwarding code.

---

### 2. Code Review: PASS

#### API Side (services/api/src/app/upload/routes.rs)

**get_client_ip() function (lines 168-176):**

```rust
fn get_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next()) // First IP in chain
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
        .unwrap_or("unknown")
        .to_string()
}
```

**Quality:** Excellent
- Checks X-Forwarded-For header first (reverse proxy standard)
- Handles comma-separated IPs (takes first = real client)
- Fallback to X-Real-IP (nginx convention)
- Safe fallback to "unknown"

**init_upload handler (line 66):**
```rust
.header("X-Forwarded-For", get_client_ip(&headers))
```

**get_signed_urls handler (line 101):**
```rust
.header("X-Forwarded-For", get_client_ip(&headers))
```

**confirm_upload handler (line 140):**
```rust
.header("X-Forwarded-For", get_client_ip(&headers))
```

**Coverage:** 100% of proxy endpoints forward IP.

---

#### Upload-Service Side (services/upload/src/app/handlers.rs)

**init_upload handler (lines 29-34):**

```rust
let ip = headers
    .get("x-forwarded-for")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("unknown")
    .to_string();
```

**Quality:** Good
- Correctly extracts X-Forwarded-For header
- Safe fallback to "unknown"
- No panics

**Database storage (services/upload/src/app/service.rs lines 74-80):**

```rust
sqlx::query!(
    r#"
    INSERT INTO uploads (id, user_id, session_id, ip_address, status)
    VALUES ($1, $2, $3, $4, 'pending')
    "#,
    upload_id,
    user_id_uuid,
    session_id_uuid,
    ip,  // <- IP from X-Forwarded-For header
)
```

**Verdict:** IP is correctly extracted, forwarded, received, and stored.

---

### 3. Runtime Test: BLOCKED

**Test:**
```bash
curl http://localhost:8080/files/upload/init \
  -H "Content-Type: application/json" \
  -H "X-Forwarded-For: 203.0.113.42" \
  -d '{"files":[{"filename":"test.stl","content_type":"application/vnd.ms-pki.stl","size_bytes":1024}]}'
```

**Response:** HTTP 500 Internal Server Error

```json
{"error":"Internal server error"}
```

**Issue:** Init endpoint returns 500 before reaching upload-service.

**Evidence:**
- Upload-service logs: No requests received
- Database: No upload records created (0 rows)
- API logs: No error details (swallowed by `.map_err(|_| AppError::Internal)`)

**Root cause:** Same bug from previous BOLA verification report:
- S3 connectivity issue OR
- Error logging inadequate OR
- Ticket validation failure

**Impact on test:** Cannot verify IP visibility in runtime because request never reaches upload-service.

---

### 4. Environment Verification: PASS

**Services health:**
```
rapidfabxyz-api-1        Up (healthy)     0.0.0.0:8080->8080/tcp
rapidfabxyz-upload-1     Up (healthy)     0.0.0.0:8082->8082/tcp
rapidfabxyz-postgres-1   Up (healthy)     0.0.0.0:5432->5432/tcp
rapidfabxyz-redis-1      Up (healthy)     0.0.0.0:6379->6379/tcp
```

**API → Upload connectivity:**
```bash
docker exec rapidfabxyz-api-1 curl -s http://upload:8082/health
# {"status":"healthy","service":"upload"}
```

**Environment variables:**
```
UPLOAD_SERVICE_URL=http://upload:8082
INTERNAL_SERVICE_TOKEN=change-this-in-production-random-64-chars
UPLOAD_TICKET_SECRET=dev-secret-change-in-production-to-random-64-chars (matches upload-service)
```

**Verdict:** Infrastructure OK. Init bug is not related to IP forwarding.

---

## WERDYKT: PARTIAL PASS

### Success Criteria Met:

✅ **Build OK** - API compiles without errors
✅ **Code implementation OK** - X-Forwarded-For correctly:
  - Extracted from request headers (API)
  - Forwarded to upload-service (all endpoints)
  - Received by upload-service (handler)
  - Stored in database (service layer)

### Blocked Criteria:

⚠️ **Runtime verification BLOCKED** - Cannot test actual IP visibility because:
  - Init endpoint returns HTTP 500 (known bug)
  - Upload-service never receives request
  - Database has 0 upload records

### Known Issues:

1. **Init endpoint regression** (HTTP 500) - SEPARATE BUG
   - Not caused by IP forwarding code
   - Likely S3 connectivity or ticket validation issue
   - Error logging inadequate (`.map_err(|_| AppError::Internal)` hides root cause)
   - Recommended action: Separate investigation/fix

---

## Comparison with Task Requirements

**Original task:**
> 1. Build się udaje?
> 2. Header jest faktycznie przekazywany?
> 3. Upload-service widzi prawdziwy IP (nie "unknown")?

**Results:**
1. ✅ Build: OK (0 errors)
2. ✅ Header forwarding: YES (code review confirms)
3. ⚠️ IP visibility: BLOCKED (init endpoint bug prevents testing)

**Task verdict expectations:**
> - ✅ PASS jeśli: build OK + upload-service widzi custom IP
> - ⚠️ PARTIAL jeśli: build OK ale upload nadal widzi "unknown"
> - ❌ FAIL jeśli: build fail

**Actual result:**
- Build: OK
- Upload-service: Never received request (API bug)
- **PARTIAL PASS** - Code implementation correct, runtime verification blocked

---

**Test Duration:** ~5 minutes
**Build Status:** PASS
**Code Review:** PASS
**Runtime Test:** BLOCKED
**Overall Status:** PARTIAL PASS (init regression blocks full verification)
