.DEFAULT_GOAL := help

CARGO       := $$(which cargo)
APP_VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\(.*\)"/\1/')
BUILD_HASH  := $(shell git rev-parse --short HEAD 2>/dev/null || echo "dev")

.PHONY: all build release check test bench lint clippy fmt fmt-check
.PHONY: doc clean coverage publish publish-dry version help

build:   ## Build (debug)
	@$(CARGO) build

release: ## Build (release)
	@$(CARGO) build --release

check:   ## Type-check only
	@$(CARGO) check --all-targets

test:    ## Run tests (nextest)
	@$(CARGO) nextest run --no-fail-fast

bench:   ## Run benchmarks (via nextest, experimental)
	@NEXTEST_EXPERIMENTAL_BENCHMARKS=1 $(CARGO) nextest bench

lint clippy: ## Lint with clippy
	@$(CARGO) clippy --all-targets -- -D warnings

fmt:     ## Format code
	@$(CARGO) fmt --all -- --style-edition 2024

fmt-check: ## Check formatting
	@$(CARGO) fmt --all --check

doc:     ## Build docs
	@$(CARGO) doc --no-deps

clean:   ## Remove build artifacts
	@$(CARGO) clean

coverage: ## Run tests with coverage (req: cargo-llvm-cov)
	@$(CARGO) llvm-cov nextest --no-cfg-coverage
	@printf '\033[32mOpen target/llvm-cov/html/index.html\033[0m\n'

publish-dry: ## Dry-run publish (validate packaging)
	@$(CARGO) publish --dry-run --allow-dirty
	@printf '\033[32m✓ Package validation passed\033[0m\n'

publish: ## Publish to crates.io
	@printf '\033[33mPublishing kalid v$(APP_VERSION)...\033[0m\n'
	@$(CARGO) publish --allow-dirty
	@printf '\033[32m✓ Published\033[0m\n'

version: ## Show version
	@printf 'kalid v%s (%s)\n' '$(APP_VERSION)' '$(BUILD_HASH)'

ci: check fmt-check lint test doc ## Full CI
	@printf '\033[32m✓ CI passed (kalid v$(APP_VERSION))\033[0m\n'

all: ci

help:
	@printf '\033[33mUsage:\033[0m make \033[36m<target>\033[0m\n'
	@awk -F ':.*## ' '/^[a-zA-Z_-]+:.*## / {printf " \033[36m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
