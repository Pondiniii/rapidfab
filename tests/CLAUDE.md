# Testing - Instrukcje dla Agentów LLM

## Workflow po zakończeniu pracy

Po zaimplementowaniu feature'a **ZAWSZE** uruchom:

```bash
task ci
```

**AGENT NIE MOŻE COMMITOWAĆ JEŚLI `task ci` FAIL!**

## Co robi `task ci`

Pełny pipeline CI w jednej komendzie:

1. **Format check** - `cargo fmt --check` - sprawdza formatowanie kodu
2. **Linter** - `cargo clippy -D warnings` - sprawdza jakość kodu i potencjalne błędy
3. **Unit tests** - `cargo test --lib --bins` - testy jednostkowe
4. **Integration tests** - `cargo test --test '*'` - testy integracyjne
5. **E2E tests** - auto-discovery wszystkich `tests/e2e/*_test.sh`

Jeśli którykolwiek krok fail → **STOP, napraw, powtórz**.

## Dostępne komendy

### CI Pipeline
```bash
task              # Full CI (default)
task ci           # To samo
```

### Development
```bash
task fmt          # Check format
task fmt:fix      # Auto-fix format
task lint         # Run clippy
task test         # Unit + Integration tests
task test:unit    # Only unit tests
task test:integration  # Only integration tests
task test:e2e     # Only E2E tests
task build        # Build release binary
task run          # Run API locally
task clean        # Clean build artifacts
```

### Docker (opcjonalne)
```bash
task docker:build # Build Docker images
task docker:up    # Start services
task docker:down  # Stop services
```

### Help
```bash
task help         # List all tasks
task --list       # To samo
```

## Jak dodać nowy test E2E

### 1. Utwórz plik testu

```bash
touch tests/e2e/my_feature_test.sh
chmod +x tests/e2e/my_feature_test.sh
```

### 2. Napisz test (bash + curl)

```bash
#!/usr/bin/env bash
set -euo pipefail

# Test configuration
API_URL="${API_URL:-http://localhost:8080}"

echo "Testing my feature..."

# Test request
RESPONSE=$(curl -sf "$API_URL/api/my-endpoint" \
  -H "Content-Type: application/json" \
  -d '{"key": "value"}')

# Validate response
if echo "$RESPONSE" | grep -q "expected_value"; then
  echo "✅ Test passed"
  exit 0
else
  echo "❌ Test failed: unexpected response"
  echo "Response: $RESPONSE"
  exit 1
fi
```

### 3. Odpal CI

```bash
task ci  # Auto-discovers your new test!
```

**Zero konfiguracji potrzebnej!**

## Convention-based Testing

### File Naming Convention
- **Lokalizacja**: `tests/e2e/`
- **Naming pattern**: `*_test.sh` (MUST end with `_test.sh`)
- **Executable**: `chmod +x` (auto-fixed if missing)
- **Exit code**: `0` = pass, non-zero = fail

### Przykłady poprawnych nazw:
- ✅ `auth_flow_test.sh`
- ✅ `upload_file_test.sh`
- ✅ `payment_checkout_test.sh`
- ❌ `test_auth.sh` (doesn't end with `_test.sh`)
- ❌ `auth_test.txt` (wrong extension)

### Test Structure Best Practices

```bash
#!/usr/bin/env bash
set -euo pipefail  # Fail fast on errors

# 1. Configuration (with defaults)
API_URL="${API_URL:-http://localhost:8080}"
TIMEOUT="${TIMEOUT:-5}"

# 2. Setup (if needed)
echo "Setting up test..."
# Create test data, etc.

# 3. Test execution
echo "Running test: feature description"
RESPONSE=$(curl -sf --max-time "$TIMEOUT" "$API_URL/endpoint")

# 4. Assertions
if [ "$RESPONSE" = "expected" ]; then
  echo "✅ Test passed"
  exit 0
else
  echo "❌ Test failed"
  exit 1
fi

# 5. Cleanup (if needed)
# Delete test data, etc.
```

## Environment Variables

Tests can use these environment variables:

```bash
API_URL=http://localhost:8080    # API endpoint
TIMEOUT=10                        # Request timeout in seconds
DATABASE_URL=postgresql://...    # DB connection (if needed)
```

Override in test runner:
```bash
API_URL=http://staging.example.com task test:e2e
```

## Debugging Failed Tests

### 1. Run single test manually

```bash
./tests/e2e/auth_flow_test.sh
```

### 2. Enable verbose mode

Add to test:
```bash
set -x  # Enable bash debug mode
```

### 3. Check API logs

```bash
# If running locally
RUST_LOG=debug cargo run

# If running in Docker
docker-compose logs -f api
```

### 4. Run test with custom API

```bash
API_URL=http://localhost:9000 ./tests/e2e/my_test.sh
```

## Common Errors & Solutions

### Error: "Tests directory not found"
```bash
# Solution: Create directory
mkdir -p tests/e2e
```

### Error: "Permission denied"
```bash
# Solution: Make test executable
chmod +x tests/e2e/my_test.sh
```

### Error: "curl: command not found"
```bash
# Solution: Install curl
sudo pacman -S curl  # Arch
sudo apt install curl  # Debian/Ubuntu
```

### Error: "cargo: command not found"
```bash
# Solution: Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Error: "task: command not found"
```bash
# Solution: Install Task
# See: https://taskfile.dev/installation/

# Arch Linux
sudo pacman -S go-task

# Or using go
go install github.com/go-task/task/v3/cmd/task@latest

# Or download binary from GitHub releases
```

## Integration with Git Hooks (Future)

Opcjonalnie można dodać pre-commit hook:

```bash
# .git/hooks/pre-commit
#!/bin/bash
task ci || {
  echo "❌ CI failed - commit blocked"
  exit 1
}
```

**Uwaga**: Na razie nie implementujemy hooków - user ręcznie odpala `task ci`.

## Performance Tips

### Fast iteration during development

```bash
# Only format + lint (fast)
task fmt && task lint

# Only unit tests (faster than integration)
task test:unit

# Full CI (before commit)
task ci
```

### Parallel test execution

E2E tests run sequentially by default (safety). Jeśli testy są niezależne, można je zrównoleglić:

```bash
# Edit scripts/test-e2e.sh and add parallel execution
# (Not implemented yet - KISS principle)
```

## Test Coverage

To check test coverage (future enhancement):

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cd services/api
cargo tarpaulin --out Html
```

## Summary

1. **After coding**: Run `task ci`
2. **If fail**: Fix errors, repeat
3. **If pass**: Commit & push
4. **Zero config**: Drop `*_test.sh` in `tests/e2e/`, it auto-runs

**Golden Rule**: `task ci` musi pass przed każdym commitem.

## Questions?

- Check `Taskfile.yml` for task definitions
- Check `scripts/test-e2e.sh` for E2E runner implementation
- Check existing test: `tests/e2e/auth_flow_test.sh` for example

Keep it simple. Keep it working.
