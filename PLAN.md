# RapidFab.xyz — plan (spis treści)

Ten dokument to indeks decyzji i planów. Każdy wpis opisuje „Co robimy” i „Dlaczego” w osobnym pliku, aby agenci mogli szybko znaleźć szczegóły bez scrollowania dużej ściany tekstu.

## Sekcje
- [PRD 001: Filozofia programowania](plan/PRD-001-filozofia-programowania.md)  
- [ADR 001: Wybór bazy danych PostgreSQL](plan/ADR-001-wybor-bazy-postgresql.md)  
- [ADR 002: Wybór stosu technologicznego](plan/ADR-002-tech-stack.md)  
- [ADR 003: Architektura webservera Axum](plan/ADR-003-webserver-axum.md)  
- [ADR 004: Skalowanie / Kubernetes-ready](plan/ADR-004-skalowanie-kubernetes.md) *(Proposed)*  
- [ADR 005: Mikroserwisy wyceny](plan/ADR-005-pricing-microservices.md) *(Proposed)*  
- [ADR 006: Email service + docker-mail-server](plan/ADR-006-email-service.md) *(Proposed)*  
- [PRD 002: Strategia testów](plan/PRD-002-testing-strategy.md) *(Proposed)*  
- [ADR 007: Struktura katalogów](plan/ADR-007-project-structure.md) *(Proposed)*  
- [ADR 008: Logging i observability (Loki Stack)](plan/ADR-008-logging-observability.md) *(Accepted)*  
- [ADR 009: Upload service — bezpieczeństwo anonimów](plan/ADR-009-upload-service.md) *(Proposed)*  
- [Szablon ADR](plan/ADR-template.md)

## Organizacja dokumentów
- Wszystkie szczegóły leżą w katalogu `plan/`. Gdy tylko uzyskamy możliwość zapisu w `.claude/plan/`, przeniesiemy pliki tam, aby zachować spójność z namingiem.  
- Każda nowa decyzja produktowa/architektoniczna dostaje osobny ADR/PRD według szablonu i jest linkowana z tego indeksu.

# RapidFab.xyz — Etapy i checklisty implementacji

Dokument porządkuje prace według kamieni milowych z `CLAUDE.md` i akceptowanych ADR/PRD. Każdy etap kończy się przejściowymi kryteriami „Definition of Done”, które muszą potwierdzić coding-agent i test-agent (pipeline opisany w `plan/PRD-002-testing-strategy.md`).

Legend:
- ✅ = wymagane na danym etapie  
- ⏩ = opcjonalne, przygotowanie do kolejnego etapu  
- 📄 = aktualizacja dokumentacji (`plan/`, `services/*/docs/`, `PLAN_STAGES.md`)

---

## M0 — Skeleton + Observability baseline

### Struktura repo (`plans/ADR-007`, `ADR-003`)
- [x] ✅ Utworzyć layout `services/<service>` z wymaganymi artefaktami (`Containerfile`, `Makefile`, `README.md`, `docs/INDEX.md`, `tests/`).
- [x] ✅ Dodać root `Makefile` (`test-unit`, `test-integration`, `test-contract`, `test-e2e`, `test-all`) oraz `docker-compose.yml` uruchamiający wszystkie kontenery.
- [ ] ✅ Przygotować szablon serwisu `services/_template/` z gotowymi plikami do kopiowania przez agentów.
- [x] 📄 Zaktualizować `services/api/docs/INDEX.md`, AUTH.md, DATABASE.md, USERS.md po zamknięciu checklisty.

### API Axum (baseline)
- [x] ✅ `services/api/` — Axum skeleton z modułami `app/auth`, `app/users`.
- [ ] ✅ Endpointy `/healthz`, `/readyz`, `/metrics` (Prometheus).
- [ ] ✅ Integracja z Postgres (połączenie, migracje stub) i storage S3 stub (konfiguracja env).
- [x] ✅ Logging `tracing` (JSON do stdout) zgodny z `ADR-008`.
- [x] ✅ Testy: unit (konfiguracja), integration (check auth flow).
- [x] 📄 Uzupełnić `services/api/docs/` — INDEX.md, AUTH.md, DATABASE.md, USERS.md, ARCHITECTURE.md.

### Infra i CI
- [ ] ✅ `infra/docker/` — docker-compose obsługujący Postgres, Redis (stub), docker-mail-server, Loki stack (wg `ADR-008`), OrcaSlicer container (stub).  
- [ ] ✅ `.github/workflows/ci.yml` — joby uruchamiające targety Makefile (unit/integration/contract/e2e).  
- [ ] ⏩ Manifesty w `infra/k8s/` (Deployment/Service dla `api`, `pricing-fdm`, `email-service`) z placeholderami env.  
- [ ] 📄 Wpis o CI i compose w `plan/ADR-004-skalowanie-kubernetes.md` i `docs/`.

### Observability (zgodnie z `ADR-008`)
- [ ] ✅ Dodać konfiguracje Loki/Promtail/Prometheus/Grafana w `infra/docker/`.  
- [ ] ✅ W `services/api/Makefile` target `make run-observability` (uruchamia stack).  
- [ ] ✅ Dodać metryki do Axum (`services/api/src/metrics.rs`).  
- [ ] ⏩ Przygotować starter dashboard JSON w `infra/grafana/`.

### Definition of Done M0
- [ ] Wszystkie powyższe zadania oznaczone ✅ ukończone i zatwierdzone przez test-agenta.  
- [ ] CI przechodzi (brak failing jobs).  
- [ ] 📄 Zaktualizowany `PLAN_STAGES.md` (oznaczenie checklist) + notatka w `CLAUDE.md` o zakończeniu M0.

---

## M1 — Pricing FDM (OrcaSlicer) + Upload flow

### Upload + storage
- [x] ✅ `services/api/` — endpoint `POST /files` generujący Signed URL (Hetzner S3), walidacja metadanych, zapis rekordu w Postgres.
- [x] ✅ Testy integration (mock S3) + kontrakty (`tests/contracts/files`).

### Upload Service (`services/upload/`)
- [x] ✅ Created upload microservice (Rust/Axum)
- [x] ✅ JWT ticket validation (HS256)
- [x] ✅ S3 client with presigned URLs (Hetzner compatible)
- [x] ✅ Quota system (anon: 100MB/day + 500MB/IP, user: 20GB + 2GB/hour)
- [x] ✅ Database migrations (uploads, files, quotas, ip_quotas)
- [x] ✅ 5 endpoints:
  - POST /internal/upload/init
  - POST /internal/upload/{id}/signed-urls
  - POST /internal/upload/{id}/confirm
  - POST /internal/upload/transfer
  - GET /internal/upload/file/{id}/read-url
- [x] ✅ API integration (proxy endpoints + JWT ticket generation)
- [x] ✅ Docker integration (minimal compose stack)
- [x] ✅ Prometheus metrics ready
- [x] ✅ CI passing (format, lint, Docker, E2E health checks)

### Pricing FDM mikroserwis (`plan/ADR-005`)
- [ ] ✅ `services/pricing-fdm/` — FastAPI/Flask wrapper na kontenerze OrcaSlicer.
- [ ] ✅ Endpoint `POST /quotes` przyjmujący `file_id`, parametry (materiał, infill, layer height).
- [ ] ✅ Skrypt w kontenerze uruchamia OrcaSlicer CLI i zwraca koszt + metryki (czas druku, zużycie).
- [ ] ✅ `Makefile` targety (lint/test) + testy unit (parsowanie wyników) i integration (mock pliku).
- [ ] 📄 Dokumentacja kontraktu w `services/pricing-fdm/docs/INDEX.md`.

### Koordynacja Axum ↔ pricing
- [ ] ✅ Endpoint `POST /quotes` w `api` (synchronicznie woła `pricing-fdm`, fallback na `spawn_blocking` jeśli brak kolejki).
- [ ] ✅ Persistencja: tabela `quotes` z historią wyników.
- [ ] ✅ Testy integration (mock pricing service).
- [ ] ⏩ Przygotować funkcję delegującą do kolejki (`todo!()`) zgodnie z decyzją w M2.

### Pipeline agentów i testy
- [x] ✅ Upload service: unit tests + integration tests + E2E health check
- [ ] ✅ Pricing service tests + E2E quote flow
- [x] ✅ CI pipeline (42s, all passing)

### Definition of Done M1
- [x] Upload service complete ✅
- [ ] Pricing FDM service implemented
- [ ] Upload + pricing flow działa end-to-end w docker-compose.
- [ ] OrcaSlicer kontener uruchamia się bez manualnej ingerencji.
- [ ] CI przechodzi (w tym testy e2e).
- [ ] 📄 Aktualizacja `PLAN_STAGES.md`, `CLAUDE.md` (zamknięcie M1).

---

## M2 — Zamówienia + Stripe + Email service

### Zamówienia
- [ ] ✅ `services/api/` — modele `orders`, `order_items`, walidacja parametrów, statusy.  
- [ ] ✅ Endpoint `POST /orders` (tworzenie) + `GET /orders/{id}` (status).  
- [ ] ✅ Testy integration, migracje SQL.

### Stripe integracja
- [ ] ✅ Konfiguracja Stripe Checkout (`/payments/session`) + webhook handler w `api`.  
- [ ] ✅ Testy integration (mock Stripe), kontrakty (payload webhook).  
- [ ] ⏩ Stub płatności offline (fallback, np. testowe `payment_provider=manual`).

### Email service (`plan/ADR-006`)
- [ ] ✅ `services/email-service/` — HTTP `POST /send`, templating, SMTP docker-mail-server.  
- [ ] ✅ Kolejka retry (synchronizacja z decyzją o brokerze).  
- [ ] ✅ Testy unit (rendering), integration (SMTP stub).  
- [ ] ✅ Logging + metryki `email_sent_total`.

### Orkiestracja zdarzeń
- [ ] ✅ Wybrany broker (`Redis Streams` lub `NATS`) opisany w nowym ADR (schemat eventów).  
- [ ] ✅ `api` publikuje `quote.requested`, `order.created`.  
- [ ] ✅ `pricing-fdm` konsumuje `quote.requested` (asynchroniczny flow — API zwraca `202 Accepted`).  
- [ ] ✅ `email-service` konsumuje `email.requested` (np. link rejestracyjny).  
- [ ] ✅ Aktualizacja `tests/contracts/` pod eventy.

### Definition of Done M2
- [ ] Async pricing działa (kolejka, worker).  
- [ ] Zamówienie i płatności end-to-end przetestowane w `tests/e2e/`.  
- [ ] Email service wysyła maile w docker-compose (Mailhog preview).  
- [ ] 📄 Update `CLAUDE.md`, `PLAN_STAGES.md`.

---

## M3 — Panel operacyjny i operacje

- [ ] ✅ `api` — endpointy admin (`GET /admin/orders`, `GET /admin/quotes`).  
- [ ] ✅ Role/permissions (admin vs user).  
- [ ] ✅ Raporty SLA, metryki biznesowe (eksport do `/metrics`).  
- [ ] ✅ Alerty Prometheus/Grafana (wg `ADR-008`).  
- [ ] ⏩ Integracja z monitoringiem Slack/email.

DoD: dashboardy gotowe, alerty działają, API ma auth dla panelu, testy e2e obejmują scenariusze admina.

---

## M4 — Frontend + optymalizacje

- [ ] ✅ Frontend Svelte/Tailwind (SSR) korzystający z publicznego API.  
- [ ] ✅ Performance tuning (profiling, caching).  
- [ ] ✅ Autoscaling manifesty K8s (HPA).  
- [ ] ✅ Cleanup zadłużeń technicznych, dokumentacja końcowa.

---

## Utrzymanie dokumentacji
- Po każdym etapie oznaczamy wykonane pozycje w `PLAN_STAGES.md` (commit z aktualizacją).  
- Jeśli pojawi się nowy komponent lub zmiana kontraktu, dopisujemy nowe checkboxy zgodnie z obowiązującymi ADR/PRD i aktualizujemy sekcje `Definition of Done`.  
- Każdy większy etap kończy się krótką notką w `CLAUDE.md` („M1 complete — upload+pricing live”).  
- W trakcie prac agenci dopisują detale w `services/<service>/docs/work/*.md`, a po domknięciu zadania migrują najważniejsze wnioski do `docs/INDEX.md`.
- **Automatyzowalne zadania**: skrypty `make`/`invoke` z parametrami zrozumiałymi dla agentów.
