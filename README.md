# RivuTV / 影舟

A Linux-native TVBox/CatVod-compatible media client built with Rust.

## Why 影舟?

- **舟** — A vessel carrying media streams to your screen
- **Rivu** — Echoes *river* / *rivulet*, fitting for stream aggregation
- Native Linux support, no Android emulation needed

## Project Structure

```
rivutv/
├── src/                 # CLI binary (rivu command)
├── crates/
│   ├── rivu-core/      # Core types, traits, and data models
│   ├── rivu-config/    # Configuration management
│   ├── rivu-spider/    # Spider engine for TVBox APIs
│   ├── rivu-player/    # Media playback backends (mpv, etc.)
│   └── rivu-ui/        # User interface (TUI/GUI)
```

## Usage

```bash
# Launch the interactive UI
rivu run

# Search media
rivu search "keyword"

# Play a URL directly
rivu play "https://example.com/stream.m3u8"
```

## License

MIT
