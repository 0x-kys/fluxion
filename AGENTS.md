# AGENTS.md

Guidelines for agents working on Fluxion, a headless text engine built in Rust.

## Build & Test Commands

### Core Commands

```bash
# Build
cargo build              # debug
cargo build --release    # optimized
cargo run --release      # run TUI

# Tests
cargo test                     # all tests
cargo test <test_name>         # single test
cargo test -p fluxion-core     # specific crate
cargo test -- --nocapture      # with output
cargo test -- --test-threads 1 # single thread

# Lint & Format
cargo check                    # check without build
cargo clippy                   # lint
cargo clippy --fix             # auto-fix
cargo fmt                      # format
cargo fmt -- --check           # check formatting
```

### CI Pipeline

`.github/workflows/rust.yml` runs `cargo build --verbose` and `cargo test --verbose`.
Always run `cargo clippy` and `cargo fmt` before committing.

## Code Style Guidelines

### Naming Conventions

- **Variables/Functions:** `snake_case` (e.g., `handle_action`, `scroll_offset`)
- **Types/Structs/Enums:** `PascalCase` (e.g., `Action`, `Editor`, `Tui`)
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Acronyms:** Regular words (e.g., `Tui`, `Rope` not `TUI`)

### Imports

Group logically: external crates first, then internal modules.

```rust
use std::error::Error;
use tracing::{Level, info};
use fluxion_core::Editor;
use fluxion_tui::Tui;
```

### Formatting

- 4-space indentation (standard Rust)
- No trailing whitespace
- Suggested max line width: 100 characters
- Trust rustfmt defaults

### Type System

- **NEVER** suppress type errors
- Use `Result<T, E>` for error handling, not `Option`
- Prefer `Box<dyn Error>` for app-level errors
- Use `?` operator for error propagation

```rust
pub fn new() -> Result<Self, Box<dyn Error>> {
    let terminal = Terminal::new(backend)?;
    Ok(Self { terminal })
}
```

### Documentation

- Add `///` for public APIs
- Describe what and why, not just what
- Include parameter/return types

### Error Handling

- Use early returns with `?` operator
- Never use empty catch blocks
- Provide meaningful error messages with context
- Use `tracing` for logging (not `println!`)

### Logging

- Use `tracing` crate (levels: `trace!`, `debug!`, `info!`, `warn!`, `error!`)
- Initialize in main: `tracing_subscriber::fmt().with_max_level(Level::INFO).init()`
- Log async operations and state changes

## Architecture Principles

### Strict Separation

- **Core (`fluxion-core`)**: Headless, no UI/terminal knowledge
- **Frontend (`fluxion-tui`)**: Renders state, maps input to actions
- **App (`fluxion`)**: CLI entry point using clap

### Data Flow

```
User Input (TUI) → Action → Core State Update → TUI Render
```

Core never knows about pixels/terminals. TUI never directly mutates Core state without Actions.

### Virtual Buffers Concept

Everything is text: file system operations, Git/JJ operations, file renames are performed by editing text buffers (Oil.nvim style).

## Tech Stack

- **Runtime:** `tokio` for async I/O
- **Text Data:** `ropey` for immutable rope data structure
- **TUI:** `ratatui` for rendering
- **Input:** `crossterm` for cross-platform terminal input
- **CLI:** `clap` with derive feature
- **Logging:** `tracing` + `tracing-subscriber`

## Workspace Structure

```
fluxion/
├── Cargo.toml (workspace root)
├── crates/
│   ├── core/    # Headless engine, state machine
│   ├── tui/     # Terminal interface, rendering
│   └── app/     # Binary entry point, CLI
└── docs/        # Architecture and project docs
```

## Testing

- Write unit tests with `#[test]` attribute
- Write integration tests in `tests/` module
- Mock external dependencies (terminal, file system)
- Test error paths, not just happy paths
- Run `cargo test` before committing

## Anti-Patterns (BLOCKING)

- Never suppress type errors
- Never use `unwrap()` except in test code
- Never use `panic!` in library code
- Never mix UI logic in Core
- Never bypass the Action-based state machine
- Never use `println!` (use `tracing`)
