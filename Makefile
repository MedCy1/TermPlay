# Makefile for TermPlay
.PHONY: help build test clean install dev lint fmt check release docker

# Variables
BINARY_NAME := termplay
VERSION := $(shell grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
TARGET_DIR := target
BUILD_DIR := build
INSTALL_DIR := ~/.local/bin

# Colors for messages
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m

# Default help
help: ## Show this help
	@echo "$(CYAN)TermPlay v$(VERSION) - Makefile$(NC)"
	@echo "=================================="
	@echo ""
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "$(CYAN)%-15s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Build and development
build: ## Build the project in release mode
	@echo "$(GREEN)🔨 Building TermPlay...$(NC)"
	cargo build --release
	@echo "$(GREEN)✅ Build finished$(NC)"

dev: ## Build and run in development mode
	@echo "$(GREEN)🚀 Running in development mode...$(NC)"
	cargo run

build-debug: ## Build in debug mode
	@echo "$(GREEN)🔨 Building in debug mode...$(NC)"
	cargo build

# Tests and quality
test: ## Run all tests
	@echo "$(GREEN)🧪 Running tests...$(NC)"
	cargo test

test-verbose: ## Run tests in verbose mode
	@echo "$(GREEN)🧪 Tests in verbose mode...$(NC)"
	cargo test -- --nocapture

bench: ## Run benchmarks
	@echo "$(GREEN)📊 Running benchmarks...$(NC)"
	cargo bench

# Linting and formatting
lint: ## Run clippy for linting
	@echo "$(GREEN)🔍 Linting with clippy...$(NC)"
	cargo clippy --all-targets --all-features -- -D warnings

fmt: ## Format the code
	@echo "$(GREEN)🎨 Formatting code...$(NC)"
	cargo fmt

check: ## Check compilation without building
	@echo "$(GREEN)✅ Checking compilation...$(NC)"
	cargo check --all-targets

check-all: lint fmt test ## Run all checks (lint, fmt, test)
	@echo "$(GREEN)✅ All checks finished$(NC)"

# Installation and distribution
install: build ## Install the binary locally
	@echo "$(GREEN)📦 Local installation...$(NC)"
	mkdir -p $(INSTALL_DIR)
	cp $(TARGET_DIR)/release/$(BINARY_NAME) $(INSTALL_DIR)/
	@echo "$(GREEN)✅ Installed to $(INSTALL_DIR)/$(BINARY_NAME)$(NC)"

uninstall: ## Uninstall the local binary
	@echo "$(YELLOW)🗑️  Uninstalling...$(NC)"
	rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "$(GREEN)✅ Uninstalled$(NC)"

# Release and distribution
release-patch: ## Create a patch release (1.0.0 -> 1.0.1)
	@echo "$(GREEN)🚀 Patch release...$(NC)"
	./scripts/release.sh patch

release-minor: ## Create a minor release (1.0.0 -> 1.1.0)
	@echo "$(GREEN)🚀 Minor release...$(NC)"
	./scripts/release.sh minor

release-major: ## Create a major release (1.0.0 -> 2.0.0)
	@echo "$(GREEN)🚀 Major release...$(NC)"
	./scripts/release.sh major

# Multi-platform build
build-linux: ## Build for Linux x86_64
	@echo "$(GREEN)🐧 Building Linux x86_64...$(NC)"
	cargo build --release --target x86_64-unknown-linux-gnu

build-windows: ## Build for Windows x86_64
	@echo "$(GREEN)🪟 Building Windows x86_64...$(NC)"
	cargo build --release --target x86_64-pc-windows-msvc

build-macos: ## Build for macOS x86_64
	@echo "$(GREEN)🍎 Building macOS x86_64...$(NC)"
	cargo build --release --target x86_64-apple-darwin

build-macos-arm: ## Build for macOS ARM64
	@echo "$(GREEN)🍎 Building macOS ARM64...$(NC)"
	cargo build --release --target aarch64-apple-darwin

build-all: build-linux build-windows build-macos build-macos-arm ## Build for all platforms

# Packaging
package: build ## Create a distribution package
	@echo "$(GREEN)📦 Creating package...$(NC)"
	mkdir -p $(BUILD_DIR)
	cp $(TARGET_DIR)/release/$(BINARY_NAME) $(BUILD_DIR)/
	cp README.md $(BUILD_DIR)/
	cp LICENSE $(BUILD_DIR)/
	tar -czf $(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz -C $(BUILD_DIR) .
	@echo "$(GREEN)✅ Package created: $(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz$(NC)"

# Documentation
docs: ## Generate documentation
	@echo "$(GREEN)📚 Generating documentation...$(NC)"
	cargo doc --no-deps --open

docs-private: ## Generate documentation including private items
	@echo "$(GREEN)📚 Full documentation...$(NC)"
	cargo doc --no-deps --document-private-items --open

# Cleaning
clean: ## Clean build files
	@echo "$(YELLOW)🧹 Cleaning...$(NC)"
	cargo clean
	rm -rf $(BUILD_DIR)
	@echo "$(GREEN)✅ Cleaning finished$(NC)"

clean-all: clean ## Full clean (build + caches)
	@echo "$(YELLOW)🧹 Full cleaning...$(NC)"
	rm -rf target/
	rm -rf Cargo.lock
	@echo "$(GREEN)✅ Full cleaning finished$(NC)"

# Development utilities
run-snake: ## Run Snake directly
	@echo "$(GREEN)🐍 Running Snake...$(NC)"
	cargo run -- game snake

run-tetris: ## Run Tetris directly
	@echo "$(GREEN)🧩 Running Tetris...$(NC)"
	cargo run -- game tetris

run-menu: ## Run the main menu
	@echo "$(GREEN)🎮 Running menu...$(NC)"
	cargo run

list-games: ## List all available games
	@echo "$(GREEN)📋 Available games:$(NC)"
	cargo run -- list

# Debug and profiling
debug: ## Build and run with debugger
	@echo "$(GREEN)🐛 Running in debug mode...$(NC)"
	cargo build
	gdb target/debug/$(BINARY_NAME)

profile: ## Build with profiling
	@echo "$(GREEN)📊 Building with profiling...$(NC)"
	cargo build --release
	perf record -g target/release/$(BINARY_NAME)

# Local CI/CD
ci-local: check-all build test ## Simulate CI locally
	@echo "$(GREEN)🔄 Local CI finished successfully$(NC)"

# Docker (optional)
docker-build: ## Build Docker image
	@echo "$(GREEN)🐳 Building Docker...$(NC)"
	docker build -t $(BINARY_NAME):$(VERSION) .

docker-run: docker-build ## Run the application in Docker
	@echo "$(GREEN)🐳 Running Docker...$(NC)"
	docker run -it --rm $(BINARY_NAME):$(VERSION)

# Maintenance
update-deps: ## Update dependencies
	@echo "$(GREEN)📦 Updating dependencies...$(NC)"
	cargo update

audit: ## Security audit of dependencies
	@echo "$(GREEN)🔒 Security audit...$(NC)"
	cargo audit

# Statistics
stats: ## Show project statistics
	@echo "$(CYAN)📊 Project statistics$(NC)"
	@echo "=========================="
	@echo "Version: $(VERSION)"
	@echo "Lines of code:"
	@find src -name "*.rs" -exec wc -l {} + | tail -1
	@echo "Number of Rust files:"
	@find src -name "*.rs" | wc -l
	@echo "Release binary size:"
	@if [ -f "$(TARGET_DIR)/release/$(BINARY_NAME)" ]; then \
		ls -lh $(TARGET_DIR)/release/$(BINARY_NAME) | awk '{print $$5}'; \
	else \
		echo "Not compiled"; \
	fi

# System information
info: ## Show environment information
	@echo "$(CYAN)ℹ️  Environment information$(NC)"
	@echo "=================================="
	@echo "Rust version: $(shell rustc --version)"
	@echo "Cargo version: $(shell cargo --version)"
	@echo "OS: $(shell uname -s)"
	@echo "Architecture: $(shell uname -m)"
	@echo "Git branch: $(shell git branch --show-current 2>/dev/null || echo 'Not available')"
	@echo "Git commit: $(shell git rev-parse --short HEAD 2>/dev/null || echo 'Not available')"

# Developer help
dev-setup: ## Setup development environment
	@echo "$(GREEN)⚙️  Setting up development environment...$(NC)"
	rustup component add clippy rustfmt
	cargo install cargo-audit cargo-watch
	chmod +x scripts/release.sh
	@echo "$(GREEN)✅ Environment setup complete$(NC)"

watch: ## Run cargo watch for development
	@echo "$(GREEN)👀 Watching files...$(NC)"
	cargo watch -x "check --all-targets" -x "test"

# Useful shortcuts
b: build ## Shortcut for build
t: test ## Shortcut for test
r: dev ## Shortcut for run
c: clean ## Shortcut for clean
l: lint ## Shortcut for lint
f: fmt ## Shortcut for fmt