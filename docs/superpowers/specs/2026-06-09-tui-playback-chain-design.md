# TUI Playback Chain — Design Spec

## Overview

Connect the TUI (ratatui) to SiteApi/SpiderEngine/MpvBackend so the user can browse
sources → categories → videos → play in a seamless interactive flow.

## Architecture

```
App (event loop)
 ├── HomeScreen (3-panel: Sources | Categories | Vod List)
 ├── DetailScreen (vod metadata + episode list)
 ├── SearchScreen (query input + results)
 ├── SiteApi (async HTTP → block_on in event loop)
 ├── SourceExtractor (strip video:// etc.)
 └── MpvBackend (subprocess spawn)
```

## Component Details

### 1. App Extensions

**New fields:**
- `api: SiteApi` — shared across screens, created once
- `player: MpvBackend` — created once

**Event loop changes** (lines affected: `App::run_loop`):
- On startup, call `SiteApi::home()` for the first site → populate HomeScreen categories + vod list
- On Enter in Sources panel → call `SiteApi::home()` for selected site
- On Enter in Categories panel → call `SiteApi::category()` → populate vod list panel
- On Enter in Vod List panel → call `SiteApi::detail()` → switch to DetailScreen
- In DetailScreen, on episode Enter → call `SiteApi::play()` → `SourceExtractor::extract()` → `MpvBackend::play()`
- `←/→` ArrowLeft/ArrowRight switches focus panel in Home

All async calls use `tokio::runtime::Handle::current().block_on()` — simple, blocking, acceptable for MVP.

### 2. HomeScreen 3-Panel Layout

```
┌─────────────┬──────────────┬─────────────────────────┐
│  Sources    │  Categories  │  Vod List               │
│             │              │                         │
│  > Site A   │  > Movie     │  > Movie 1 (2024) HD    │
│    Site B   │    TV Series │    Movie 2 (2023) 4K    │
│    Site C   │    Anime     │    Movie 3 (2025)       │
│             │    Variety   │    Movie 4              │
│             │              │    Movie 5              │
│             │              │                         │
│   panel 0   │   panel 1    │   panel 2              │
│  [30%]      │  [25%]       │  [45%]                 │
└─────────────┴──────────────┴─────────────────────────┘
```

**Panel widths:** 30% / 25% / 45%

**New struct fields on HomeScreen:**
- `focus: usize` — which panel is active (0, 1, or 2)
- `categories_selected: usize` — selected category index
- `vod_list: Vec<Vod>` — current category's vod list
- `vod_selected: usize` — selected vod index
- `loading: bool` — showing "Loading..." during API calls

### 3. Keyboard Mapping

| Key | Action |
|-----|--------|
| `j`/`↓` | Move selection down in active panel |
| `k`/`↑` | Move selection up in active panel |
| `h`/`←` | Move focus to previous panel (Home) |
| `l`/`→` | Move focus to next panel (Home) |
| `Enter` | Act on selected item (load cats/vods/detail/play) |
| `Esc` | Go back (detail→home, search→home) |
| `/` | Enter search mode |
| `q` | Quit |

In DetailScreen, `h`/`l` or `←`/`→` switches flag group (source quality tabs), `j`/`k` navigates episodes.

### 4. Data Flow

```
SiteApi::home(site) ──→ ApiResult { class, list }
                            ├── class[] → Categories panel
                            └── list[]  → Vod List panel (featured)

SiteApi::category(site, tid, pg=1, []) ──→ ApiResult { list }
                            └── list[]  → Vod List panel

SiteApi::detail(site, &[vod_id]) ──→ ApiResult { list[0], ... }
                            └── list[0] → DetailScreen.vod
                                parse flags from vod_play_from/vod_play_url

SiteApi::play(site, flag, id) ──→ ApiResult { url, header }
                            └── PlayInfo → SourceExtractor → MpvBackend
```

### 5. Error Handling

- API calls failing → show `"Error: {msg}"` in the relevant panel
- Empty responses → show `"(empty)"` in the panel
- mpv not found → error message, stay in UI
- Loading state → panel shows `"Loading..."` while fetching

### 6. Poker/Blocking Approach

No channels. `Handle::current().block_on()` wraps each async SiteApi call.
This blocks the terminal UI for the duration of the HTTP request (typically
200-500ms). Acceptable for MVP. Can be upgraded to tokio::spawn + channels later.

### 7. Search Integration

SearchScreen keeps current behavior. When user types and presses Enter:
1. `block_on(api.search(site, &query, 1))`
2. Results populate SearchScreen.results
3. Enter on a result → `detail()` → switch to DetailScreen

### 8. Test Strategy

- Unit tests for HomeScreen focus/switching logic
- Unit tests for DetailScreen flag navigation
- Mock-based tests for App event handler function (separate state mutation from I/O)
- Integration: verify full flow with recorded JSON fixtures

### 9. Files Changed

| File | Change |
|------|--------|
| `rivu-ui/src/app.rs` | Add SiteApi, MpvBackend, block_on handlers, keyboard routing |
| `rivu-ui/src/screens/home.rs` | 3-panel layout, focus, vod_list, category/vod selection |
| `rivu-ui/src/screens/detail.rs` | Flag switching, play trigger callback |
| `rivu-ui/src/screens/search.rs` | Enter triggers API search |
| `rivu-ui/Cargo.toml` | Add rivu-spider, rivu-player deps |
| `src/main.rs` | Pass SiteApi+MpvBackend into App |

## Scope

In scope:
- 3-panel Home browsing with working API calls
- Detail view with episode selection
- Playback via mpv
- Basic search
- Loading/error states

Out of scope:
- Pagination (stick to pg=1 for now)
- Persistent site/category state across sessions
- Live TV playback
- Keyboard config remapping
