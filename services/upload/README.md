# Upload Service

File upload management and S3 integration for RapidFab.xyz. Handles multipart uploads, temporary storage, quota enforcement, and file transfers between anonymous and user storage.

## Quick Start

```bash
# Development
make run

# Run tests
make test

# Build Docker image
make docker-build

# Run in Docker
make docker-run
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `UPLOAD_HOST` | Host to bind to | `0.0.0.0` |
| `UPLOAD_PORT` | Port to listen on | `8082` |
| `RUST_LOG` | Log level | `info` |
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `S3_ENDPOINT` | Hetzner S3 endpoint | Required |
| `S3_BUCKET` | S3 bucket name | Required |
| `S3_REGION` | S3 region | Required |
| `S3_ACCESS_KEY_ID` | S3 access key | Required |
| `S3_SECRET_ACCESS_KEY` | S3 secret key | Required |
| `QUOTA_ANON_DAILY_MB` | Anonymous daily quota (MB) | `100` |
| `QUOTA_USER_MONTHLY_GB` | User monthly quota (GB) | `10` |

## Endpoints

### Health Check
- **GET** `/health`
- Returns service health status

Example:
```bash
curl http://localhost:8082/health
```

## Development

```bash
# Format code
make fmt

# Lint
make lint

# Run tests
make test

# Build release
make build
```

## Architecture

See `docs/INDEX.md` for detailed architecture and API contracts.

---

**Upload Service** | Part of RapidFab.xyz
