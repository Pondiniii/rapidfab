.PHONY: help build test-unit test-integration test-all compose-up compose-down

help:
	@echo "RapidFab.xyz — Makefile"
	@echo "  make build              — build API"
	@echo "  make test-unit          — run unit tests"
	@echo "  make test-integration   — run integration tests"
	@echo "  make test-all           — run all tests"
	@echo "  make compose-up         — docker-compose up"
	@echo "  make compose-down       — docker-compose down"

build:
	cd services/api && $(MAKE) build

test-unit:
	cd services/api && $(MAKE) test-unit

test-integration:
	cd services/api && $(MAKE) test-integration

test-all: test-unit test-integration

compose-up:
	docker-compose up -d

compose-down:
	docker-compose down
