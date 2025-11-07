# PRD 002: Strategia testów RapidFab.xyz

## Kontekst
System składa się z wielu mikroserwisów (Axum API, pricingi, e-mail). Każdy komponent ma być niezależny, ale jednocześnie musimy mieć absolutną pewność, że kontrakty między serwisami są dotrzymane. Potrzebujemy metodyki testów, którą da się uruchomić automatycznie przez agentów (coding/test) i która wykrywa regresje zanim trafią na produkcję.

## Cele
- Szybkie testy jednostkowe dla każdej zmiany (feedback < 10s per moduł).  
- Testy integracyjne, które łapią regresje w kontraktach REST/event tuż po zmianie.  
- E2E, które sprawdzają rzeczywiste scenariusze „upload → wycena → zamówienie → e-mail”.  
- Automatyzacja: `make test-unit`, `make test-contract`, `make test-e2e` uruchamiane przez coding-agent/test-agent oraz w CI.  
- Repeatability: testy da się odpalić lokalnie albo w pipeline bez ręcznej konfiguracji.

## Struktura katalogów testów

```
services/
  api/                         # Axum backend (Rust)
    src/
    tests/                     # moduły Rust (integration/component)
    Makefile                   # targety fmt/lint/test dla serwisu
  pricing-fdm/                 # Python (OrcaSlicer wrapper)
    app/
    tests/unit/                # pytest unit
    tests/integration/         # pytest component z mockami
    Makefile
  email-service/
    app/
    tests/
    Makefile
  (kolejne serwisy analogicznie)
tests/
  contracts/                   # consumer-driven contracts (JSON Schema + testy)
  e2e/                         # scenariusze systemowe (requests/Playwright/k6)
  fixtures/                    # sample CAD/JSON/mail do testów
```

- Każdy mikroserwis ma własny katalog `tests/` oraz lokalny `Makefile`, który udostępnia co najmniej targety `fmt`, `lint`, `test`.  
- W katalogu głównym `tests/` trzymamy testy wspólne: kontraktowe i e2e uruchamiane na `docker-compose`.  
- `tests/fixtures/` zawiera przykładowe pliki współdzielone przez wszystkie scenariusze.

## Typy testów

### 1. Unit
- Rust: `cargo test` w module (logika domenowa, walidatory, repozytoria z in-memory DB).  
- Python: `pytest tests/unit` (funkcje pricingowe, template’y e-mail).  
- Brak I/O, zero zależności zewnętrznych (mocki/fixture’y).

### 2. Integration / Component
- Rust: `cargo test --test <name>` na routerach Axum (z mock connection do Postgresa i S3).  
- Python: `pytest tests/integration` z fixture’ami uruchamiającymi serwis w `TestClient`, z mockiem storage/kolejki.  
- Sprawdzamy poprawność endpointów, serializacji, obsługę błędów, logging.

### 3. Contract tests
- OpenAPI generowane z Axum i serwisów pricingowych/e-mailowych.  
- `tests/contracts/` zawiera:
  - JSON Schemy request/response/eventów.  
  - Testy (Rust/Python) weryfikujące, że oba serwisy spełniają kontrakt (consumer-driven).  
- Przy zmianie kontraktu agent aktualizuje schema i uruchamia testy konsumentów.

### 4. System / E2E
- `tests/e2e/` ma skrypty Playwright/requests/k6.  
- `make test-e2e` odpala `docker-compose.yml` (API + pricing FDM + email + Postgres + Redis + Mail server + observability stuby), następnie uruchamia scenariusze:
  1. Upload nowego pliku → wycena FDM → zamówienie → webhook Stripe (stub) → e-mail.  
  2. Ponowna wycena istniejącego pliku (bez ponownego uploadu).  
  3. Failure path (pricing zwraca `quote.failed`, API słusznie raportuje status).

### 5. Load / resilience (opcjonalnie)
- Skrypty `tests/e2e/load/` (np. `k6`).  
- Sprawdzają SLA (p95, throttle), retry kolejki, zachowanie przy awarii microserwisu.

## Automatyzacja
- Globalny `Makefile` (root):
  - `make test-unit` → deleguje do `services/*/Makefile test-unit` (Rust `cargo test`, Python `pytest tests/unit`).  
  - `make test-integration` → odpala testy komponentowe w każdym serwisie.  
  - `make test-contract` → generuje OpenAPI/Schema + weryfikuje kontrakty (`tests/contracts`).  
  - `make test-e2e` → `docker-compose up --build` + scenariusze `tests/e2e`.  
  - `make test-all` → `test-unit + test-integration + test-contract + test-e2e`.  
- Każdy serwis ma lokalny `Makefile` z aliasami (np. `make test`, `make fmt`). Globalne cele korzystają z nich automatycznie.  
- Pipeline agentów:
  1. **coding-agent**: przed oddaniem pracy uruchamia `make test-unit` (minimum) oraz wszystkie testy dotyczące modyfikowanego serwisu (`services/<nazwa>/Makefile test`). Wyniki dopisuje do notatek.  
  2. **test-agent**: uruchamia `make test-all`, weryfikuje logi, kontrakty i scenariusze e2e, a także jakość kodu i zgodność z wymaganiami.  
- CI (GitHub Actions lub inny runner) powiela dokładnie ten podział jobów (`unit`, `integration`, `contract`, `e2e`), by odzwierciedlać manualny pipeline agentów.

## Konsekwencje
+ Szybki feedback dla pojedynczych modułów.  
+ Gwarancja, że kontrakty Axum ↔ mikroserwisy nie rozjadą się w ciszy.  
+ Jedno miejsce (`tests/`) na scenariusze end-to-end i load.  
– Dłuższy pipeline (konieczność spinania unit + integracja + e2e).  
– Wymaga utrzymywania mocków i fixture’ów w synchronizacji z kodem.

## Status
Accepted
