# RapidFab.xyz — karta projektu (LLM Agent First)

## Po co istnieje ten dokument
Ujednolica kierunek pracy nad rapidfab.xyz i zapewnia, że każdy agent LLM rozumie wizję: minimalny, modularny hub wycen on-demand podobny do Xometry, lecz prostszy, tańszy i szybszy. Repozytorium ma być **LLM Agent First** – mała liczba plików, przewidywalna struktura katalogów, jasne kontrakty API i zasady współpracy między agentami.

## Cel produktu
- Stworzyć platformę „upload → instant quote → buy” dla druku 3D/CNC/MJF, działającą szybko i czytelnie na desktop/mobile.
- Backend ma być banalny do zrozumienia, modularny i bezpieczny, z łatwym wdrażaniem nowych funkcji bez kruszenia całości.
- System obsługuje wielu użytkowników (klienci, admin), jest stabilny i operuje w kosztach ~10 USD VPS + Hetzner S3.
- Zapewnić pełną kontrolę nad infrastrukturą (brak vendor lock-in typu AWS/GCP Mailgun).

## Filozofia i zasady LLM Agent First
- **Minimalizm katalogów i kodu**: jeden moduł = jeden cel, zero zbędnych warstw.
- **Jasne kontrakty**: każdy endpoint opisany w OpenAPI, kluczowe funkcje/trait-y mają krótkie doc-komentarze z efektem.
- **Deterministyczne flow**: brak ukrytej magii, brak globalnego stanu i side effectów poza warstwą infrastruktury.
- **Dokumentacja w repo**: krótkie pliki `.md` w katalogach głównych zamiast osobnych wiki.
- **Automatyzowalne zadania**: skrypty `make`/`invoke` z parametrami zrozumiałymi dla agentów.

## Styl kodowania
- **Less is more**: preferuj krótkie moduły Rust (< 300 linii); jeśli rośnie → rozbij na podmoduły.
- **SOLID / DRY / KISS**: brak powtórzeń, proste nazewnictwo (`verb_subject`), jawne interfejsy traits.
- **Czytelność > „spryt"**: unikamy makr, dopóki nie poprawiają czytelności; dependency injection przez konstruktor/trait.
- **Konwencja**: `cargo fmt`, `cargo clippy -D warnings`, testy (`cargo test`). W Python workerach `ruff` + `pytest` jeśli są użyte.
- **Logging i błędy**: `tracing` z poziomami INFO/WARN, błędy mapowane na struktury JSON, brak stack trace w odpowiedzi REST.

## Testing - Strategia i Lokalizacje

### Poziomy testów
Zgodnie z **plan/PRD-002-testing-strategy.md**, projekt używa 4 poziomów testów:

#### 1. Unit Tests (w kodzie modułów)
**Lokalizacja:** `services/api/src/**/*_test.rs` lub inline `#[cfg(test)]`
**Uruchomienie:**
```bash
cd services/api
cargo test --lib --bins
```
**Cel:** Testowanie pojedynczych funkcji, logiki biznesowej w izolacji.
**Przykład:** Testy parsowania konfiguracji, validacja DTO, hashowanie haseł.

#### 2. Integration Tests (z bazą danych)
**Lokalizacja:** `services/api/tests/*.rs`
**Pliki:**
- `integration_test.rs` - testy auth flow (register, login, logout)
- `health_test.rs` - testy health endpoints

**Uruchomienie:**
```bash
# Start PostgreSQL
docker-compose up -d postgres

# Run tests
cd services/api
export DATABASE_URL="postgres://rapidfab:rapidfab-dev@localhost:5432/rapidfab"
cargo test --test integration_test -- --test-threads=1
```

**Cel:** Testowanie integracji z PostgreSQL, sqlx queries, migracje.
**Pokrycie:** Auth flow, DB persistence, session management.

#### 3. Contract Tests (API contracts)
**Lokalizacja:** `tests/contracts/` (TODO - planned for M1)
**Cel:** Weryfikacja kontraktów API między serwisami (API ↔ Pricing FDM).

#### 4. E2E Tests (full stack)
**Lokalizacja:** `tests/e2e/auth_flow_test.sh`
**Uruchomienie:**
```bash
# Start full stack
docker-compose up -d

# Run E2E tests
./tests/e2e/auth_flow_test.sh
```

**Testowane scenariusze:**
1. Health check (`/health/healthz`)
2. User registration (`POST /auth/register`)
3. Get profile with auth (`GET /users/me`)
4. Logout (`POST /auth/logout`)
5. Access denied after logout (401)
6. Login with credentials (`POST /auth/login`)

**Output:**
```
=== E2E Test: Auth Flow ===
Test 1: Health check... ✅ PASS
Test 2: Register user... ✅ PASS
Test 3: Get user profile... ✅ PASS
Test 4: Logout... ✅ PASS
Test 5: Profile access after logout... ✅ PASS
Test 6: Login with credentials... ✅ PASS
```

### Uruchomienie wszystkich testów

#### Lokalne (bez Docker)
```bash
# Format + Lint
cd services/api
cargo fmt --check
cargo clippy --all-targets -- -D warnings

# Unit tests
cargo test --lib --bins

# Integration (wymaga DB)
docker-compose up -d postgres
export DATABASE_URL="postgres://rapidfab:rapidfab-dev@localhost:5432/rapidfab"
cargo test --test '*' -- --test-threads=1
```

#### Docker (full stack)
```bash
# Start all services
docker-compose up -d

# Wait for API
sleep 10

# Run E2E
./tests/e2e/auth_flow_test.sh
```

#### Makefile (root)
```bash
make test-unit          # Unit tests
make test-integration   # Integration tests
make test-all           # All tests
make test-pipeline      # Full E2E pipeline (Docker containers)
```

### Test Pipeline (Prod-like Container Testing)

**Cel:** Testowanie całej usługi jako kontenery Docker (prod-like environment).

**Skrypt:** `scripts/test-pipeline.sh`

**Kroki:**
1. **Cleanup** - `docker-compose down -v --remove-orphans`
2. **Build** - Kompilacja obrazów Docker (compilation pipeline)
3. **Deploy** - `docker-compose up -d` (deployment pipeline)
4. **Health checks** - Czekanie na gotowość API
5. **E2E tests** - Uruchomienie wszystkich testów z `tests/e2e/*.sh`
6. **Cleanup** - Posprzątanie kontenerów i volumes

**Uruchomienie:**
```bash
# Pełny pipeline (build + deploy + test + cleanup)
make test-pipeline

# Alternatywnie bezpośrednio
./scripts/test-pipeline.sh
```

**Output:**
```
=== RapidFab Testing Pipeline ===
[1/5] Cleanup previous containers... ✓
[2/5] Building Docker images (compilation pipeline)... ✓
[3/5] Starting services (deployment pipeline)... ✓
[4/5] Running E2E tests...
  Running: auth_flow_test.sh ✓ PASSED
[5/5] Cleanup... ✓

=== Test Results ===
Passed: 1
Failed: 0
✓ All tests passed!
```

**Kiedy używać:**
- Po zakończeniu pracy agenta (weryfikacja przed commit)
- Przed git push (upewnienie się że deployment działa)
- Przy dodawaniu nowych testów E2E (automatycznie wykrywa `tests/e2e/*.sh`)
- Debugging problemów z docker-compose

**Dodawanie nowych testów:**
Wystarczy dodać plik `tests/e2e/new_test.sh` - script automatycznie go znajdzie i uruchomi.

### CI/CD Pipeline
**Lokalizacja:** `.github/workflows/ci.yml`
**Jobs:**
1. **fmt** - Format check (`cargo fmt --check`)
2. **clippy** - Linting (`cargo clippy -D warnings`)
3. **unit-tests** - Unit tests
4. **integration-tests** - Integration tests (z PostgreSQL service)
5. **build** - Release build
6. **docker-build** - Docker image build

**Trigger:** Push/PR na `main` lub `develop`
**Status:** https://github.com/Pondiniii/rapidfab/actions

### Dokumentacja testów
- **Strategia:** `plan/PRD-002-testing-strategy.md`
- **Coverage:** `services/api/docs/TEST_NOTES.md`
- **Smoke test report:** `services/api/docs/SMOKE_TEST_REPORT.md`

### Zasady testowania dla agentów
1. **Każdy feature = testy** - Minimum 1 test automatyczny.
2. **Test first dla edge cases** - Najpierw test, potem fix.
3. **Cleanup po testach** - Testy muszą czyścić dane w DB.
4. **Fixtures minimalne** - Używaj `tests/fixtures.rs` dla setup.
5. **Determinizm** - Testy muszą być powtarzalne (brak rand(), timestamps fixed).
6. **Naming convention:** `test_<scenario>_<expected_outcome>`

### Quick Test Commands (cheat sheet)

```bash
# Quick smoke test (2 min)
cd services/api && cargo check && cargo clippy

# Full local test (5 min)
docker-compose up -d postgres
cd services/api && cargo test

# Full E2E test (10 min)
docker-compose up -d
./tests/e2e/auth_flow_test.sh

# Full pipeline test - RECOMMENDED (prod-like, 3-5 min)
make test-pipeline
# Testuje: compilation + deployment + E2E + cleanup

# Watch mode (development)
cd services/api
cargo watch -x test
```

### Debugging testów
```bash
# Verbose output
RUST_LOG=debug cargo test -- --nocapture

# Single test
cargo test test_auth_flow -- --nocapture

# Show stdout even for passing tests
cargo test -- --show-output

# E2E debug
BASE_URL=http://localhost:8080 ./tests/e2e/auth_flow_test.sh
```  
