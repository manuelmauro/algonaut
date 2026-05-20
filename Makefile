.PHONY: setup clean fmt-check fmt clippy clippy-release check check-release check-wasm build build-release test test-release integration check-integration harness harness-down docker-rustsdk-build docker-rustsdk-run docker-test fetch-openapi-specs generate-clients ci doc help

# Version of openapi-generator used to regenerate the algod/indexer clients.
OPENAPI_GENERATOR_VERSION := v6.6.0
OPENAPI_IMAGE := openapitools/openapi-generator-cli:$(OPENAPI_GENERATOR_VERSION)

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

# Compile-check the cucumber runner without invoking it (it would need
# a live harness). Catches type errors that `cargo check` skips because
# `[[test]] test = false` for features_runner.
check-integration:
	cargo test --test features_runner --no-run

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

# Refresh the pinned Algorand OpenAPI specs from upstream
fetch-openapi-specs:
	curl -fsSL -o openapi/specs/algod.oas3.json \
	  https://raw.githubusercontent.com/algorand/go-algorand/master/daemon/algod/api/algod.oas3.yml
	curl -fsSL -o openapi/specs/indexer.oas3.json \
	  https://raw.githubusercontent.com/algorand/indexer/main/api/indexer.oas3.yml

# Regenerate the algod/indexer clients into openapi/generated/ (requires Docker).
# Output is for review-diffing against the customized crates; it does NOT
# overwrite algonaut_algod/ or algonaut_indexer/. See
# docs/adr/openapi-client-regeneration.md.
generate-clients:
	docker run --rm -v "$(CURDIR)":/local $(OPENAPI_IMAGE) generate \
	  -c /local/openapi/config-algod.yaml --skip-validate-spec \
	  -i /local/openapi/specs/algod.oas3.json -o /local/openapi/generated/algod
	docker run --rm -v "$(CURDIR)":/local $(OPENAPI_IMAGE) generate \
	  -c /local/openapi/config-indexer.yaml --skip-validate-spec \
	  -i /local/openapi/specs/indexer.oas3.json -o /local/openapi/generated/indexer
	@echo 'Regenerated into openapi/generated/. Review drift with e.g.:'
	@echo '  git diff --no-index openapi/generated/algod/src algonaut_algod/src'

# Run all CI checks (fmt-check, clippy, test, check-integration, build)
ci: fmt-check clippy test check-integration build

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
