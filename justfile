# Show available targets
help:
  @just --list --unsorted

# Run static checks
check:
  cargo check --workspace

# Run unit and integration tests
test:
  cargo test --workspace

# Run clippy with warnings denied
lint:
  cargo clippy --workspace --all-targets -- -D warnings

# Format all Rust code
fmt:
  cargo fmt --all

# Install the `skil` CLI locally from this repository
install:
  cargo install --path src/skil

# Build the workspace
build:
  cargo build --workspace

# Preview docs website with Cloudflare Workers
cf-dev:
  npx wrangler dev

# Deploy docs website to Cloudflare Workers
cf-deploy:
  npx wrangler deploy
