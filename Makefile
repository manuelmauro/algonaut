.PHONY: setup clean fmt-check fmt clippy clippy-release check check-release check-wasm build build-release test test-release integration harness harness-down docker-rustsdk-build docker-rustsdk-run docker-test ci doc help

# Setup development environment
setup:
	rustup component add rustfmt clippy
	rustup target install wasm32-unknown-unknown
	cargo fetch
	lefthook install

# Cleanup compilation outputs
clean:
	cargo clean

# Check the code format
fmt-check:
	cargo fmt --all -- --check
# Format the code
fmt:
	cargo fmt --all

# Run rust clippy with debug profile
clippy:
	cargo clippy --workspace --all-targets -- -D warnings
# Run rust clippy with release profile
clippy-release:
	cargo clippy --release --workspace --all-targets -- -D warnings

# Check code with debug profile
check:
	cargo check --workspace
# Check code with release profile
check-release:
	cargo check --release --workspace
# Check code targeting wasm32
check-wasm:
	cargo check --target wasm32-unknown-unknown

# Build the workspace with debug profile
build:
	cargo build --workspace
# Build the workspace with release profile
build-release:
	cargo build --release --workspace

# Run all unit tests with debug profile
test:
	cargo test --workspace --lib --examples --tests
# Run all unit tests with release profile
test-release:
	cargo test --release --workspace --lib --examples --tests

# Run cucumber integration tests (requires a running harness)
integration:
	cargo test --test features_runner --

# Bring the integration test harness up
harness:
	./test-harness.sh up
# Bring the integration test harness down
harness-down:
	./test-harness.sh down

# Build the Rust SDK testing docker image
docker-rustsdk-build:
	docker build -t rust-sdk-testing .
# Run the Rust SDK testing docker image
docker-rustsdk-run:
	docker ps -a
	docker run -it --network host rust-sdk-testing:latest
# Run the full docker test (harness + build + run)
docker-test: harness docker-rustsdk-build docker-rustsdk-run

# Run all CI checks (fmt-check, clippy, test, build)
ci: fmt-check clippy test build

# Generate documentation
doc:
	cargo doc --no-deps --open

# Show help
help:
	@echo ''
	@echo 'Usage:'
	@echo ' make [target]'
	@echo ''
	@echo 'Targets:'
	@awk '/^[a-zA-Z\-\_0-9]+:/ { \
	helpMessage = match(lastLine, /^# (.*)/); \
		if (helpMessage) { \
			helpCommand = substr($$1, 0, index($$1, ":")); \
			helpMessage = substr(lastLine, RSTART + 2, RLENGTH); \
			printf "\033[36m%-30s\033[0m %s\n", helpCommand,helpMessage; \
		} \
	} \
	{ lastLine = $$0 }' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help
