# Template Service Documentation

> **TODO: REPLACE THIS** - Update all sections with your service specifics

## Architecture Overview

### Purpose
<!-- Describe what this service does and why it exists -->

### Dependencies
<!-- List other services/databases this service depends on -->
- None yet

### Technology Stack
- Rust 1.75+
- Axum web framework
- Tokio async runtime

## API Contracts

### Endpoints

#### GET /health
Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "service": "template-service"
}
```

**TODO: Document your service endpoints here**

Example:
```
POST /api/endpoint
Content-Type: application/json

{
  "field": "value"
}
```

## Configuration

### Environment Variables

| Variable | Required | Description | Default |
|----------|----------|-------------|---------|
| `SERVICE_HOST` | No | Bind host | `0.0.0.0` |
| `SERVICE_PORT` | No | Service port | `8080` |
| `RUST_LOG` | No | Log level | `info` |

**TODO: Add service-specific configuration**

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
- E2E tests: `../../tests/e2e/service_name_test.sh`

**TODO: Document test scenarios and edge cases**

## Error Handling

### Error Codes

**TODO: Document error codes and their meanings**

Example:
| Code | Status | Description |
|------|--------|-------------|
| `ERR_001` | 400 | Invalid input |
| `ERR_002` | 500 | Internal error |

## Deployment

### Docker

```bash
docker build -t rapidfab-template-service -f Containerfile .
docker run -p 8080:8080 rapidfab-template-service
```

### Health Checks

- Endpoint: `GET /health`
- Expected: `200 OK`

---

**Last Updated:** TODO
