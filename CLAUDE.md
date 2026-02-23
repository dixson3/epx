# epx Project Instructions

## Pre-Commit Checklist

Before committing Rust code changes, always run:

1. `cargo fmt` — auto-format all source files
2. `cargo clippy -- -D warnings` — zero warnings policy
3. `cargo test` — all tests must pass

CI enforces all three (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release`) on both macOS and Ubuntu.
