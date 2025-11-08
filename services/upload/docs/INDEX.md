# Upload Service Documentation

## Architecture Overview

### Purpose
File upload management and S3 integration for RapidFab.xyz. This service handles:
- Multipart upload initialization and signed URL generation
- File storage in Hetzner S3 (anonymous and user-owned)
- Upload quota enforcement (anonymous daily, user monthly)
- File transfers from anonymous to user storage (on login/registration)
- Read-only file access via signed URLs

### Dependencies
- PostgreSQL (upload metadata, quotas)
- Hetzner S3 (file storage)

### Technology Stack
- Rust 1.75+
- Axum web framework
- Tokio async runtime
- AWS SDK for S3 (Hetzner compatible)
- SQLx (PostgreSQL)

## S3 Folder Structure

```
s3://rapidfab-uploads/
├── anon/{session_id}/          # Anonymous uploads (7 day TTL)
│   ├── {file_id}.step
│   └── {file_id}.stl
└── users/{user_id}/             # User uploads (permanent)
    ├── {file_id}.step
    └── {file_id}.stl
```

## API Contracts

### Endpoints

#### GET /health
Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "service": "upload"
}
```

#### POST /internal/upload/init
Initialize multipart upload session.

**Request:**
```json
{
  "file_name": "part.step",
  "file_size": 1048576,
  "session_id": "uuid-v4",
  "user_id": "uuid-v4"  // Optional, null for anonymous
}
```

**Response:**
```json
{
  "upload_id": "uuid-v4",
  "expires_at": "2024-01-15T12:00:00Z"
}
```

#### POST /internal/upload/{id}/signed-urls
Generate signed URLs for multipart upload chunks.

**Request:**
```json
{
  "chunk_count": 5
}
```

**Response:**
```json
{
  "urls": [
    {"part_number": 1, "url": "https://..."},
    {"part_number": 2, "url": "https://..."}
  ]
}
```

#### POST /internal/upload/{id}/confirm
Confirm upload completion.

**Request:**
```json
{
  "etags": [
    {"part_number": 1, "etag": "abc123"},
    {"part_number": 2, "etag": "def456"}
  ]
}
```

**Response:**
```json
{
  "file_id": "uuid-v4",
  "s3_key": "anon/{session_id}/{file_id}.step"
}
```

#### POST /internal/upload/transfer
Transfer files from anonymous to user storage.

**Request:**
```json
{
  "session_id": "uuid-v4",
  "user_id": "uuid-v4"
}
```

**Response:**
```json
{
  "transferred_count": 3,
  "file_ids": ["uuid-1", "uuid-2", "uuid-3"]
}
```

#### GET /internal/upload/file/{id}/read-url
Generate temporary read-only signed URL.

**Response:**
```json
{
  "url": "https://s3.endpoint/...",
  "expires_at": "2024-01-15T12:00:00Z"
}
```

## Configuration

### Environment Variables

| Variable | Required | Description | Default |
|----------|----------|-------------|---------|
| `UPLOAD_HOST` | No | Bind host | `0.0.0.0` |
| `UPLOAD_PORT` | No | Service port | `8082` |
| `RUST_LOG` | No | Log level | `info` |
| `DATABASE_URL` | Yes | PostgreSQL connection | - |
| `S3_ENDPOINT` | Yes | Hetzner S3 endpoint | - |
| `S3_BUCKET` | Yes | S3 bucket name | - |
| `S3_REGION` | Yes | S3 region | - |
| `S3_ACCESS_KEY_ID` | Yes | S3 access key | - |
| `S3_SECRET_ACCESS_KEY` | Yes | S3 secret key | - |
| `QUOTA_ANON_DAILY_MB` | No | Anonymous daily quota | `100` |
| `QUOTA_USER_MONTHLY_GB` | No | User monthly quota | `10` |

## Testing

### Running Tests

```bash
# All tests
make test

# Specific test
cargo test test_name
```

### Test Structure

- Unit tests: inline in `src/` files
- Integration tests: `tests/*.rs`
- E2E tests: `../../tests/e2e/upload_test.sh`

## Error Handling

### Error Codes

| Code | Status | Description |
|------|--------|-------------|
| `QUOTA_EXCEEDED` | 429 | Upload quota exceeded |
| `INVALID_FILE` | 400 | Invalid file format or size |
| `UPLOAD_NOT_FOUND` | 404 | Upload session not found |
| `S3_ERROR` | 500 | S3 operation failed |

## Deployment

### Docker

```bash
docker build -t rapidfab-upload-service -f Containerfile .
docker run -p 8082:8082 rapidfab-upload-service
```

### Health Checks

- Endpoint: `GET /health`
- Expected: `200 OK`

---

**Last Updated:** 2025-01-08
