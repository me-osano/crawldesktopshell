# Changelog

## [Unreleased]

### Added
- **Predefined Scheme Expansion (Rust)** — `expand_predefined_scheme()` in `crawl-theme::dynamic::generate` converts 14-color schemes (`cPrimary`, `cOnPrimary`, etc.) to full 48-key MD3 palette. Replaces Python `expand_predefined_scheme`.
- **Wallust-style Theme Engines (Rust)** — `generate_normal_dark/light()` (vibrant/faithful/dysfunctional), `generate_muted_dark/light()` (muted), and `generate_theme()` dispatcher in `crawl-theme::dynamic::generate`. Includes `collect_theme_map()` for building the 48-key palette.
- **Terminal Theme Generation (Rust)** — `TerminalColors`, `TerminalGenerator`, and `write_terminal_themes()` in `crawl-theme::apply::terminal`. Supports foot (INI), ghostty (key=value), kitty (key value), alacritty (TOML), wezterm (TOML with tab bar + indexed colors).
- **IPC: `ThemeGenerateFromPredefined`** — daemon command to expand a 14-color scheme JSON to a 48-key palette map.
- **IPC: `ThemeGenerateTerminal`** — daemon command to write terminal config files from scheme JSON terminal section + output mapping.
- **IPC handlers** — `ThemeService` in `crawl-sysd` handles both new commands.

### Changed
- **`TemplateProcessor.qml::handleTerminalThemesGenerate`** — terminal config generation now calls daemon IPC (`ThemeGenerateTerminal`) instead of spawning `python3 template-processor.py`.
- **`TemplateProcessor.qml::executePredefinedScheme`** — palette expansion now calls daemon IPC (`ThemeGenerateFromPredefined`) instead of Python `--scheme` flag. Expanded 48-color palette is passed to Python as positional JSON arg for template rendering.
- **`TemplateProcessor.qml::buildUserTemplateCommandForPredefined`** — updated to pass expanded palette JSON as positional arg instead of `--scheme` flag.
- **`crawl-sysd/src/daemon.rs`** — fixed 23 missing import errors (pre-existing).

### Fixed
- `handleTerminalThemesGenerate` — replaced undefined `root._send()` call with `CrawlService.sendRequest()`.

### Tests
- Added 13 unit tests in `crawl-theme::dynamic::generate::tests` covering `interpolate_color`, `expand_predefined_scheme` (dark/light/missing key), and `generate_theme` (all scheme types).
