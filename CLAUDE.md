# dvim - A vim-like text editor in Rust

## Build & Test
- `cargo check --quiet` for fast compilation verification (PREFER over cargo build)
- `cargo test --quiet` to run tests
- `cargo clippy -- -D warnings` for linting
- `cargo fmt --all` before committing

## Architecture
- Terminal UI via crossterm/ratatui
- Buffer management in src/buffer/
- Modal input handling in src/mode/
- Rendering in src/ui/

## Conventions
- Use `thiserror` for custom error types, `anyhow` in main
- Prefer `Result` over `unwrap`/`expect` — clippy enforces this
- Write tests alongside implementation, not as afterthought
