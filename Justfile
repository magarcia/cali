# Cali CLI Task Runner

default:
    @just --list

# Build the project
build:
    cargo build --release

# Run tests
test:
    cargo test

# Run tests with output
test-v:
    cargo test -- --nocapture

# Run clippy
clippy:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check code without building
check:
    cargo check

# Run the CLI (debug mode)
run ARGS:
    cargo run -- {{ARGS}}

# Install the release binary
install:
    cargo install --path .

# Clean build artifacts
clean:
    cargo clean

# Run integration tests
integration:
    cargo test --test integration

# Profile startup time
profile:
    hyperfine 'cargo run --release --' --shell none

# Show size of release binary
size:
    ls -lh target/release/cali

# Generate test coverage report
coverage:
    ~/.cargo/bin/cargo llvm-cov --html --output-dir coverage
