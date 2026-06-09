# CLAUDE.md — RivuTV / 影舟

## Commands
- `cargo check` — verify compilation
- `cargo test` — run all tests
- `cargo clippy` — lint checks
- `rivu run` — launch TUI (after `cargo build`)
- `rivu play <url>` — direct playback

## Code Style
- Concise Rust, no comments unless genuinely necessary
- Re-export public API at crate root lib.rs
- Workspace: each crate has focused responsibility

## Important
- Do NOT commit concrete source config URLs (饭太硬 etc.)
- Do NOT add Android/Java-specific constructs
- Prefer `thiserror` over manual Display impls for error types
- All async I/O via tokio + reqwest

## Key Design Decisions
- mpv subprocess for playback (not a Rust-native decoder)
- ratatui for TUI (not web, not GTK/Qt unless asked)
- TVBox JSON protocol for source configs (100% compatible)
- No database for MVP — memory-only state, JSON file persist
- Source configuration via JSON URL (not hardcoded)

## Modifying Guidelines
Before changing a crate, check its Cargo.toml for dependency scope.
New features need tests. Large features need a plan in docs/plans/.
