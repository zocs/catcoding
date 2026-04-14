# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CatCoding Development Makefile
# Standard CI/CD commands — run `make help` to see all targets
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

.PHONY: help lint test build check all ci clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ── Lint ─────────────────────────────────────────────────────

lint: lint-rust lint-python lint-dashboard ## Run all linters

lint-rust: ## Lint Rust code (fmt + clippy)
	cargo fmt --all -- --check
	cargo clippy -p catcoding-daemon --all-features -- -D warnings
	cargo clippy -p catcoding --all-features -- -D warnings

lint-python: ## Lint Python agents (ruff)
	ruff check agents/
	ruff format agents/ --check

lint-dashboard: ## Lint dashboard (TypeScript)
	cd dashboard && npx vue-tsc --noEmit

# ── Test ─────────────────────────────────────────────────────

test: test-rust test-python ## Run all tests

test-rust: ## Run Rust tests
	cargo test --all-features --verbose

test-python: ## Run Python tests
	pytest agents/ -v --tb=short

test-integration: ## Run integration test (needs NATS)
	nats-server -js &
	sleep 2
	cargo build --release -p catcoding-daemon
	./target/release/catcoding-daemon &
	sleep 3
	curl -sf http://localhost:9800/health && echo "✅ OK" || echo "❌ FAIL"
	@pkill catcoding-daemon 2>/dev/null; pkill nats-server 2>/dev/null

# ── Build ────────────────────────────────────────────────────

build: build-rust build-dashboard ## Build everything

build-rust: ## Build Rust binaries (release)
	cargo build --release

build-dashboard: ## Build dashboard
	cd dashboard && npm ci && npm run build

# ── Full check (mirrors CI pipeline locally) ─────────────────

check: lint test build ## Run full CI pipeline locally

# ── CI (alias for check) ────────────────────────────────────

ci: check ## Same as 'check' — run full CI locally

# ── Formatting ──────────────────────────────────────────────

fmt: ## Auto-format all code
	cargo fmt --all
	ruff format agents/
	cd dashboard && npx prettier --write src/

# ── Clean ────────────────────────────────────────────────────

clean: ## Clean build artifacts
	cargo clean
	cd dashboard && rm -rf dist node_modules
