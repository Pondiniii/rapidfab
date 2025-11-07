.PHONY: help build test-unit test-integration test-all test-pipeline compose-up compose-down

help:
	@echo "RapidFab.xyz — Makefile"
	@echo "  make build              — build API"
	@echo "  make test-unit          — run unit tests"
	@echo "  make test-integration   — run integration tests"
	@echo "  make test-all           — run all tests"
	@echo "  make test-pipeline      — E2E testing pipeline (Docker containers)"
	@echo "  make compose-up         — docker-compose up"
	@echo "  make compose-down       — docker-compose down"

build:
	cd services/api && $(MAKE) build

test-unit:
	cd services/api && $(MAKE) test-unit

test-integration:
	cd services/api && $(MAKE) test-integration

test-all: test-unit test-integration

test-pipeline:
	@echo "Running full E2E testing pipeline (Docker containers)..."
	./scripts/test-pipeline.sh

compose-up:
	docker-compose up -d

compose-down:
	docker-compose down
