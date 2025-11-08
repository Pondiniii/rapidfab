# FDM Pricing Microservice

Automated FDM 3D printing pricing service powered by Orca Slicer.

## Overview

This microservice provides instant pricing quotes for FDM 3D printing by:
1. Downloading STL files from presigned S3 URLs
2. Slicing models using Orca Slicer (with Xvfb for headless operation)
3. Extracting print metrics (time, filament weight, volume)
4. Calculating pricing based on material costs and print time

## Architecture

- **Language**: Rust + Axum
- **Slicer**: Orca Slicer v2.3.1 (via subprocess)
- **Deployment**: Docker container (Debian + Xvfb + OrcaSlicer)
- **State**: Stateless (no database)

## API

### POST /internal/pricing/fdm/quote

**Request:**
```json
{
  "file_url": "https://s3.../presigned-url",
  "material": "pla",
  "infill": 20,
  "layer_thickness": 200
}
```

**Response:**
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

**Parameters:**
- `material`: pla, abs, petg, abs-esd, asa, nylon, pc, tpu
- `infill`: 10-100 (percentage)
- `layer_thickness`: 100, 200, 300 (micrometers)

**Errors:**
- 400: Invalid parameters
- 422: Model cannot be sliced (unprintable)
- 500: Internal error

## Configuration

Environment variables (see `.env.example`):

```bash
# Service
PRICING_FDM_HOST=0.0.0.0
PRICING_FDM_PORT=8083

# Orca Slicer
ORCA_PROFILES_DIR=/app/profiles
ORCA_BINARY=orca-slicer
TEMP_DIR=/tmp/pricing-fdm

# Pricing
BASE_FEE_USD=5.00
MACHINE_RATE_USD_PER_HOUR=10.00
MARGIN_MULTIPLIER=1.30

# Material costs (per gram)
MATERIAL_PLA_COST_PER_G=0.02
MATERIAL_ABS_COST_PER_G=0.025
MATERIAL_PETG_COST_PER_G=0.03
# ... (see config.rs for full list)
```

## Development

```bash
# Build
cargo build

# Run locally (requires Orca Slicer installed)
cargo run

# Test
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## Docker

```bash
# Build
docker build -t pricing-fdm:latest -f Containerfile .

# Run
docker run -p 8083:8083 -e RUST_LOG=info pricing-fdm:latest

# Health check
curl http://localhost:8083/health
```

## Profiles

Orca Slicer profiles are bundled in `profiles/`:
- `machine.json` - Generic FDM printer (200x200x200mm)
- `process_standard.json` - Standard quality (0.2mm layer height)
- `filament_pla.json` - Generic PLA filament

For MVP, all materials use the same profile (TODO: material-specific profiles).

## Limitations (MVP)

- Single quality preset (0.2mm layer height)
- Generic printer profile (not machine-specific)
- No support for multi-material or color selection
- No advanced features (supports, brim, ironing)

## Future Enhancements

- Profile management API (upload custom profiles)
- Material-specific profiles (different temps, speeds)
- Advanced slicing parameters (supports, rafts, etc.)
- Result caching (Redis, keyed by file hash + params)
- Multiple quality presets (fine, standard, economy)
- Prometheus metrics (slice_duration, errors, etc.)

## Architecture Decision

See `plan/ADR-010-fdm-pricing-service.md` for detailed architecture decisions.
