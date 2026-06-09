# RivuTV / 影舟 — AI Agent Instructions

## Project Overview
RivuTV is a Linux-native TVBox/CatVod-compatible media client built in Rust.
It fetches source configuration from TVBox-format JSON URLs (e.g., 饭太硬-style),
then uses those sources to browse, search, and play video content.

## Architecture
- **Workspace**: 6 crates (`rivu-core`, `rivu-config`, `rivu-spider`, `rivu-player`, `rivu-ui`, + root binary `rivutv`)
- **UI**: ratatui (TUI, primary) — no GUI framework unless explicitly requested
- **Player**: mpv spawned as subprocess (primary)
- **HTTP**: reqwest async HTTP client
- **Config**: serde_json for all serialization
- **Async**: tokio runtime

## Data Flow
```
Source URL (1) → Config Loader → Site Objects
Site selection → SiteApi.homeContent() → Categories + Featured
Category browse → SiteApi.categoryContent() → Vod list
Vod detail → SiteApi.detailContent() → Flags + Episodes
Episode play → SiteApi.playerContent() → Play URL
Play URL → Source.extract() → mpv playback
```

## TVBox Protocol
The source JSON format follows the standard TVBox spec:
- `sites[]` — VOD source definitions (key, name, api, type, jar, ext, etc.)
- `lives[]` — Live TV source definitions
- `parses[]` — Jiexi/parse definitions
- `rules[]`, `headers[]`, `proxy[]`, `doh[]`, `flags[]`, `ads[]` — various settings

API modes for sites:
- Type 0/1: HTTP XML/JSON API (direct HTTP calls)
- Type 2: JSON API (same as type 1)
- Type 3: Spider plugin (custom code — Rust trait impl, JS/Python in future)
- Type 4: API mode (HTTP with `ac=...` params)

Response types from all APIs use the `Result` wrapper:
```json
{
  "class": [{"type_id": "1", "type_name": "Movie"}],
  "list": [{"vod_id": "123", "vod_name": "...", "vod_pic": "...", "vod_remarks": "..."}],
  "filters": [...]
}
```

## Coding Conventions
- **No comments** in code unless the logic is genuinely non-obvious
- **Minimal code** — prefer concise idiomatic Rust over verbosity
- **Trait-first design** — define behavior in traits, implement concretely
- **All public APIs return Result** — no panics in library code
- **Keep file scope tight** — one responsibility per file, split by domain not layer
- **Use `thiserror`** for error types in new crates
- **Use `derive`** liberally — Debug, Clone, Serialize, Deserialize, Default where sensible
- **Async functions** for I/O (reqwest, file ops), sync for parsing/computation
- **NO unwrap/expect in library code** — only in test code and binary entrypoint

## Testing
- `cargo test` — run all tests
- Tests live in `tests/` at crate level, or inline `#[cfg(test)] mod tests` in source files
- Integration tests at workspace root in `tests/` directory
- Use `rstest` for parameterized tests if needed

## Git Conventions
- Commit messages: `type: short description` (feat, fix, refactor, docs, test, chore)
- NEVER commit source URLs (饭太硬 etc.) to the repository
- NEVER commit API keys or tokens

## Sensitive Data
The file `sources.toml` or any file containing concrete TVBox source URLs
is `.gitignore`'d. Users configure their own sources locally.
