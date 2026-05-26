pragma Singleton
import Qt.labs.folderlistmodel

import QtQuick
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Services

Singleton {
  id: root

  // ── Scheme Discovery (from ColorSchemeService) ──────────────────────
  property var schemes: []
  property bool scanning: false
  property string schemesDirectory: Quickshell.shellDir + "/Assets/ColorScheme"
  property string downloadedSchemesDirectory: Settings.configDir + "colorschemes"
  property string colorsJsonFilePath: Settings.configDir + "colors.json"

  // ── Daemon Theme State (from Pattern A IPC bridge) ──────────────────
  property var currentTheme: null      // ThemeChangedEvent from daemon
  property var currentPalette: ({})    // Full palette map from daemon
  property string lastError: ""

  // ── Signals ─────────────────────────────────────────────────────────
  signal themeChanged(string name, string source, string scheme, var palette)
  signal themeApplied(string app, string path)
  signal themeGenerated(string name, string color)

  // ── Dark mode: re-apply current scheme on toggle ────────────────────
  Connections {
    target: Settings.data.colorSchemes
    function onDarkModeChanged() {
      Logger.d("Theme", "Detected dark mode change");
      if (!Settings.data.colorSchemes.useWallpaperColors && Settings.data.colorSchemes.predefinedScheme) {
        applyScheme(Settings.data.colorSchemes.predefinedScheme);
      }
      const enabled = !!Settings.data.colorSchemes.darkMode;
      ToastService.showNotice(enabled ? "Dark Mode" : "Light Mode", "Enabled", "dark-mode");
    }
  }

  // ── Daemon Event Listener (Pattern A) ───────────────────────────────
  Component.onCompleted: {
    if (typeof CrawlService !== "undefined" && CrawlService) {
      CrawlService.themeChanged.connect(root._dispatchDaemonEvent);
    }
  }

  function _dispatchDaemonEvent(data) {
    switch (data.event) {
      case "changed":
        root.currentTheme = data;
        root.currentPalette = data.palette || {};
        root.themeChanged(data.name, data.source, data.scheme, data.palette);
        // Trigger template generation for daemon-originated changes
        if (root.hasEnabledTemplates()) {
          AppThemeService.generate();
        }
        break;
      case "applied":
        root.themeApplied(data.app, data.path);
        break;
      case "generated":
        root.themeGenerated(data.name, data.color);
        break;
    }
  }

  // ── Scheme Scanning ─────────────────────────────────────────────────
  function init() {
    Logger.i("Theme", "Service started");
    loadColorSchemes();
  }

  function loadColorSchemes() {
    Logger.d("Theme", "Load color schemes");
    scanning = true;
    schemes = [];
    Quickshell.execDetached(["mkdir", "-p", downloadedSchemesDirectory]);
    findProcess.command = ["find", "-L", schemesDirectory, downloadedSchemesDirectory, "-mindepth", "2", "-name", "*.json", "-type", "f"];
    findProcess.running = true;
  }

  function getBasename(path) {
    if (!path)
      return "";
    var chunks = path.split("/");
    var filename = chunks[chunks.length - 1];
    var schemeName = filename.replace(".json", "");
    if (schemeName === "Crawlds") {
      return "Crawlds (default)";
    } else if (schemeName === "Crawlds-legacy") {
      return "CrawlDS (legacy)";
    } else if (schemeName === "Tokyo-Night") {
      return "Tokyo Night";
    } else if (schemeName === "Rosepine") {
      return "Rose Pine";
    }
    return schemeName;
  }

  function resolveSchemePath(nameOrPath) {
    if (!nameOrPath)
      return "";
    if (nameOrPath.indexOf("/") !== -1) {
      return nameOrPath;
    }
    var schemeName = nameOrPath.replace(".json", "");
    if (schemeName === "Crawlds (default)") {
      schemeName = "Crawlds";
    } else if (schemeName === "CrawlDS (legacy)") {
      schemeName = "CrawlDS-legacy";
    } else if (schemeName === "Tokyo Night") {
      schemeName = "Tokyo-Night";
    } else if (schemeName === "Rose Pine") {
      schemeName = "Rosepine";
    }
    var preinstalledPath = schemesDirectory + "/" + schemeName + "/" + schemeName + ".json";
    var downloadedPath = downloadedSchemesDirectory + "/" + schemeName + "/" + schemeName + ".json";
    for (var i = 0; i < schemes.length; i++) {
      if (schemes[i].indexOf("/" + schemeName + "/") !== -1 || schemes[i].indexOf("/" + schemeName + ".json") !== -1) {
        return schemes[i];
      }
    }
    return preinstalledPath;
  }

  // ── Daemon mapping: local basename → daemon ThemeSet name ────────────
  property var _knownVariants: ({
    "mocha": true,
    "latte": true,
    "frappe": true,
    "macchiato": true,
    "nord": true,
    "tokyo night": "tokyo-night",
    "rose pine": "rose-pine-light",
    "gruvbox": "gruvbox-light",
    "kanagawa": "kanagawa-light"
  })

  function _daemonNameFor(name) {
    var lower = name.toLowerCase();
    // Direct match
    if (_knownVariants[lower] === true) return lower;
    // Aliased match
    if (_knownVariants[lower]) return _knownVariants[lower];
    // grubox/kanagawa with dark/light suffix
    if (lower.indexOf("gruvbox") !== -1) return lower.replace(" ", "-");
    if (lower.indexOf("kanagawa") !== -1) return lower.replace(" ", "-");
    if (lower.indexOf("rose pine") !== -1) return lower.replace(" ", "-");
    return null;
  }

  // ── IPC: send request through CrawlService ───────────────────────────
  function _send(method, params, callback) {
    if (typeof CrawlService !== "undefined" && CrawlService && CrawlService.connected) {
      CrawlService.sendRequest(method, params || {}, function(result) {
        if (result && result.error) {
          root.lastError = result.error.message || "Theme error";
          if (callback) callback(null, result.error);
        } else {
          root.lastError = "";
          if (callback) callback(result || {}, null);
        }
      });
    } else {
      root.lastError = "CrawlService not connected";
      if (callback) callback(null, { error: { message: root.lastError } });
    }
  }

  // ── Apply Scheme (handles both predefined and custom) ────────────────
  function applyScheme(nameOrPath) {
    var filePath = resolveSchemePath(nameOrPath);
    schemeReader.path = "";
    schemeReader.path = filePath;
  }

  function setPredefinedScheme(schemeName) {
    Logger.i("Theme", "Setting predefined scheme:", schemeName);

    var resolvedPath = resolveSchemePath(schemeName);
    var basename = getBasename(schemeName);

    var schemeExists = false;
    for (var i = 0; i < schemes.length; i++) {
      if (getBasename(schemes[i]) === basename) {
        schemeExists = true;
        break;
      }
    }

    if (schemeExists) {
      Settings.data.colorSchemes.predefinedScheme = basename;
      applyScheme(schemeName);
      ToastService.showNotice("Color Scheme", basename, "settings-color-scheme");
    } else {
      Logger.e("Theme", "Scheme not found:", schemeName);
      ToastService.showError("Color Scheme", `'${basename}' Not found`);
    }
  }

  // ── Internal scheme loader ───────────────────────────────────────────
  Process {
    id: findProcess
    running: false

    onExited: function (exitCode) {
      if (exitCode === 0) {
        var output = stdout.text.trim();
        var files = output.split('\n').filter(function (line) {
          return line.length > 0;
        });
        files.sort(function (a, b) {
          var nameA = getBasename(a).toLowerCase();
          var nameB = getBasename(b).toLowerCase();
          return nameA.localeCompare(nameB);
        });
        schemes = files;
        scanning = false;
        Logger.d("Theme", "Listed", schemes.length, "schemes");
        var stored = Settings.data.colorSchemes.predefinedScheme;
        if (stored) {
          var basename = getBasename(stored);
          if (basename !== stored) {
            Settings.data.colorSchemes.predefinedScheme = basename;
          }
          if (!Settings.data.colorSchemes.useWallpaperColors) {
            applyScheme(basename);
          }
        }
      } else {
        Logger.e("Theme", "Failed to find color scheme files");
        schemes = [];
        scanning = false;
      }
    }

    stdout: StdioCollector {}
    stderr: StdioCollector {}
  }

  // ── Read scheme JSON and apply ──────────────────────────────────────
  FileView {
    id: schemeReader
    onLoaded: {
      try {
        var data = JSON.parse(text());
        var variant = data;
        if (data && (data.dark || data.light)) {
          if (Settings.data.colorSchemes.darkMode) {
            variant = data.dark || data.light;
          } else {
            variant = data.light || data.dark;
          }
        }

        // 1. Write colors.json for Theme.qml to pick up
        root.writeColorsToDisk(variant);
        Logger.i("Theme", "Applied color scheme:", getBasename(path));

        // 2. If this is a known daemon variant, sync system themes
        var name = getBasename(path);
        var daemonName = root._daemonNameFor(name);
        if (daemonName) {
          Logger.d("Theme", "Syncing daemon theme to:", daemonName);
          root._send("ThemeSet", { name: daemonName }, function(result, error) {
            if (error) {
              Logger.w("Theme", "Daemon theme sync failed:", error);
            }
          });
        }

        // 3. Generate templates
        if (root.hasEnabledTemplates()) {
          AppThemeService.generateFromPredefinedScheme(data);
        }
      } catch (e) {
        Logger.e("Theme", "Failed to parse scheme JSON:", path, e);
      }
    }
  }

  // ── Write colors.json ───────────────────────────────────────────────
  FileView {
    id: colorsWriter
    path: colorsJsonFilePath
    printErrors: false
    onSaved: {}

    JsonAdapter {
      id: out
      property color cPrimary: "#000000"
      property color cOnPrimary: "#000000"
      property color cSecondary: "#000000"
      property color cOnSecondary: "#000000"
      property color cTertiary: "#000000"
      property color cOnTertiary: "#000000"
      property color cError: "#000000"
      property color cOnError: "#000000"
      property color cSurface: "#000000"
      property color cOnSurface: "#000000"
      property color cSurfaceVariant: "#000000"
      property color cOnSurfaceVariant: "#000000"
      property color cOutline: "#000000"
      property color cShadow: "#000000"
      property color cHover: "#000000"
      property color cOnHover: "#000000"
    }
  }

  function writeColorsToDisk(obj) {
    function pick(o, a, b, fallback) {
      return (o && (o[a] || o[b])) || fallback;
    }
    out.cPrimary = pick(obj, "cPrimary", "primary", out.cPrimary);
    out.cOnPrimary = pick(obj, "cOnPrimary", "onPrimary", out.cOnPrimary);
    out.cSecondary = pick(obj, "cSecondary", "secondary", out.cSecondary);
    out.cOnSecondary = pick(obj, "cOnSecondary", "onSecondary", out.cOnSecondary);
    out.cTertiary = pick(obj, "cTertiary", "tertiary", out.cTertiary);
    out.cOnTertiary = pick(obj, "cOnTertiary", "onTertiary", out.cOnTertiary);
    out.cError = pick(obj, "cError", "error", out.cError);
    out.cOnError = pick(obj, "cOnError", "onError", out.cOnError);
    out.cSurface = pick(obj, "cSurface", "surface", out.cSurface);
    out.cOnSurface = pick(obj, "cOnSurface", "onSurface", out.cOnSurface);
    out.cSurfaceVariant = pick(obj, "cSurfaceVariant", "surfaceVariant", out.cSurfaceVariant);
    out.cOnSurfaceVariant = pick(obj, "cOnSurfaceVariant", "onSurfaceVariant", out.cOnSurfaceVariant);
    out.cOutline = pick(obj, "cOutline", "outline", out.cOutline);
    out.cShadow = pick(obj, "cShadow", "shadow", out.cShadow);
    out.cHover = pick(obj, "cHover", "hover", out.cHover);
    out.cOnHover = pick(obj, "cOnHover", "onHover", out.cOnHover);

    colorsWriter.path = "";
    colorsWriter.path = colorsJsonFilePath;
    colorsWriter.writeAdapter();
  }

  // ── Template check ──────────────────────────────────────────────────
  function hasEnabledTemplates() {
    const activeTemplates = Settings.data.templates.activeTemplates;
    if (!activeTemplates || activeTemplates.length === 0) {
      return false;
    }
    for (let i = 0; i < activeTemplates.length; i++) {
      if (activeTemplates[i].enabled) {
        return true;
      }
    }
    return false;
  }

  // ── IPC Commands (Pattern A) ────────────────────────────────────────
  function setTheme(name, accent, callback) {
    var params = { name: name };
    if (accent) params.accent = accent;
    root._send("ThemeSet", params, callback);
  }

  function generateFromColor(color, scheme, mode, callback) {
    var params = { color: color };
    if (scheme) params.scheme = scheme;
    if (mode) params.mode = mode;
    root._send("ThemeGenerate", params, callback);
  }

  function generateFromImage(path, scheme, mode, callback) {
    var params = { path: path };
    if (scheme) params.scheme = scheme;
    if (mode) params.mode = mode;
    root._send("ThemeGenerateFromImage", params, callback);
  }

  function listThemes(callback) {
    root._send("ThemeList", {}, callback);
  }

  function getCurrent(callback) {
    root._send("ThemeGet", {}, callback);
  }
}
