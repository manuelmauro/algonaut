.PHONY: setup clean fmt-check fmt clippy clippy-release check check-release check-wasm build build-release test test-release cucumber check-cucumber test-e2e check-e2e harness harness-down sandbox sandbox-down sandbox-clean fund docker-rustsdk-build docker-rustsdk-run docker-test fetch-openapi-specs generate-clients ci doc help

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

# Run the cucumber acceptance suite (requires a running harness)
cucumber:
	cargo test --test cucumber --

# Compile-check the cucumber runner without invoking it (it would need
# a live harness). Catches type errors that `cargo check` skips because
# `[[test]] test = false` for the cucumber suite.
check-cucumber:
	cargo test --test cucumber --no-run

# Run the ARC-56 end-to-end tests against a running node (e.g. `make sandbox`).
# They self-fund from KMD; the suite is `[[test]] test = false`, so it is run
# explicitly here. Override ALGOD_URL/ALGOD_TOKEN/KMD_URL/KMD_TOKEN to retarget.
test-e2e:
	cargo test --test e2e --

# Compile-check the e2e suite without a node (it is `[[test]] test = false`, so
# the default `cargo test` skips it). Mirrors check-cucumber.
check-e2e:
	cargo test --test e2e --no-run

# Bring the integration test harness up
harness:
	./test-harness.sh up
# Bring the integration test harness down
harness-down:
	./test-harness.sh down

# Local Algorand sandbox (https://github.com/algorand/sandbox), cloned on first
# use into a git-ignored directory. Unlike `harness`, which compiles algod and
# indexer from source, the sandbox runs prebuilt release images on the standard
# ports examples.env expects: algod :4001, kmd :4002, indexer :8980, token of
# 64 'a's. The `dev` config enables DevMode (a block per transaction).
SANDBOX_DIR := sandbox
SANDBOX_REPO := https://github.com/algorand/sandbox
SANDBOX_CONFIG ?= dev
# microAlgos sent to each example account by `make fund`.
FUND_AMOUNT ?= 1000000000

# Bring up a fast local sandbox for development (prebuilt images, no rebuild)
sandbox:
	@if [ ! -d "$(SANDBOX_DIR)/.git" ]; then \
	  git clone --depth 1 "$(SANDBOX_REPO)" "$(SANDBOX_DIR)"; \
	fi
	cd "$(SANDBOX_DIR)" && ./sandbox up $(SANDBOX_CONFIG)
# Tear down the sandbox network; keeps the clone so the next `sandbox` is quick
sandbox-down:
	@if [ -d "$(SANDBOX_DIR)" ]; then cd "$(SANDBOX_DIR)" && ./sandbox down; fi
# Delete the sandbox containers, data, and the clone entirely
sandbox-clean:
	@if [ -d "$(SANDBOX_DIR)" ]; then cd "$(SANDBOX_DIR)" && ./sandbox clean || true; fi
	rm -rf "$(SANDBOX_DIR)"

# The example accounts aren't in the dev genesis, and a fresh sandbox resets
# balances; FUND_AMOUNT (microAlgos) overrides the per-account amount.
# Fund the example accounts (every *_ADDRESS in examples.env); run after `make sandbox`
fund:
	@test -d "$(SANDBOX_DIR)" || { echo "no $(SANDBOX_DIR)/ — run 'make sandbox' first"; exit 1; }
	@set -e; \
	from=$$(cd "$(SANDBOX_DIR)" && ./sandbox goal account list | awk '/microAlgos/{print $$2; exit}'); \
	if [ -z "$$from" ]; then echo "no funded genesis account found — is the sandbox up?"; exit 1; fi; \
	addrs=$$(sed -n 's/^export [A-Z0-9_]*_ADDRESS="\(.*\)"/\1/p' examples.env); \
	for a in $$addrs; do \
	  (cd "$(SANDBOX_DIR)" && ./sandbox goal clerk send -a $(FUND_AMOUNT) -f "$$from" -t "$$a") >/dev/null; \
	  echo "funded $$a with $(FUND_AMOUNT) microAlgos"; \
	done

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
# overwrite them. Per ADR relocate-generated-models the generated *models* now
# live in algonaut_model::{algod,indexer} while the *apis* stay in the client
# crates, so the generated `src/models` is diffed against algonaut_model and the
# generated `src/apis` against the client crate (see the echo guidance below and
# docs/adr/openapi-client-regeneration.md, docs/adr/relocate-generated-models.md).
generate-clients:
	python3 openapi/preprocess.py
	docker run --rm -v "$(CURDIR)":/local $(OPENAPI_IMAGE) generate \
	  -c /local/openapi/config-algod.yaml --skip-validate-spec \
	  -t /local/openapi/templates \
	  -i /local/openapi/generated/_specs/algod.oas3.json -o /local/openapi/generated/algod
	docker run --rm -v "$(CURDIR)":/local $(OPENAPI_IMAGE) generate \
	  -c /local/openapi/config-indexer.yaml --skip-validate-spec \
	  -t /local/openapi/templates \
	  -i /local/openapi/generated/_specs/indexer.oas3.json -o /local/openapi/generated/indexer
	@echo 'Formatting generated output so the diff reflects only semantic drift...'
	find openapi/generated/algod openapi/generated/indexer -name '*.rs' \
	  | xargs rustfmt --edition 2024
	@echo 'Regenerated into openapi/generated/. Models relocate to algonaut_model,'
	@echo 'apis stay in the client crate (ADR relocate-generated-models). Review drift:'
	@echo '  git diff --no-index openapi/generated/algod/src/models   algonaut_model/src/algod'
	@echo '  git diff --no-index openapi/generated/algod/src/apis     algonaut_algod/src/apis'
	@echo '  git diff --no-index openapi/generated/indexer/src/models algonaut_model/src/indexer'
	@echo '  git diff --no-index openapi/generated/indexer/src/apis   algonaut_indexer/src/apis'

# Run all CI checks (fmt-check, clippy, test, check-cucumber, check-e2e, build)
ci: fmt-check clippy test check-cucumber check-e2e build

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
