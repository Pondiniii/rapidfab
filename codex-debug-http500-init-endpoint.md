# HTTP 500 /files/upload/init Debug Report

## Root Cause Analysis
- Upload tickets issued by the API no longer contained the standard `exp` claim after the `expires_at` field was changed from `String` to `DateTime<Utc>`.
- `jsonwebtoken::Validation::new(Algorithm::HS256)` defaults to requiring `exp`, so the upload-service rejected every ticket with `MissingRequiredClaim("exp")` before any business logic executed.
- The API interpreted the 401 from upload-service as a generic 500, so the request died before reaching quota logic or DB writes, which is why no upload rows or logs were created.

## Investigation Steps
1. Reviewed the proxy flow in `services/api/src/app/upload/routes.rs` and confirmed the only fallible steps before contacting upload-service were JWT generation and the outbound HTTP call.
2. Added a round-trip unit test in `services/api/src/app/upload/mod.rs` to encode a ticket and immediately decode it with `jsonwebtoken`, which reproduced the exact `MissingRequiredClaim("exp")` error.
3. Validated upload-service’s `validate_ticket` path (`services/upload/src/auth/ticket.rs`) to confirm it uses the same `Validation` defaults and therefore fails the same way at runtime.

## Fix Description
- Annotated the `expires_at` field on both API and upload-service ticket structs with `#[serde(with = "chrono::serde::ts_seconds", rename = "exp")]`, restoring an RFC-compliant numeric `exp` claim while keeping a strongly typed `DateTime<Utc>` in code (`services/api/src/app/upload/mod.rs:14-22`, `services/upload/src/auth/ticket.rs:6-13`).
- Added the `anon_ticket_roundtrips` unit test to guard against future regressions and prove that tokens can be decoded with the stricter validator (`services/api/src/app/upload/mod.rs:79-108`).
- Introduced structured error logging around ticket generation, proxy requests, and response parsing so future 500s include actionable context (`services/api/src/app/upload/routes.rs:48-145`).

## Test Results / Verification
- `cd services/api && SQLX_OFFLINE=true cargo test app::upload::tests::anon_ticket_roundtrips -- --nocapture` ✅
- `cd services/upload && SQLX_OFFLINE=true cargo test -- --nocapture` ✅
- `task ci` ❌ — blocked because this environment cannot talk to the Docker daemon (`dial unix /var/run/docker.sock: connect: operation not permitted`).
- Runtime curl / DB verification ✅ pending — requires containers running; please run `docker-compose -f docker-compose.minimal.yml up -d` (or `task ci`) locally and re-run the curl & DB check once Docker access is available.
