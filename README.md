# seekX

`seekX` is a Rust-based Linux app launcher inspired by Walker's keyboard-first flow.

## Features

- Fast `.desktop` app discovery from user/system/Flatpak locations
- Fuzzy search ranking for names, comments, categories, and keywords
- Keyboard-first UX: `Up/Down`, `Enter`, `Alt+Enter`, `Esc`
- Lightweight native desktop UI with `egui/eframe`

## Project layout

- `src/main.rs`: app boot and native window configuration
- `src/desktop.rs`: `.desktop` scanning and parsing
- `src/search.rs`: fuzzy matching/scoring engine
- `src/launcher.rs`: process launch and web search fallback
- `src/ui.rs`: launcher panel UI and interaction logic

## Run

```bash
cd ~/project/seekX
cargo run
```

## Controls

- `Type`: filter app list
- `Enter`: launch selected app
- `Alt+Enter`: open web search with current query
- `Enter` (with no matching apps): open web search with current query
- `Esc`: close launcher

## Notes

- This is a strong MVP foundation for adding Wayland layer-shell behavior, icon rendering, app usage history weighting, and plugin support.
