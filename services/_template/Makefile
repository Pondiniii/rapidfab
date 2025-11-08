.PHONY: help fmt lint test build run docker-build docker-run

CARGO = cargo
SERVICE_NAME = template-service
# TODO: Replace with your service name
DOCKER_IMAGE = rapidfab-$(SERVICE_NAME)

help:
	@echo "$(SERVICE_NAME) Makefile"
	@echo "  make fmt          — cargo fmt"
	@echo "  make lint         — cargo clippy -D warnings"
	@echo "  make test         — cargo test"
	@echo "  make build        — cargo build --release"
	@echo "  make run          — cargo run"
	@echo "  make docker-build — build Docker image"
	@echo "  make docker-run   — run Docker container"

fmt:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --all-targets -- -D warnings

test:
	$(CARGO) test

build:
	$(CARGO) build --release

run:
	RUST_LOG=info $(CARGO) run

docker-build:
	docker build -t $(DOCKER_IMAGE):latest -f Containerfile .

docker-run:
	docker run -p 8080:8080 --rm $(DOCKER_IMAGE):latest
