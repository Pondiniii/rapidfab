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

---

## 🚨 TESTING - ABSOLUTNA ZASADA DLA AGENTÓW 🚨

### TL;DR - PRZECZYTAJ TO NAJPIERW

**Po KAŻDEJ zmianie kodu - BEZ WYJĄTKÓW:**

```bash
task ci
```

**KONIEC. To wszystko. Jedna komenda.**

### Zasady (ZERO kompromisów):

1. **ZAWSZE** uruchom `task ci` po zakończeniu pracy
2. **NIGDY** nie commituj jeśli `task ci` pokazuje błąd (❌)
3. **TYLKO** commituj gdy widzisz `✅ CI passed`
4. Jeśli coś failuje → napraw → `task ci` → powtórz

### Co testuje `task ci` (42 sekundy):

```
🚀 Running CI...
  ├─ Format check (cargo fmt)
  ├─ Linter (cargo clippy -D warnings)
  ├─ Unit tests
  ├─ Docker build (with cache)
  ├─ Docker deploy + health checks
  ├─ E2E tests (auto-discovery)
  └─ Cleanup
✅ CI passed
```

### Output (silent mode):

**Sukces (3 linie):**
```
🚀 Running CI...
Failed: 0
✅ CI passed
```

**Fail (pokazuje tylko błędy):**
```
🚀 Running CI...
error[E0308]: mismatched types
  --> src/main.rs:42:5
❌ Clippy failed
```

### Dlaczego to jest ważne?

- **Jeden command** = wszystko przetestowane (fmt, lint, unit, Docker, E2E)
- **42 sekundy** = szybki feedback loop
- **Silent mode** = zero spamu, tylko błędy
- **Auto-discovery** = nowe testy automatycznie wykrywane
- **Prod-like** = testuje Docker containers, nie native code

### Kiedy NIE używać `task ci`:

NIGDY. Zawsze używaj `task ci`.

### Przykładowy workflow:

```bash
# 1. Agent implementuje feature
vim src/app/my_feature.rs

# 2. NATYCHMIAST po zmianach
task ci

# 3a. Jeśli ✅ CI passed
git add .
git commit -m "feat: add my feature"
git push

# 3b. Jeśli ❌ failed
# Fix błąd...
task ci  # Powtórz aż ✅
```

### Dokumentacja szczegółowa:

Jeśli potrzebujesz więcej info → `tests/CLAUDE.md`

**ALE pamiętaj: 99% czasu potrzebujesz tylko `task ci`.**

---

## 🎯 TASK BATCHING - EFEKTYWNOŚĆ CONTEXT DLA AGENTÓW

### TL;DR

**Grupuj powiązane taski razem - context restore jest kosztowny!**

### Dlaczego batching?

**PROBLEM:** Każde wywołanie coding-agent = nowy context. Agent musi:
- Przeczytać strukturę projektu
- Zrozumieć zależności
- Załadować mental model

**KOSZT:** ~30-60s overhead + tokeny na każdy context switch

**ROZWIĄZANIE:** Grupuj 3-5 logicznie powiązanych tasków w jeden batch.

### Zasady grupowania

**✅ DOBRZE (batch):**
```
Batch: "Implement S3 client + presigned URLs"
Tasks:
1. Create storage/s3_client.rs with S3Client struct
2. Implement generate_presigned_put_url()
3. Implement generate_presigned_get_url()
4. Add unit tests for URL generation
5. Update config.rs with S3 settings
```

**❌ ŹLE (pojedynczo):**
```
Task 1: Create storage/s3_client.rs
[agent runs, exits]
Task 2: Implement generate_presigned_put_url()
[agent runs, exits - musi ponownie czytać s3_client.rs]
Task 3: Add unit tests
[agent runs, exits - musi znowu czytać kod]
```

### Kryteria batching

Grupuj tasks jeśli mają:
- **Wspólny plik/moduł** (np. wszystko w `storage/`)
- **Wspólną domenę** (np. quota system: checker + DB + tests)
- **Zależności sekwencyjne** (np. model → repository → endpoint)
- **Wspólny test scope** (np. auth flow: login + logout + middleware + tests)

**NIE** grupuj jeśli:
- Tasks dotyczą różnych serwisów
- Wymagają różnych agentów (coding vs senior-api-developer)
- Są niezależne i mogą być parallel

### Przykłady z projektu

**Batch 1: Upload ticket validation + config**
```
1. Create src/auth/ticket.rs - validate JWT
2. Add UPLOAD_TICKET_SECRET to config.rs
3. Update .env.example with ticket settings
4. Write unit tests for ticket validation
```
→ Wszystko w jednym contexcie, agent rozumie flow od początku do końca.

**Batch 2: Quota system complete**
```
1. Create storage/quota.rs - quota checker logic
2. Update DB migrations with quota tables
3. Add Redis rate limiter integration
4. Implement metrics (upload_rate_limit_hits_total)
5. Write integration tests for quota enforcement
```
→ Cały quota system w jednym sesji, łatwiej zapewnić spójność.

### Metryki

**Pojedyncze taski:**
- 5 tasks × (60s context + 120s work) = 15 minut
- 5 × context overhead = marnowanie zasobów

**Batched (5 tasks):**
- 1 × (60s context + 600s work) = 11 minut
- Oszczędność: ~25% czasu + mniej tokenów

### Workflow dla agentów

Gdy dostajesz liste tasków:
1. **Zgrupuj** po module/domenie
2. **Zweryfikuj** zależności (co musi być pierwsze)
3. **Wykonaj batch** jako jedną sesję
4. **Uruchom `task ci`** po całym batchu (nie po każdym tasku)

---

## Styl kodowania
- **Less is more**: preferuj krótkie moduły Rust (< 300 linii); jeśli rośnie → rozbij na podmoduły.
- **SOLID / DRY / KISS**: brak powtórzeń, proste nazewnictwo (`verb_subject`), jawne interfejsy traits.
- **Czytelność > „spryt"**: unikamy makr, dopóki nie poprawiają czytelności; dependency injection przez konstruktor/trait.
- **Konwencja**: `cargo fmt`, `cargo clippy -D warnings`, testy (`cargo test`). W Python workerach `ruff` + `pytest` jeśli są użyte.
- **Logging i błędy**: `tracing` z poziomami INFO/WARN, błędy mapowane na struktury JSON, brak stack trace w odpowiedzi REST.

## Testing - Lokalizacje (dla referencji)

**UŻYWAJ `task ci` - nie uruchamiaj testów ręcznie!**

### Gdzie są testy:

1. **Unit tests**: `services/api/src/**/*_test.rs` (inline w kodzie)
2. **Integration tests**: `services/api/tests/*.rs` (integration_test.rs, health_test.rs, security_test.rs)
3. **E2E tests**: `tests/e2e/*_test.sh` (auto-discovery, bash scripts)

### Dodawanie nowych testów E2E:

```bash
# 1. Utwórz plik w tests/e2e/
touch tests/e2e/my_feature_test.sh
chmod +x tests/e2e/my_feature_test.sh

# 2. Napisz test (bash + curl)
cat > tests/e2e/my_feature_test.sh <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

# Test logic
curl -sf http://localhost:8080/api/endpoint || exit 1
echo "✅ Test passed"
EOF

# 3. Run CI (auto-discovers new test!)
task ci
```

**Zero konfiguracji - file convention: `tests/e2e/*_test.sh` + executable.**

---

### Inne komendy (dla advanced use cases):

**99% czasu NIE potrzebujesz tych komend - używaj `task ci`!**

```bash
# Debugging pojedynczego testu
task test:e2e          # Tylko E2E tests
task test:unit         # Tylko unit tests
task fmt               # Tylko format check
task lint              # Tylko linter

# Development watch mode (continuous testing)
cd services/api && cargo watch -x test

# Verbose test output (debugging)
cd services/api && RUST_LOG=debug cargo test -- --nocapture
```

### Dokumentacja szczegółowa:

- **Pełna dokumentacja testowania:** `tests/CLAUDE.md`
- **Strategia testów:** `plan/PRD-002-testing-strategy.md`

---

## Recent Milestones

### 2025-11-08: Upload Service Complete
- Implemented upload microservice (services/upload/)
- 5 endpoints, quota system, S3 integration
- ADR-009 accepted and deployed
- CI passing, Docker ready
- Next: Pricing FDM service (M1)

---
