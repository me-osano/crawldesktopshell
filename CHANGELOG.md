# Changelog

## [Unreleased]

### Added
- **Clipboard Backend (Rust IPC)** — `ClipboardBackend` in `crawl-sysd` with SQLite persistence (via `sqlx 0.7`), FNV-1a dedup, 200ms polling monitor via `wl-clipboard-rs`, event broadcasting, and in-memory LRU cache. Replaces old QML+cliphist shell-pipe approach.
- **Clipboard IPC: 8 commands** — `ClipboardList`, `ClipboardGetContent`, `ClipboardCopy`, `ClipboardDelete`, `ClipboardWipe`, `ClipboardPin`, `ClipboardPasteText`, `ClipboardSet` in `crawl-ipc` commands enum.
- **Clipboard IPC: 4 events** — `Changed`, `Deleted`, `Cleared`, `Pinned` in `ClipboardEvent` enum, dispatched as `CrawlEvent::Clipboard`.
- **Clipboard IPC types** — `ClipEntry` (history item with id, preview, mime, flags) and `ClipContent` (payload with optional base64 image data) in `crawl-ipc`.
- **`ClipboardService` (daemon)** — Registers with `Service` trait, initialises DB + cache on start, runs monitor thread, handles all 8 clipboard IPC methods.
- **`ClipboardService.qml` (rewrite)** — Replaced cliphist shell commands with `CrawlService.sendRequest` IPC calls for all operations (list, decode, copy, paste, delete, wipe).
- **`CrawlService.qml` clipboard wrappers** — 8 convenience methods (`clipboardList`, `clipboardGetContent`, `clipboardCopy`, etc.) dispatching to backend IPC.
- **`ClipboardPanel.qml` (standalone panel)** — Rewired to use IPC-based `ClipboardService`; removed cliphist dependency checks.
- **`core/docs/CLIPBOARD.md`** — Comprehensive documentation of the clipboard backend architecture, IPC protocol, storage layer, monitor thread, and QML integration.
- **Predefined Scheme Expansion (Rust)** — `expand_predefined_scheme()` in `crawl-theme::dynamic::generate` converts 14-color schemes (`cPrimary`, `cOnPrimary`, etc.) to full 48-key MD3 palette. Replaces Python `expand_predefined_scheme`.
- **Wallust-style Theme Engines (Rust)** — `generate_normal_dark/light()` (vibrant/faithful/dysfunctional), `generate_muted_dark/light()` (muted), and `generate_theme()` dispatcher in `crawl-theme::dynamic::generate`. Includes `collect_theme_map()` for building the 48-key palette.
- **Terminal Theme Generation (Rust)** — `TerminalColors`, `TerminalGenerator`, and `write_terminal_themes()` in `crawl-theme::apply::terminal`. Supports foot (INI), ghostty (key=value), kitty (key value), alacritty (TOML), wezterm (TOML with tab bar + indexed colors).
- **IPC: `ThemeGenerateFromPredefined`** — daemon command to expand a 14-color scheme JSON to a 48-key palette map.
- **IPC: `ThemeGenerateTerminal`** — daemon command to write terminal config files from scheme JSON terminal section + output mapping.
- **IPC handlers** — `ThemeService` in `crawl-sysd` handles both new commands.
- **Toast position setting** — `notifications.toastLocation` (default: `"top"`) independent from notification position. Configurable in Settings → Notifications → Toast tab.

### Changed
- **`TemplateProcessor.qml::handleTerminalThemesGenerate`** — terminal config generation now calls daemon IPC (`ThemeGenerateTerminal`) instead of spawning `python3 template-processor.py`.
- **`TemplateProcessor.qml::executePredefinedScheme`** — palette expansion now calls daemon IPC (`ThemeGenerateFromPredefined`) instead of Python `--scheme` flag. Expanded 48-color palette is passed to Python as positional JSON arg for template rendering.
- **`TemplateProcessor.qml::buildUserTemplateCommandForPredefined`** — updated to pass expanded palette JSON as positional arg instead of `--scheme` flag.
- **`crawl-sysd/src/daemon.rs`** — fixed 23 missing import errors (pre-existing).
- **Notification architecture** — QML frontend no longer registers its own `NotificationServer` on D-Bus. `NotificationService.qml` now subscribes to `CrawlService.notificationChanged` events from the Rust backend (`crawl-sysd`), which is the sole owner of `org.freedesktop.Notifications`. All notification lifecycle (history, expiry, deduplication, persistence) is handled by the backend.
- **`NotificationService.qml`** — Rewritten from 1209 to 556 lines. Removed local `NotificationServer`, watcher components, progress timer, `activeNotifications` map, local history/state persistence. Added IPC event dispatcher (`_onNotificationEvent`) for 8 event types, snake→camel case field mapper (`_mapFromBackend`), and bootstrap via `CrawlService.notificationGetState`.
- **Settings tabs** — `DurationSubTab.qml`, `HistorySubTab.qml`, and `GeneralSubTab.qml` now read/write notification policy via `CrawlService.notificationGetPolicy/setPolicy` IPC instead of local `Settings.data.notifications`.
- **Toast location** — `Toast.qml` and `ToastScreen.qml` use `notifications.toastLocation` (default `"top"`) instead of sharing `notifications.location` (default `"top_right"`).

### Removed
- **`ClipboardProvider.qml`** — Removed the clipboard history launcher provider (`>clip` command) from launcher panel.
- **`ClipboardPreview.qml`** — Removed the launcher's clipboard preview component (only used by clipboard provider).
- **`ClipboardSubTab.qml`** — Removed clipboard history settings tab from launcher settings.
- **Launcher preview infrastructure** — Removed `previewActive`/`previewPanelWidth` properties and preview panel CrawlBox from `Launcher.qml` and `LauncherOverlayWindow.qml` (only used by clipboard).
- **Clipboard launcher settings** — Removed `enableClipboardHistory`, `enableClipPreview`, `clipboardWrapText`, `clipboardWatchTextCommand`, `clipboardWatchImageCommand`, `screenshotAnnotationTool` from `Settings.qml` and `settings-default.json`.
- **shell.qml clipboard init** — Removed `ClipboardService.checkCliphistAvailability()` call.
- **Legacy settings** — Removed `cliphist` from `package.nix` runtime deps and `Settings.qml` default watch commands.
- **Backend sound stubs** — Removed `NotificationGetSoundState`/`NotificationSetSoundState` IPC methods from `crawl-sysd` and `crawl-ipc` commands enum. Sound is handled entirely on the frontend.
- **Duplicated notification settings** — Removed `respectExpireTimeout`, `lowUrgencyDuration`, `normalUrgencyDuration`, `criticalUrgencyDuration`, `saveToHistory` from `Settings.qml` and `settings-default.json`. These are managed by the backend's `NotificationPolicy`.

### Fixed
- `handleTerminalThemesGenerate` — replaced undefined `root._send()` call with `CrawlService.sendRequest()`.

### Tests
- Added 13 unit tests in `crawl-theme::dynamic::generate::tests` covering `interpolate_color`, `expand_predefined_scheme` (dark/light/missing key), and `generate_theme` (all scheme types).
