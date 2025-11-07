# ADR 008: Logging i observability (Loki Stack)

## Kontekst
RapidFab.xyz składa się z mikroserwisów (Axum API, pricing-fdm, pricing-sls, email-service). Każdy serwis generuje logi i metryki. Potrzebujemy:
- **Centralizacji logów** z wszystkich kontenerów w jednym miejscu
- **Metryk** (request rate, latency, errors, business metrics)
- **Wizualizacji** (dashboardy Grafana z wykresami + logami)
- **Alertów** (Slack/email gdy błędy > threshold)
- **Self-hosted** (brak SaaS, pełna kontrola, niskie koszty)
- **Łatwości implementacji** (MVP w < 1 dzień, zero vendor lock-in)

Rozważaliśmy 3 rozwiązania:
1. **Loki Stack** (Loki + Promtail + Prometheus + Grafana) — oficjalny stack Grafany
2. **OpenTelemetry** (OTEL + Tempo + Loki + Prometheus) — distributed tracing, złożone
3. **Vector + ClickHouse** — minimalistyczne, niszowe

## Decyzja
Implementujemy **Loki Stack** jako baseline observability dla MVP:
- **Loki**: baza logów (time-series, label-based indexing)
- **Promtail**: agent zbierający logi z kontenerów Docker (auto-discovery)
- **Prometheus**: baza metryk (scraping `/metrics` z serwisów co 15s)
- **Grafana**: unified UI (logi + metryki + dashboardy + alerty)

Serwisy logują strukturalnie (JSON) do stdout, Promtail czyta z Docker socket, Prometheus scrape'uje `/metrics` endpoints.

## Uzasadnienie
- **Oficjalny stack Grafana** — dobrze zintegrowany, stabilny, popularny (łatwo znaleźć pomoc).
- **Proste setup** — docker-compose + 3 pliki konfiguracyjne, działa out-of-the-box.
- **Loki jest lekki** — brak full-text indexing (tylko labels), mniejsze zużycie RAM/storage niż Elasticsearch.
- **Jedno UI** — Grafana do wszystkiego (logi, metryki, dashboardy, alerty).
- **Auto-discovery** — Promtail wykrywa kontenery po Docker labels, zero ręcznej konfiguracji.
- **Darmowy i open-source** — pełna kontrola, brak kosztów licencji.
- **Wystarczające dla MVP → Faza 2** — skaluje do 1M requestów/dzień bez problemu.

## Architektura

```
┌──────────────────┐
│  Axum API        │──┐ stdout (JSON logs)
│  (tracing-json)  │  │ /metrics (Prometheus format)
└──────────────────┘  │
                      │
┌──────────────────┐  │
│  Pricing FDM     │──┤
│  (structlog)     │  │
└──────────────────┘  │
                      ├─→ Promtail ─────→ Loki ────┐
┌──────────────────┐  │                            │
│  Pricing SLS     │──┤                            │
│  (structlog)     │  │                            ├─→ Grafana (UI)
└──────────────────┘  │                            │
                      │   HTTP GET /metrics        │
┌──────────────────┐  │          ↓                 │
│  Email Service   │──┘      Prometheus ───────────┘
│  (logging)       │
└──────────────────┘
```

### Flow logów
1. Serwis pisze JSON do stdout: `{"timestamp": "...", "level": "info", "service": "api", "msg": "Request processed"}`
2. Promtail czyta stdout z Docker socket (filtruje po labels: `logging=promtail`)
3. Promtail parsuje JSON, dodaje labels (`service`, `level`), wysyła do Loki
4. Grafana query: `{service="api", level="error"}` → pokazuje błędy z API

### Flow metryk
1. Serwis wystawia `/metrics` endpoint (format Prometheus):
   ```
   http_requests_total{method="POST", endpoint="/quotes", status="200"} 1234
   http_request_duration_seconds_bucket{endpoint="/quotes", le="0.1"} 980
   ```
2. Prometheus scrape'uje co 15s (konfiguracja: `prometheus.yml`)
3. Grafana query: `rate(http_requests_total[5m])` → wykres requestów/sekundę

## Implementacja

### 1. Docker Compose

```yaml
# infra/docker/docker-compose.observability.yml
version: '3.8'

services:
  loki:
    image: grafana/loki:2.9.3
    container_name: loki
    ports:
      - "3100:3100"
    volumes:
      - ./loki-config.yml:/etc/loki/local-config.yaml
      - loki-data:/loki
    command: -config.file=/etc/loki/local-config.yaml
    restart: unless-stopped

  promtail:
    image: grafana/promtail:2.9.3
    container_name: promtail
    volumes:
      - ./promtail-config.yml:/etc/promtail/config.yml
      - /var/run/docker.sock:/var/run/docker.sock:ro
    command: -config.file=/etc/promtail/config.yml
    restart: unless-stopped
    depends_on:
      - loki

  prometheus:
    image: prom/prometheus:v2.48.0
    container_name: prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=15d'
    restart: unless-stopped

  grafana:
    image: grafana/grafana:10.2.2
    container_name: grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - ./grafana-datasources.yml:/etc/grafana/provisioning/datasources/datasources.yml
      - grafana-data:/var/lib/grafana
    restart: unless-stopped
    depends_on:
      - loki
      - prometheus

volumes:
  loki-data:
  prometheus-data:
  grafana-data:
```

### 2. Loki Config

```yaml
# infra/docker/loki-config.yml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    address: 127.0.0.1
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
    - from: 2023-01-01
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/index
    cache_location: /loki/cache
    shared_store: filesystem
  filesystem:
    directory: /loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h  # 7 dni

chunk_store_config:
  max_look_back_period: 168h  # retention 7 dni

table_manager:
  retention_deletes_enabled: true
  retention_period: 168h
```

### 3. Promtail Config

```yaml
# infra/docker/promtail-config.yml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  # Auto-discover Docker containers z label logging=promtail
  - job_name: docker
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      # Tylko kontenery z label logging=promtail
      - source_labels: ['__meta_docker_container_label_logging']
        regex: 'promtail'
        action: keep
      # Service name z Docker label
      - source_labels: ['__meta_docker_container_label_service']
        target_label: 'service'
      # Container name jako fallback
      - source_labels: ['__meta_docker_container_name']
        regex: '/(.*)'
        target_label: 'container'
    pipeline_stages:
      # Parse JSON logs
      - json:
          expressions:
            level: level
            timestamp: timestamp
            message: msg
      # Dodaj timestamp jako label (dla sortowania)
      - timestamp:
          source: timestamp
          format: RFC3339Nano
      # Dodaj level jako label
      - labels:
          level:
```

### 4. Prometheus Config

```yaml
# infra/docker/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  # Axum API
  - job_name: 'api'
    static_configs:
      - targets: ['api:8080']
    metrics_path: '/metrics'

  # Pricing services (Python FastAPI)
  - job_name: 'pricing-fdm'
    static_configs:
      - targets: ['pricing-fdm:8081']
    metrics_path: '/metrics'

  - job_name: 'pricing-sls'
    static_configs:
      - targets: ['pricing-sls:8082']
    metrics_path: '/metrics'

  # Email service
  - job_name: 'email'
    static_configs:
      - targets: ['email:8083']
    metrics_path: '/metrics'

  # Self-monitoring
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
```

### 5. Grafana Datasources

```yaml
# infra/docker/grafana-datasources.yml
apiVersion: 1

datasources:
  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
    isDefault: false
    jsonData:
      maxLines: 1000

  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    jsonData:
      timeInterval: 15s
```

### 6. Logowanie w serwisach

#### Axum (Rust)
```rust
// services/api/src/main.rs
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    // JSON logs do stdout
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .with_current_span(false)
        .init();

    tracing::info!(
        service = "api",
        version = env!("CARGO_PKG_VERSION"),
        "Starting RapidFab API"
    );
}

// W handlerach
tracing::info!(
    endpoint = "/quotes",
    method = "POST",
    status = 200,
    duration_ms = elapsed.as_millis(),
    "Request processed"
);
```

#### Python (Pricing services)
```python
# services/pricing-fdm/app/main.py
import structlog

structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.add_log_level,
        structlog.processors.JSONRenderer()
    ]
)

logger = structlog.get_logger()

logger.info(
    "quote_calculated",
    service="pricing-fdm",
    quote_id=str(quote_id),
    volume_cm3=volume,
    price_eur=price
)
```

#### Docker labels (w docker-compose.yml)
```yaml
services:
  api:
    labels:
      logging: "promtail"
      service: "api"

  pricing-fdm:
    labels:
      logging: "promtail"
      service: "pricing-fdm"
```

### 7. Metryki w serwisach

#### Axum (Rust)
```rust
// Cargo.toml
[dependencies]
prometheus = "0.13"
axum = { version = "0.7", features = ["prometheus"] }

// services/api/src/metrics.rs
use prometheus::{IntCounterVec, HistogramVec, Encoder, TextEncoder};
use once_cell::sync::Lazy;

pub static HTTP_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        opts!("http_requests_total", "Total HTTP requests"),
        &["method", "endpoint", "status"]
    ).unwrap()
});

pub static HTTP_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    HistogramVec::new(
        histogram_opts!("http_request_duration_seconds", "HTTP request latency"),
        &["method", "endpoint"]
    ).unwrap()
});

// Endpoint /metrics
async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode_to_string(&metric_families).unwrap()
}
```

#### Python (FastAPI)
```python
# requirements.txt: prometheus-fastapi-instrumentator

from prometheus_fastapi_instrumentator import Instrumentator

app = FastAPI()

# Auto-instrumentacja (request count, latency, errors)
Instrumentator().instrument(app).expose(app, endpoint="/metrics")

# Custom metrics
from prometheus_client import Counter, Histogram

quote_calculations = Counter(
    'quote_calculations_total',
    'Total quote calculations',
    ['technology', 'status']
)

quote_duration = Histogram(
    'quote_calculation_duration_seconds',
    'Quote calculation duration',
    ['technology']
)

# Użycie
with quote_duration.labels(technology='fdm').time():
    price = calculate_price(file)
quote_calculations.labels(technology='fdm', status='success').inc()
```

## Przykładowe query w Grafanie

### Logi (LogQL)
```logql
# Wszystkie błędy z API w ostatniej godzinie
{service="api", level="error"}

# Requesty do /quotes trwające > 100ms
{service="api"} |= "Request processed" | json | duration_ms > 100

# Errory z konkretnego quote_id
{service=~"pricing-.*"} | json | quote_id="abc-123-def"
```

### Metryki (PromQL)
```promql
# Request rate per service
rate(http_requests_total[5m])

# P95 latency per endpoint
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Error rate
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

# Quote calculations per technology
sum by (technology) (rate(quote_calculations_total[5m]))
```

## Dashboard starter

Przykładowy dashboard Grafana (JSON) w `infra/grafana-dashboard-starter.json`:
- Panel 1: Request rate (wykres liniowy, Prometheus)
- Panel 2: Latency P50/P95 (wykres liniowy, Prometheus)
- Panel 3: Error rate (wykres, alert jeśli > 1%)
- Panel 4: Last 100 errors (tabela, Loki query)
- Panel 5: Quote calculations by tech (bar chart)

Import: Grafana → Dashboards → Import → Upload JSON.

## Alerty (opcjonalne, Faza 2)

Prometheus Alert Manager + Grafana Alerting:
- Error rate > 1% przez 5 minut → Slack #alerts
- Latency P95 > 500ms przez 5 minut → Slack
- Brak heartbeat z serwisu przez 2 minuty → Slack

Config w `prometheus-alerts.yml`:
```yaml
groups:
  - name: rapidfab
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) > 0.01
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate: {{ $value | humanizePercentage }}"
```

## Konsekwencje

### Pozytywne
+ **Jedno UI** (Grafana) dla wszystkich potrzeb observability.
+ **Oficjalny stack** — stabilny, popularny, łatwo znaleźć help online.
+ **Proste setup** — działa po docker-compose up w < 5 minut.
+ **Auto-discovery** — Promtail wykrywa nowe kontenery automatycznie.
+ **Niskie koszty** — ~1.5 GB RAM, 5-10 GB storage/miesiąc.
+ **Skalowalne** — wystarczy do 1M requestów/dzień bez optymalizacji.
+ **Self-hosted** — zero vendor lock-in, pełna kontrola.

### Negatywne
– **4 kontenery** (Loki, Promtail, Prometheus, Grafana) — więcej niż minimalne rozwiązanie.
– **Brak distributed tracing** — trudniej debugować requesty przez wiele serwisów (mitigation: dodać trace_id do logów ręcznie).
– **Loki nie ma full-text search** — trzeba znać label (np. `service=api`), nie można szukać po dowolnym tekście.
– **Retention trzeba pilnować** — domyślnie 7 dni, potem trzeba czyścić ręcznie (albo zwiększyć storage).
– **Query language** (LogQL) wymaga nauki — prostsze niż PromQL, ale nie trywialne.

## Migracja na OTEL (przyszłość, opcjonalna)

Gdy będziemy mieć 5+ mikroserwisów i potrzebujemy distributed tracing:
1. Dodać OpenTelemetry Collector (routing traces → Tempo, logs → Loki)
2. Instrumentacja serwisów (otel SDK w Rust/Python)
3. Dodać Tempo (baza traces)
4. Correlation w Grafanie (trace_id → logi)

Loki + Prometheus pozostają, dodajemy tylko tracing layer. Inwestycja w Loki Stack nie jest stracona.

## Zasoby i koszty

### Zasoby
- **Loki**: 512 MB RAM, 2-5 GB storage (7 dni retention)
- **Promtail**: 128 MB RAM
- **Prometheus**: 512 MB RAM, 3-5 GB storage (15 dni retention)
- **Grafana**: 256 MB RAM, 100 MB storage (dashboardy)
- **Razem**: ~1.5 GB RAM, ~10 GB storage

### VPS recommendation
- Hetzner CX21 (2 vCPU, 4 GB RAM, 40 GB SSD) — €5.83/miesiąc
- Wystarczy na backend + observability + PostgreSQL + Redis

### Retention policy
- **Loki**: 7 dni (configurable w loki-config.yml)
- **Prometheus**: 15 dni (--storage.tsdb.retention.time flag)
- Starsze logi można archiwizować do S3 (opcjonalne, Faza 3)

## Status
**Accepted** — implementacja w Fazie 0 (baseline observability dla MVP)

## Następne kroki
1. Utworzyć `infra/docker/docker-compose.observability.yml` z konfiguracją wyżej
2. Dodać JSON logging do Axum (tracing-subscriber)
3. Dodać structlog do Python services
4. Dodać `/metrics` endpoint w każdym serwisie
5. Dodać Docker labels `logging=promtail` + `service=<name>`
6. Uruchomić stack: `docker-compose -f docker-compose.observability.yml up -d`
7. Zaimportować starter dashboard do Grafany
8. Przetestować: wygenerować ruch, sprawdzić logi + metryki w Grafanie
9. Ustawić alerty (opcjonalne, Faza 1+)

## Przykład użycia (developer workflow)

```bash
# Start observability stack
cd infra/docker
docker-compose -f docker-compose.observability.yml up -d

# Check logs (all services)
# Grafana → Explore → Loki → {service=~".*"}

# Check metrics (API latency)
# Grafana → Explore → Prometheus → histogram_quantile(0.95, rate(http_request_duration_seconds_bucket{service="api"}[5m]))

# Debug konkretny request (trace_id w logach)
# Loki query: {service=~".*"} | json | trace_id="abc-123"

# Alerty
# Grafana → Alerting → pokazuje aktywne alerty
```

## Referencje
- [Grafana Loki Documentation](https://grafana.com/docs/loki/latest/)
- [Promtail Configuration](https://grafana.com/docs/loki/latest/clients/promtail/configuration/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [tracing-subscriber (Rust)](https://docs.rs/tracing-subscriber/)
- [structlog (Python)](https://www.structlog.org/)
