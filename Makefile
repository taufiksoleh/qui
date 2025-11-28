.PHONY: help build release run run-release test clean install fmt lint check clippy all dev watch fix fix-all

# Default target
.DEFAULT_GOAL := help

# Binary name
BINARY_NAME := kube-tui
INSTALL_PATH := /usr/local/bin

## help: Show this help message
help:
	@echo "Available commands:"
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'

## build: Build debug version
build:
	cargo build

## release: Build optimized release version
release:
	cargo build --release

## run: Run debug version
run:
	cargo run

## run-release: Run optimized release version
run-release:
	cargo run --release

## dev: Build and run debug version (alias for run)
dev: run

## test: Run all tests
test:
	cargo test

## test-verbose: Run tests with verbose output
test-verbose:
	cargo test -- --nocapture

## check: Quick compile check without building
check:
	cargo check

## fmt: Format code with rustfmt
fmt:
	cargo fmt

## fmt-check: Check if code is formatted
fmt-check:
	cargo fmt -- --check

## lint: Run clippy linter
lint:
	cargo clippy -- -D warnings

## clippy: Run clippy with all features (alias for lint)
clippy: lint

## fix: Automatically fix lint warnings
fix:
	cargo fix --allow-dirty --allow-staged

## fix-all: Automatically fix lint warnings and format code
fix-all:
	cargo fix --allow-dirty --allow-staged
	cargo fmt

## clean: Remove build artifacts
clean:
	cargo clean
	rm -rf target/

## install: Install binary to system (requires sudo)
install: release
	sudo cp target/release/$(BINARY_NAME) $(INSTALL_PATH)/
	@echo "Installed $(BINARY_NAME) to $(INSTALL_PATH)"

## install-local: Install binary to ~/.local/bin (no sudo)
install-local: release
	mkdir -p ~/.local/bin
	cp target/release/$(BINARY_NAME) ~/.local/bin/
	@echo "Installed $(BINARY_NAME) to ~/.local/bin"
	@echo "Make sure ~/.local/bin is in your PATH"

## uninstall: Uninstall binary from system
uninstall:
	sudo rm -f $(INSTALL_PATH)/$(BINARY_NAME)
	@echo "Uninstalled $(BINARY_NAME) from $(INSTALL_PATH)"

## watch: Watch for changes and rebuild (requires cargo-watch)
watch:
	cargo watch -x run

## all: Run fmt, lint, test, and build release
all: fmt lint test release

## size: Show binary size
size: release
	@ls -lh target/release/$(BINARY_NAME) | awk '{print "Binary size: " $$5}'

## bench: Run benchmarks
bench:
	cargo bench

## doc: Build and open documentation
doc:
	cargo doc --open

## update: Update dependencies
update:
	cargo update

## outdated: Check for outdated dependencies (requires cargo-outdated)
outdated:
	cargo outdated

## audit: Check for security vulnerabilities (requires cargo-audit)
audit:
	cargo audit

## bloat: Analyze binary bloat (requires cargo-bloat)
bloat: release
	cargo bloat --release

## expand: Expand macros (requires cargo-expand)
expand:
	cargo expand

## tree: Show dependency tree
tree:
	cargo tree
