.PHONY: help fmt lint test test-unit test-integration test-e2e build run

CARGO = cargo

help:
	@echo "API Makefile"
	@echo "  make fmt                — cargo fmt"
	@echo "  make lint               — cargo clippy"
	@echo "  make test               — run all tests"
	@echo "  make test-unit          — unit tests"
	@echo "  make test-integration   — integration tests"
	@echo "  make test-e2e           — e2e tests (requires running API)"
	@echo "  make build              — cargo build --release"
	@echo "  make run                — cargo run"

fmt:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --all-targets -- -D warnings

test: test-unit test-integration

test-unit:
	$(CARGO) test --lib --bins

test-integration:
	$(CARGO) test --test '*' -- --test-threads=1

test-e2e:
	@echo "Running E2E tests..."
	@chmod +x ../../tests/e2e/auth_flow_test.sh
	@../../tests/e2e/auth_flow_test.sh

build:
	$(CARGO) build --release

run:
	RUST_LOG=info $(CARGO) run
