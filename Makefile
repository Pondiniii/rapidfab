.PHONY: help build test-unit test-integration test-all test-pipeline compose-up compose-down ci test

# Compatibility layer: redirect to Task if available, otherwise use legacy commands
TASK := $(shell command -v task 2> /dev/null)

help:
	@echo "RapidFab.xyz — Makefile"
	@echo ""
ifdef TASK
	@echo "✅ Task is installed - using Task-based CI"
	@echo "  make ci                 — Full CI pipeline (redirects to: task ci)"
	@echo "  make test               — Same as 'make ci'"
else
	@echo "⚠ Task not installed - using legacy Make commands"
	@echo "  Install Task: https://taskfile.dev/installation/"
endif
	@echo ""
	@echo "Legacy commands (still supported):"
	@echo "  make build              — build API"
	@echo "  make test-unit          — run unit tests"
	@echo "  make test-integration   — run integration tests"
	@echo "  make test-all           — run all tests"
	@echo "  make test-pipeline      — E2E testing pipeline (Docker containers)"
	@echo "  make compose-up         — docker-compose up"
	@echo "  make compose-down       — docker-compose down"
	@echo ""
ifdef TASK
	@echo "Recommended: Use 'task' directly for better DX"
	@echo "  task ci                 — Full CI (format + lint + test + e2e)"
	@echo "  task --list             — Show all available tasks"
endif

# Modern CI commands (redirect to Task if available)
ci:
ifdef TASK
	@task ci
else
	@echo "⚠ Task not installed - running legacy test pipeline..."
	@$(MAKE) test-all
	@./scripts/test-e2e.sh
endif

test: ci

# Legacy commands (backward compatibility)
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
