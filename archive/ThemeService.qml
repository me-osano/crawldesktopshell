pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import QtQml

Singleton {
  id: root

  // ── Config ──────────────────────────────────────────────────────────────
  property string themePath: {
    const cacheHome = Quickshell.env("XDG_CACHE_HOME")
      || `${Quickshell.env("HOME")}/.cache`
    return `${cacheHome}/crawl/current-theme.json`
  }

  // ── Raw theme data (for Theme.qml reactive bindings) ────────────────────
  property var currentTheme: ({})
  property string currentVariant: "dark"
  property bool isDark: true

  // ── Theme list + current metadata ────────────────────────────────────────
  property var themesList: []
  property string currentThemeName: ""
  property string currentThemeSource: ""
  property string currentThemeScheme: ""

  // ── Theme mode: dark / light ─────────────────────────────────────────────
  property string mode: "dark"

  // ── Dynamic theme state ──────────────────────────────────────────────────
  property bool hasDynamic: false
  property var dynamicTheme: ({})

  // ── M3 Color Roles ───────────────────────────────────────────────────────
  property string primary: ""
  property string onPrimary: ""
  property string secondary: ""
  property string onSecondary: ""
  property string tertiary: ""
  property string onTertiary: ""
  property string error: ""
  property string onError: ""
  property string surface: ""
  property string onSurface: ""
  property string surfaceVariant: ""
  property string onSurfaceVariant: ""
  property string outline: ""
  property string shadow: ""
  property string hover: ""
  property string onHover: ""

  // ── Terminal colors ─────────────────────────────────────────────────────
  property var terminalNormal: ({})
  property var terminalBright: ({})
  property string terminalForeground: ""
  property string terminalBackground: ""
  property string terminalSelectionFg: ""
  property string terminalSelectionBg: ""
  property string terminalCursor: ""
  property string terminalCursorText: ""

  // ── Raw parsed data (for direct access) ─────────────────────────────────
  property var data: ({})

  // ── File cache (fast startup, no-daemon fallback) ───────────────────────
  FileView {
    id: fileView
    path: root.themePath
    printErrors: false
    watchChanges: true
    onLoaded: {
      if (!CrawlService.connected) root._fromFile()
    }
    onFileChanged: {
      if (!CrawlService.connected) root._fromFile()
    }
  }

  // ── Subscribe to CrawlService singleton ─────────────────────────────────
  Component.onCompleted: {
    CrawlService.themeChanged.connect(root._onThemeEvent)
    CrawlService.connectedChanged.connect(root._onServiceConnected)
    if (CrawlService.connected) root._fetchFromDaemon()
  }

  function _onServiceConnected() {
    if (CrawlService.connected) root._fetchFromDaemon()
  }

  // ── Handle theme events from daemon ─────────────────────────────────────
  // ThemeEvent::Changed only carries metadata (name, source, scheme),
  // so we must fetch the full theme data via getTheme().
  function _onThemeEvent(eventData) {
    root._fetchFromDaemon()
  }

  function _fetchFromDaemon() {
    if (!CrawlService.connected) return
    CrawlService.getTheme(function(resp) {
      if (resp && !resp.error) {
        root._applyData(resp)
        root.hasDynamic = resp.metadata?.source === "generated"
          || resp.metadata?.scheme !== "predefined"
        if (root.hasDynamic) root.dynamicTheme = resp
      }
    })
    CrawlService.listThemes(function(resp) {
      if (resp && resp.themes) root.themesList = resp.themes
    })
  }

  // ── File cache fallback ─────────────────────────────────────────────────
  function _fromFile() {
    try {
      const parsed = JSON.parse(fileView.text())
      if (parsed && parsed.metadata) root._applyData(parsed)
    } catch (e) {}
  }

  // ── Apply theme data (common path for IPC and file) ─────────────────────
  function _applyData(themeData) {
    root.data = themeData
    root.currentTheme = themeData
    root.currentThemeName = themeData.metadata?.name ?? ""
    root.currentThemeSource = themeData.metadata?.source ?? ""
    root.currentThemeScheme = themeData.metadata?.scheme ?? ""
    root._updateColors()
  }

  // ── Refresh colors when mode toggles ────────────────────────────────────
  onModeChanged: {
    root.currentVariant = root.mode
    root.isDark = root.mode === "dark"
    root._updateColors()
  }

  function _updateColors() {
    const d = root.data
    const modeData = d[root.mode] || {}
    const colors = modeData.colors || {}

    root.primary          = colors.primary          ?? ""
    root.onPrimary        = colors.on_primary       ?? ""
    root.secondary        = colors.secondary        ?? ""
    root.onSecondary      = colors.on_secondary     ?? ""
    root.tertiary         = colors.tertiary         ?? ""
    root.onTertiary       = colors.on_tertiary      ?? ""
    root.error            = colors.error            ?? ""
    root.onError          = colors.on_error         ?? ""
    root.surface          = colors.surface          ?? ""
    root.onSurface        = colors.on_surface       ?? ""
    root.surfaceVariant   = colors.surface_variant  ?? ""
    root.onSurfaceVariant = colors.on_surface_variant ?? ""
    root.outline          = colors.outline          ?? ""
    root.shadow           = colors.shadow           ?? ""
    root.hover            = colors.hover            ?? ""
    root.onHover          = colors.on_hover         ?? ""

    const term = modeData.terminal || {}
    root.terminalNormal      = term.normal      ?? {}
    root.terminalBright      = term.bright      ?? {}
    root.terminalForeground  = term.foreground  ?? ""
    root.terminalBackground  = term.background  ?? ""
    root.terminalSelectionFg = term.selection_fg ?? ""
    root.terminalSelectionBg = term.selection_bg ?? ""
    root.terminalCursor      = term.cursor      ?? ""
    root.terminalCursorText  = term.cursor_text ?? ""
  }

  // ── Dynamic theme generation ────────────────────────────────────────────
  function generateFromColor(color, scheme) {
    CrawlService.generateTheme(color, scheme || null, function(resp) {
      if (resp && !resp.error) {
        root._applyData(resp)
        root.hasDynamic = true
        root.dynamicTheme = resp
      }
    })
  }

  // ── Public: force refresh ───────────────────────────────────────────────
  function refresh() {
    root._fetchFromDaemon()
    fileView.reload()
  }
}
