set shell := ["bash", "-uc"]

# List available recipes
default:
    @just --list

# Build the whole workspace
build:
    cargo build --workspace

# Build in release mode
release:
    cargo build --workspace --release

# Run the CLI (pass args after `--`)
run *args:
    cargo run --bin dm -- {{args}}

# Run the TUI
tui *args:
    cargo run --bin dmtui -- {{args}}

# Run the GUI
ui *args:
    cargo run --bin dmui -- {{args}}

# Run the test suite
test:
    cargo test --workspace

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy with warnings denied
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run cargo-deny checks
deny:
    cargo deny check

# Run fmt-check, lint, and tests
check: fmt-check lint test

# Install the CLI locally
install:
    cargo install --path crates/cli

# Remove build artifacts
clean:
    cargo clean
