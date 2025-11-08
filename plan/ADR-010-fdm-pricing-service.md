# ADR-010: FDM Pricing Service Architecture

**Status**: Accepted
**Date**: 2025-11-08
**Authors**: Claude Code
**Related**: ADR-009 (Upload Service), ADR-005 (Pricing Microservices), ADR-002 (Tech Stack)

---

## Context

RapidFab.xyz needs automated pricing for FDM 3D printing. Users upload STL files and receive instant quotes based on slicing simulation. The pricing must be:
- Fast (< 10s response time)
- Accurate (based on actual slicing, not estimation)
- Cost-effective (runs on ~10 USD VPS)
- Isolated (no impact on other services)
- Stateless (no database, horizontally scalable)

### Requirements

**Functional:**
- Accept STL file URL + print parameters (material, infill, layer thickness)
- Slice model using Orca Slicer to extract metrics (print time, filament weight)
- Calculate pricing based on material cost + machine time + margin
- Return detailed quote (price, lead time, print metrics)

**Non-Functional:**
- Response time: < 10s for typical models (< 10MB)
- Accuracy: ±5% vs actual production cost
- Availability: 99% uptime
- Scalability: Handle 100 requests/hour (MVP)

---

## Decision

### 1. Technology Choice

**Rust + Axum** (instead of Python + FastAPI)

**Rationale:**
- Consistent with existing stack (API, Upload services)
- Better performance for subprocess management (Orca Slicer invocation)
- Stronger type safety for pricing calculations (no floating-point surprises)
- Smaller Docker image footprint (~400MB vs ~800MB for Python + NumPy)
- User preference (specified Rust if confident)

**Trade-offs:**
- (+) Performance, memory efficiency, type safety
- (+) Easier integration with existing Rust codebase
- (-) Rust has limited mesh/CAD libraries vs Python (trimesh, numpy-stl)
- (-) For future ML-based pricing, Python would be easier

**Mitigation:**
- Use Orca Slicer for all mesh operations (no need for Rust mesh libs)
- If ML needed later, spawn separate Python service for that

---

### 2. Architecture Pattern

**Single Container: Orca Slicer + Rust Microservice**

```
┌─────────────────────────────────────────┐
│ pricing-fdm Container                   │
│                                         │
│ ┌─────────────────┐  ┌───────────────┐ │
│ │ Rust Binary     │  │ OrcaSlicer    │ │
│ │ (Axum API)      │  │ + Xvfb        │ │
│ │                 │  │               │ │
│ │ - HTTP handlers │  │ - CLI slicer  │ │
│ │ - S3 download   │  │ - Profiles    │ │
│ │ - Pricing logic │  │ - G-code gen  │ │
│ └────────┬────────┘  └───────▲───────┘ │
│          │ subprocess        │         │
│          └───────────────────┘         │
│                                         │
│ /app/profiles/ (bundled presets)       │
└─────────────────────────────────────────┘
```

**Alternatives Considered:**

**Option A: Separate Containers (Orca Engine + Pricing API)**
```
pricing-fdm ───HTTP──→ orca-engine
```
- (+) Clean separation of concerns
- (+) Orca engine can be shared by other services
- (-) Network overhead (~50-100ms per request)
- (-) Two containers to deploy/monitor
- (-) Requires inter-container communication

**Option B: Native Rust Slicer Library**
```
pricing-fdm (Rust + libslic3r bindings)
```
- (+) No external dependencies
- (+) Faster (no subprocess overhead)
- (-) libslic3r Rust bindings immature
- (-) Complex to maintain (C++ interop)
- (-) Locked into Slic3r engine (can't switch to Orca features)

**Chosen: Single Container**
- KISS principle - fewer moving parts
- No network overhead (subprocess ~10ms vs HTTP ~100ms)
- Atomic deployments (profiles + code together)
- Sufficient for MVP scale (100 req/hour)

---

### 3. Orca Slicer Integration

**Subprocess with Xvfb** (headless X11 virtual framebuffer)

**Problem:** Orca Slicer requires OpenGL/GLFW for rendering, cannot run headless natively.

**Solution:**
```bash
xvfb-run -a orca-slicer --slice 0 --export-3mf model.stl
```

**Alternatives:**
1. **PrusaSlicer Console** - Designed for CLI, but fewer features than Orca
2. **CuraEngine** - True CLI engine, but different profile format
3. **VNC in Docker** - Massive overhead (~2GB image), not automatable

**Trade-offs:**
- (+) Full Orca Slicer features (auto-orient, supports, etc.)
- (+) Proven to work (tested in research phase)
- (+) Small overhead (~100ms, ~50MB RAM)
- (-) Dependency on Xvfb package
- (-) Not officially supported (no CLI docs from Orca)

**Risk Mitigation:**
- Pin Orca Slicer version (v2.3.1) in Dockerfile
- Monitor release notes for CLI changes
- Health check includes test slice
- Fallback: Switch to PrusaSlicer if Orca breaks

---

### 4. Metric Extraction

**Parse G-code Comments** (primary method)

Orca Slicer embeds metrics in G-code comments:
```gcode
; estimated printing time (normal mode) = 2h 30m 45s
; filament used [g] = 125.5
; filament used [mm] = 41234.56
; filament used [cm3] = 101.2
```

**Extraction Method:**
1. Slice model → export 3MF
2. Extract 3MF (ZIP archive) → `Metadata/plate_1.gcode`
3. Parse G-code with regex:
   - `; estimated printing time.*= (\d+)h (\d+)m`
   - `; filament used \[g\] = ([\d.]+)`
4. Fallback: Estimate from weight (density formula)

**Alternatives:**
- **XML parsing** (`Metadata/slice_info.config`) - More structured, but less reliable
- **API mode** - Orca doesn't have HTTP API
- **Custom slicer** - Overkill for MVP

---

### 5. Pricing Calculation

**Cost Model:**
```
material_cost = filament_weight_g × material_cost_per_g
machine_cost = print_time_hours × machine_rate_usd_per_hour
base_cost = material_cost + machine_cost + base_fee_usd
total_price = base_cost × margin_multiplier
```

**Configurable Parameters:**
- `base_fee_usd` = 5.00 (setup, QA, packaging)
- `machine_rate_usd_per_hour` = 10.00 (depreciation, electricity, labor)
- `margin_multiplier` = 1.30 (30% markup)
- `material_cost_per_g` = 0.02 (PLA), 0.025 (ABS), etc.

**Lead Time Estimation:**
```
< 8h print   → 1 day lead time
8-24h print  → 2 days
24-48h print → 3 days
> 48h print  → (hours / 24) days
```

**Future Enhancements:**
- Volume discounts (batch pricing)
- Rush fees (24h turnaround)
- Color/finish premiums
- Support material costs

---

### 6. State Management

**Stateless** (no database)

All pricing is calculated on-demand, no storage of quotes.

**Rationale:**
- Pricing logic deterministic (same inputs → same output)
- No need for quote history in MVP
- Horizontally scalable (no DB bottleneck)
- Simpler deployment

**Future:**
- Cache results in Redis (key: `sha256(file) + params`)
- Store quotes in PostgreSQL (for analytics, quote retrieval)

---

### 7. Profile Management

**Bundled Profiles** (MVP)

Profiles included in Docker image:
- `profiles/machine.json` - Generic FDM printer (200x200x200mm)
- `profiles/process_standard.json` - 0.2mm layer height
- `profiles/filament_pla.json` - Generic PLA

**Mapping:**
- `layer_thickness=100` → fine preset (future)
- `layer_thickness=200` → standard preset (current)
- `layer_thickness=300` → economy preset (future)
- All materials → same filament profile (MVP limitation)

**Future: Profile Management API**
```
POST /internal/pricing/fdm/profiles
Body: { "name": "bambu_pla_black", "profile": "base64..." }
```

Store profiles in Docker volume or S3.

---

## Consequences

### Positive

1. **Fast Development** - MVP ready in ~4h (vs weeks for custom slicer)
2. **Accurate Pricing** - Based on actual slicing, not estimation
3. **Simple Deployment** - One container, no external dependencies
4. **Cost-Effective** - Runs on shared VPS, no GPU needed
5. **Maintainable** - Clear module boundaries, testable

### Negative

1. **Orca CLI Dependency** - Undocumented, may break in updates
2. **Container Size** - ~400MB (Orca AppImage + deps)
3. **MVP Limitations** - Single quality preset, no material-specific profiles
4. **Xvfb Overhead** - ~100ms per request, ~50MB RAM per slice

### Mitigations

- Pin Orca version, monitor releases
- Multi-stage build, Alpine base (minimize size)
- Roadmap for profile management (post-MVP)
- Health checks, retry logic

---

## Implementation

### File Structure

```
services/pricing-fdm/
├── Cargo.toml              # Rust dependencies
├── Containerfile           # Multi-stage Docker build
├── Makefile               # Standard targets
├── README.md              # Quick start guide
├── profiles/              # Orca presets (bundled)
│   ├── machine.json
│   ├── process_standard.json
│   └── filament_pla.json
├── src/
│   ├── main.rs            # Axum server
│   ├── config.rs          # Environment config
│   ├── app/
│   │   ├── dto.rs         # Request/Response types
│   │   ├── handlers.rs    # HTTP endpoints
│   │   └── pricing.rs     # Pricing calculation
│   ├── slicer/
│   │   ├── orca.rs        # Subprocess wrapper
│   │   └── parser.rs      # G-code metric extraction
│   └── utils/
│       └── download.rs    # S3 presigned URL download
└── tests/
    └── integration_test.rs
```

### API Contract

**POST /internal/pricing/fdm/quote**

Request:
```json
{
  "file_url": "https://s3.../presigned-url",
  "material": "pla",
  "infill": 20,
  "layer_thickness": 200
}
```

Response:
```json
{
  "quote_id": "uuid",
  "total_usd": 45.50,
  "material_cost_usd": 12.30,
  "machine_cost_usd": 28.20,
  "base_fee_usd": 5.00,
  "lead_time_days": 3,
  "print_time_hours": 2.82,
  "filament_weight_g": 615.0,
  "volume_cm3": 495.9
}
```

### Integration Points

**With API Service:**
```
Client → API → Upload Service (store STL)
              ↓
          Pricing Service (quote)
              ↓
          Return quote to client
```

**With Upload Service:**
- API generates presigned S3 URL
- Passes URL to Pricing Service
- Pricing Service downloads STL, slices, returns quote

---

## Testing Strategy

### Unit Tests
- Pricing calculation (material cost, lead time)
- G-code parsing (time, weight extraction)
- Request validation (material, infill, layer thickness)

### Integration Tests
- Orca Slicer subprocess execution
- 3MF extraction and parsing
- End-to-end slice with test cube

### E2E Tests
```bash
tests/e2e/fdm_pricing_test.sh
```
- Health check
- Invalid request handling
- Material validation
- Response format verification

### CI Pipeline
```bash
task ci  # Runs format, lint, unit, Docker build, E2E
```

---

## Deployment

### Docker Compose

```yaml
pricing-fdm:
  build: ./services/pricing-fdm
  ports:
    - "8083:8083"
  environment:
    - BASE_FEE_USD=5.00
    - MACHINE_RATE_USD_PER_HOUR=10.00
    - MATERIAL_PLA_COST_PER_G=0.02
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8083/health"]
```

### Health Checks

- `/health` - Simple "OK" response
- Container health check: curl every 10s
- Optional: Test slice with 1mm cube on startup

---

## Metrics & Monitoring

### Prometheus Metrics (Future)

```
pricing_fdm_requests_total{material,status}
pricing_fdm_slice_duration_seconds
pricing_fdm_download_size_bytes
pricing_fdm_errors_total{error_type}
```

### Logging

```
info: "Processing quote request for material=pla, infill=20, layer_thickness=200um"
info: "Slicing successful: print_time=2.82h, weight=615g"
info: "Quote generated: id=uuid, total=$45.50"
error: "Slicing failed: <error>"
```

---

## Future Work

### Phase 2: Advanced Features
- Material-specific profiles (temps, speeds)
- Quality presets (fine/standard/economy)
- Support structures cost calculation
- Multi-material pricing

### Phase 3: Optimization
- Result caching (Redis)
- STL validation before slicing
- Parallel slicing (process pool)
- Profile hot-reload (no redeploy)

### Phase 4: ML Pricing
- Train model on historical quotes
- Estimate price without slicing (instant quotes)
- Fallback to slicing for accuracy validation

---

## References

- Orca Slicer: https://github.com/SoftFever/OrcaSlicer
- ADR-002: Tech Stack Selection
- ADR-005: Pricing Microservices Strategy
- ADR-009: Upload Service Architecture
- Research Report: `tests/orca-slicer-research.md` (if exists)

---

## Acceptance Criteria

✅ Container builds successfully
✅ `/health` endpoint returns 200 OK
✅ Quote request with valid params returns pricing
✅ Invalid params rejected with 400
✅ `task ci` passes all tests
✅ E2E test validates API contract
✅ Documentation complete (README, ADR)

---

**Date Accepted**: 2025-11-08
**Next Review**: After 1000 production quotes (evaluate accuracy, performance)
