.PHONY: help build run test lint fmt clean docker-build docker-run

help:
	@echo "Available targets:"
	@echo "  build        - Build Rust binary"
	@echo "  run          - Run service locally"
	@echo "  test         - Run tests"
	@echo "  lint         - Run clippy"
	@echo "  fmt          - Format code"
	@echo "  clean        - Clean build artifacts"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run Docker container"

build:
	cargo build --release

run:
	cargo run

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --check

clean:
	cargo clean

docker-build:
	docker build -t pricing-fdm:latest -f Containerfile .

docker-run:
	docker run -p 8083:8083 -e RUST_LOG=info pricing-fdm:latest
