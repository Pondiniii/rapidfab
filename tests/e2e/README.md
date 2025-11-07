# E2E Tests

End-to-end tests for the RapidFab API.

## Available Tests

### Auth Flow Test

Tests the complete authentication flow:
1. Health check
2. User registration
3. Profile access (authenticated)
4. Logout
5. Profile access after logout (should fail with 401)
6. Login with credentials

## Running Tests

### Prerequisites
- `curl` installed
- API running on http://localhost:8080 (or custom BASE_URL)
- PostgreSQL database available

### Local

```bash
# Start API and dependencies
docker-compose up -d api

# Wait for API to be ready
sleep 5

# Run test
./tests/e2e/auth_flow_test.sh
```

### With Custom URL

```bash
BASE_URL=http://localhost:3000 ./tests/e2e/auth_flow_test.sh
```

### From Makefile

```bash
# From project root
make test-e2e

# Or from services/api
cd services/api
make test-e2e
```

## CI/CD

E2E tests run automatically in GitHub Actions as part of the CI pipeline.

## Test Output

Successful run:
```
=== E2E Test: Auth Flow ===
Base URL: http://localhost:8080
Email: test-1699999999@example.com
Test 1: Health check... ✅ PASS
Test 2: Register user... ✅ PASS
Test 3: Get user profile... ✅ PASS
Test 4: Logout... ✅ PASS
Test 5: Profile access after logout (should fail)... ✅ PASS
Test 6: Login with credentials... ✅ PASS

=== All E2E tests passed! ✅ ===
```

## Troubleshooting

### Connection Refused
- Ensure API is running: `docker-compose ps`
- Check port mapping: `docker-compose port api 8080`

### Test Failures
- Check API logs: `docker-compose logs api`
- Verify database: `docker-compose ps postgres`
- Try manual curl: `curl http://localhost:8080/health/healthz`

### Database Issues
- Reset database: `docker-compose down -v && docker-compose up -d`
- Run migrations: `cd services/api && sqlx migrate run`
