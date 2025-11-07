# Infrastructure

## Services

- **PostgreSQL** (port 5432) - Primary database
- **Redis** (port 6379) - Queue/cache (stub for M2)
- **Loki** (port 3100) - Log aggregation
- **Promtail** - Log collection from Docker
- **Prometheus** (port 9090) - Metrics collection
- **Grafana** (port 3000) - Dashboards (admin/admin)
- **API** (port 8080) - RapidFab API

## Quick Start

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f api

# Check health
curl http://localhost:8080/health/healthz
curl http://localhost:8080/metrics

# Access dashboards
open http://localhost:3000  # Grafana (admin/admin)
open http://localhost:9090  # Prometheus
```

## Observability

### Logs (Loki)
- All API logs are JSON formatted
- Collected by Promtail from Docker labels
- View in Grafana: Explore → Loki → `{service="api"}`

### Metrics (Prometheus)
- API exposes `/metrics` endpoint
- Scraped every 10s
- View in Grafana: Explore → Prometheus

### Retention
- Logs: 31 days (Loki)
- Metrics: 15 days (Prometheus)

## Configuration Files

```
infra/docker/
├── loki/
│   └── loki-config.yml         # Loki server configuration
├── promtail/
│   └── promtail-config.yml     # Log collection rules
├── prometheus/
│   └── prometheus.yml          # Scrape targets
└── grafana/
    ├── datasources.yml         # Loki + Prometheus datasources
    ├── dashboards.yml          # Dashboard provisioning
    └── dashboards/
        └── rapidfab-overview.json  # Main dashboard
```

## Grafana Dashboard

The default dashboard includes:
- **HTTP Requests Total** - Request rate by method and endpoint
- **HTTP Request Duration (P95)** - 95th percentile response time (target: < 150ms)
- **Database Connections** - Active PostgreSQL connections
- **Error Rate (5xx)** - Server errors per second
- **Recent Logs** - Live log stream from API

## Troubleshooting

### Check service health
```bash
docker-compose ps
```

All services should show "healthy" status.

### View Loki logs
```bash
docker-compose logs loki
```

### Reload Prometheus config
```bash
curl -X POST http://localhost:9090/-/reload
```

### Check Promtail targets
```bash
curl http://localhost:9080/targets
```

## Development

### Adding new metrics
1. Expose metric in API (`/metrics` endpoint)
2. Update `prometheus.yml` if needed
3. Create panel in Grafana dashboard

### Adding new log labels
1. Add label to `promtail-config.yml` in `relabel_configs`
2. Use in Loki queries: `{service="api", level="error"}`
