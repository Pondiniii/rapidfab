# Template Service

> **TODO: REPLACE THIS** - Describe what this service does in 1-2 sentences.
> Example: "Handles order processing and communicates with pricing service."

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
| `SERVICE_HOST` | Host to bind to | `0.0.0.0` |
| `SERVICE_PORT` | Port to listen on | `8080` |
| `RUST_LOG` | Log level | `info` |

**TODO: Add service-specific environment variables here**

## Endpoints

### Health Check
- **GET** `/health`
- Returns service health status

**TODO: Document your service endpoints here**

Example:
```bash
curl http://localhost:8080/health
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

**Template Service** | Part of RapidFab.xyz
