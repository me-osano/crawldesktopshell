pragma Singleton

import QtQuick
import QtQuick.Window
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "../Common/Helpers/sha256.js" as Checksum
import qs.Common
import qs.Services

Singleton {
  id: root

  // State synced from backend
  property real lastSeenTs: 0
  property bool doNotDisturb: false

  // Models (populated from IPC events)
  property ListModel activeList: ListModel {}
  property ListModel historyList: ListModel {}

  // Rate limiting for notification sounds (minimum 100ms between sounds)
  property var lastSoundTime: 0
  readonly property int minSoundInterval: 100

  // ── IPC bootstrap ──────────────────────────────────────────────────────

  Component.onCompleted: {
    CrawlService.notificationChanged.connect(root._onNotificationEvent);
    Qt.callLater(root._bootstrap);
  }

  function _bootstrap() {
    CrawlService.notificationGetState(function (state) {
      if (!state || state.error) return;
      root._populateFromState(state);
    });
  }

  function _populateFromState(state) {
    activeList.clear();
    historyList.clear();

    if (state.popups) {
      for (var i = 0; i < state.popups.length; i++) {
        var item = root._mapFromBackend(state.popups[i]);
        activeList.append(item);
        root.queueImage(item.originalImage, item.appName, item.summary, item.id);
      }
    }
    if (state.history) {
      for (var j = 0; j < state.history.length; j++) {
        var hitem = root._mapFromBackend(state.history[j]);
        historyList.append(hitem);
      }
    }
    root.doNotDisturb = state.do_not_disturb || false;
    root.lastSeenTs = state.last_seen_ts || 0;
  }

  // ── Event dispatcher ───────────────────────────────────────────────────

  function _onNotificationEvent(data) {
    if (!data || !data.event) return;
    switch (data.event) {
    case "popup_added":      _onPopupAdded(data.item); break;
    case "popup_updated":    _onPopupUpdated(data.item); break;
    case "popup_removed":    _onPopupRemoved(data.id); break;
    case "history_added":    _onHistoryAdded(data.item); break;
    case "history_removed":  _onHistoryRemoved(data.id); break;
    case "history_cleared":  _onHistoryCleared(); break;
    case "dnd_changed":      _onDndChanged(data.enabled); break;
    case "last_seen_changed": _onLastSeenChanged(data.ts); break;
    }
  }

  function _onPopupAdded(item) {
    var mapped = root._mapFromBackend(item);
    root.queueImage(mapped.originalImage, mapped.appName, mapped.summary, mapped.id);
    activeList.insert(0, mapped);

    if (!item.muted) {
      root.playNotificationSound(item.urgency, item.app_name);
    }
  }

  function _onPopupUpdated(item) {
    var mapped = root._mapFromBackend(item);
    for (var i = 0; i < activeList.count; i++) {
      var existing = activeList.get(i);
      if (existing.id === mapped.id) {
        activeList.setProperty(i, "summary", mapped.summary);
        activeList.setProperty(i, "summaryMarkdown", mapped.summaryMarkdown);
        activeList.setProperty(i, "body", mapped.body);
        activeList.setProperty(i, "bodyMarkdown", mapped.bodyMarkdown);
        activeList.setProperty(i, "appName", mapped.appName);
        activeList.setProperty(i, "urgency", mapped.urgency);
        activeList.setProperty(i, "expireTimeout", mapped.expireTimeout);
        activeList.setProperty(i, "progress", mapped.progress);
        activeList.setProperty(i, "originalImage", mapped.originalImage || existing.originalImage);
        activeList.setProperty(i, "cachedImage", existing.cachedImage);
        activeList.setProperty(i, "actionsJson", mapped.actionsJson);
        activeList.setProperty(i, "originalId", mapped.originalId);
        break;
      }
    }
  }

  function _onPopupRemoved(id) {
    root.animateAndRemove(id);
  }

  function _onHistoryAdded(item) {
    var mapped = root._mapFromBackend(item);
    historyList.insert(0, mapped);
  }

  function _onHistoryRemoved(id) {
    for (var i = 0; i < historyList.count; i++) {
      if (historyList.get(i).id === id) {
        historyList.remove(i);
        break;
      }
    }
  }

  function _onHistoryCleared() {
    historyList.clear();
  }

  function _onDndChanged(enabled) {
    root.doNotDisturb = enabled;
  }

  function _onLastSeenChanged(ts) {
    root.lastSeenTs = ts;
  }

  // ── Field mapping: backend (snake_case) → model (camelCase) ────────────

  function _mapFromBackend(item) {
    if (!item) return null;
    var ts = item.timestamp_ms || Date.now();
    var actions = item.actions || [];
    return {
      "id": item.id || "",
      "summary": root.processNotificationText(item.summary || ""),
      "summaryMarkdown": root.processNotificationMarkdown(item.summary || ""),
      "body": root.processNotificationText(item.body || ""),
      "bodyMarkdown": root.processNotificationMarkdown(item.body || ""),
      "appName": root.getAppName(item.app_name || ""),
      "urgency": item.urgency < 0 || item.urgency > 2 ? 1 : item.urgency,
      "expireTimeout": item.expire_timeout || 0,
      "timestamp": new Date(ts),
      "progress": item.progress !== undefined ? item.progress : 1.0,
      "originalImage": item.original_image || "",
      "cachedImage": item.cached_image || item.original_image || "",
      "originalId": item.original_id || 0,
      "actionsJson": JSON.stringify(actions.map(function (a) {
        return { "text": (a.text || "").trim() || "Action", "identifier": a.identifier || "" };
      }))
    };
  }

  // ── IPC commands (user actions routed to backend) ──────────────────────

  function dismissActiveNotification(id) {
    for (var i = 0; i < activeList.count; i++) {
      if (activeList.get(i).id === id) {
        activeList.remove(i);
        break;
      }
    }
    CrawlService.notificationDismiss(id);
  }

  function dismissAllActive() {
    CrawlService.notificationDismissAll();
  }

  function dismissOldestActive() {
    if (activeList.count > 0) {
      var last = activeList.get(activeList.count - 1);
      dismissActiveNotification(last.id);
    }
  }

  function removeFromHistory(notificationId) {
    var found = false;
    for (var i = 0; i < historyList.count; i++) {
      if (historyList.get(i).id === notificationId) {
        found = true;
        break;
      }
    }
    if (found) {
      CrawlService.notificationRemoveHistory(notificationId);
    }
    return found;
  }

  function removeOldestHistory() {
    if (historyList.count > 0) {
      var oldest = historyList.get(historyList.count - 1);
      CrawlService.notificationRemoveHistory(oldest.id);
      return true;
    }
    return false;
  }

  function clearHistory() {
    CrawlService.notificationClearHistory();
  }

  function invokeAction(id, actionId) {
    CrawlService.notificationInvokeAction(id, actionId);
    return true;
  }

  function invokeActionAndSuppressClose(id, actionId) {
    return invokeAction(id, actionId);
  }

  function updateLastSeenTs() {
    root.lastSeenTs = Date.now();
    CrawlService.notificationSetLastSeen(root.lastSeenTs);
  }

  onDoNotDisturbChanged: {
    CrawlService.notificationSetDnd(root.doNotDisturb);
    ToastService.showNotice(doNotDisturb ? "Do Not Disturb enabled" : "Do Not Disturb disabled", doNotDisturb ? "You'll find these notifications in your history" : "Showing all notifications", doNotDisturb ? "bell-off" : "bell");
  }

  // ── Sound ──────────────────────────────────────────────────────────────

  function playNotificationSound(urgency, appName) {
    if (!SoundService.multimediaAvailable) return;
    if (!Settings.data.notifications?.sounds?.enabled) return;
    if (AudioService.muted) return;

    if (appName) {
      var excludedApps = Settings.data.notifications.sounds.excludedApps || "";
      if (excludedApps.trim() !== "") {
        var excludedList = excludedApps.toLowerCase().split(',').map(function (a) { return a.trim(); });
        if (excludedList.includes(appName.toLowerCase())) return;
      }
    }

    var soundFile = root.getNotificationSoundFile(urgency);
    if (!soundFile || soundFile.trim() === "") return;

    var now = Date.now();
    if (now - root.lastSoundTime < root.minSoundInterval) return;
    root.lastSoundTime = now;

    var volume = Settings.data.notifications?.sounds?.volume ?? 0.5;
    SoundService.playSound(soundFile, { volume: volume, fallback: false, repeat: false });
  }

  function getNotificationSoundFile(urgency) {
    var settings = Settings.data.notifications?.sounds;
    if (!settings) return "";

    var defaultSoundFile = Quickshell.shellDir + "/Assets/Sounds/notification-generic.wav";

    if (!settings.separateSounds) {
      var f = settings.normalSoundFile;
      return (f && f.trim() !== "") ? f : defaultSoundFile;
    }

    var key;
    switch (urgency) {
    case 0:  key = "lowSoundFile"; break;
    case 1:  key = "normalSoundFile"; break;
    case 2:  key = "criticalSoundFile"; break;
    default: key = "normalSoundFile"; break;
    }
    var sf = settings[key];
    return (sf && sf.trim() !== "") ? sf : defaultSoundFile;
  }

  // ── Image caching ──────────────────────────────────────────────────────

  function queueImage(path, appName, summary, notificationId) {
    if (!path || !notificationId) return;
    var filePath = path.startsWith("file://") ? path.substring(7) : path;
    var isImageUri = path.startsWith("image://");
    var isTempFile = (path.startsWith("/") || path.startsWith("file://")) && filePath.startsWith("/tmp/");
    if (!isImageUri && !isTempFile) return;

    ImageCacheService.getNotificationIcon(path, appName, summary, function (cachedPath, success) {
      if (success && cachedPath) {
        root.updateImagePath(notificationId, "file://" + cachedPath);
      }
    });
  }

  function updateImagePath(notificationId, path) {
    root._updateModel(activeList, notificationId, "cachedImage", path);
    root._updateModel(historyList, notificationId, "cachedImage", path);
  }

  function _updateModel(model, notificationId, prop, value) {
    for (var i = 0; i < model.count; i++) {
      if (model.get(i).id === notificationId) {
        model.setProperty(i, prop, value);
        break;
      }
    }
  }

  function generateImageId(notification, image) {
    if (image && image.startsWith("image://")) {
      if (image.startsWith("image://qsimage/")) {
        var key = (notification.appName || "") + "|" + (notification.summary || "");
        return Checksum.sha256(key);
      }
      return Checksum.sha256(image);
    }
    return "";
  }

  // ── Text processing ────────────────────────────────────────────────────

  function processNotificationText(text) {
    if (!text) return "";
    var parts = text.split(/(<[^>]+>)/);
    var result = "";
    var allowedTags = ["b", "i", "u", "a", "br"];
    for (var i = 0; i < parts.length; i++) {
      var part = parts[i];
      if (part.startsWith("<") && part.endsWith(">")) {
        var content = part.substring(1, part.length - 1);
        var firstWord = content.split(/[\s/]/).filter(function (s) { return s.length > 0; })[0]?.toLowerCase();
        if (allowedTags.includes(firstWord)) {
          result += part;
        }
      } else {
        result += root.escapeHtml(part);
      }
    }
    return result;
  }

  function processNotificationMarkdown(text) {
    return root.sanitizeMarkdown(text);
  }

  function sanitizeMarkdown(text) {
    if (!text) return "";
    var input = String(text);
    input = input.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, function (match, alt) { return alt ? alt : ""; });
    var links = [];
    input = input.replace(/\[([^\]]+)\]\(([^)]+)\)/g, function (match, label, urlAndTitle) {
      var urlPart = (urlAndTitle || "").trim().split(/\s+/)[0] || "";
      var safeUrl = root.sanitizeMarkdownUrl(urlPart);
      var safeLabel = root.escapeHtml(label);
      if (!safeUrl) return safeLabel;
      var token = "__MDLINK_" + links.length + "__";
      links.push({ "label": safeLabel, "url": safeUrl });
      return token;
    });
    input = root.escapeHtml(input);
    for (var j = 0; j < links.length; j++) {
      var t = "__MDLINK_" + j + "__";
      var l = links[j];
      input = input.split(t).join("[" + l.label + "](" + l.url + ")");
    }
    return input;
  }

  function sanitizeMarkdownUrl(url) {
    if (!url) return "";
    var trimmed = url.trim();
    if (trimmed === "") return "";
    var lower = trimmed.toLowerCase();
    if (lower.startsWith("http://") || lower.startsWith("https://") || lower.startsWith("mailto:")) {
      return encodeURI(trimmed);
    }
    return "";
  }

  function escapeHtml(text) {
    if (!text) return "";
    return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  // ── App name ───────────────────────────────────────────────────────────

  function getAppName(name) {
    if (!name || name.trim() === "") return "Unknown";
    name = name.trim();
    if (name.includes(".") && (name.startsWith("com.") || name.startsWith("org.") || name.startsWith("io.") || name.startsWith("net."))) {
      var parts = name.split(".");
      var appPart = parts[parts.length - 1];
      if (!appPart || appPart === "app" || appPart === "desktop") {
        appPart = parts[parts.length - 2] || parts[0];
      }
      if (appPart) name = appPart;
    }
    if (name.includes(".")) {
      var parts2 = name.split(".");
      var displayName = parts2[parts2.length - 1];
      if (!displayName || /^\d+$/.test(displayName)) {
        displayName = parts2[parts2.length - 2] || parts2[0];
      }
      if (displayName) {
        displayName = displayName.charAt(0).toUpperCase() + displayName.slice(1);
        displayName = displayName.replace(/([a-z])([A-Z])/g, '$1 $2');
        displayName = displayName.replace(/app$/i, '').trim();
        displayName = displayName.replace(/desktop$/i, '').trim();
        displayName = displayName.replace(/flatpak$/i, '').trim();
        if (!displayName) {
          displayName = parts2[parts2.length - 1].charAt(0).toUpperCase() + parts2[parts2.length - 1].slice(1);
        }
      }
      return displayName || name;
    }
    var dn = name.charAt(0).toUpperCase() + name.slice(1);
    dn = dn.replace(/([a-z])([A-Z])/g, '$1 $2');
    dn = dn.replace(/app$/i, '').trim();
    dn = dn.replace(/desktop$/i, '').trim();
    return dn || name;
  }

  function getIcon(icon) {
    if (!icon) return "";
    if (icon.startsWith("/") || icon.startsWith("file://")) return icon;
    return ThemeIcons.iconFromName(icon);
  }

  // ── Signals ────────────────────────────────────────────────────────────

  signal animateAndRemove(string notificationId)

  function focusSenderWindow(appName) {
    if (!appName || appName === "" || appName === "Unknown") return false;
    var normalizedName = appName.toLowerCase().replace(/\s+/g, "");
    for (var i = 0; i < CompositorService.windows.count; i++) {
      var win = CompositorService.windows.get(i);
      var winAppId = (win.appId || "").toLowerCase();
      var segments = winAppId.split(".");
      var lastSegment = segments[segments.length - 1] || "";
      if (winAppId === normalizedName || lastSegment === normalizedName || winAppId.includes(normalizedName) || normalizedName.includes(lastSegment)) {
        CompositorService.focusWindow(win);
        return true;
      }
    }
    return false;
  }

  function getHistorySnapshot() {
    var items = [];
    for (var i = 0; i < historyList.count; i++) {
      var entry = historyList.get(i);
      items.push({
        "id": entry.id,
        "summary": entry.summary,
        "body": entry.body,
        "appName": entry.appName,
        "urgency": entry.urgency,
        "timestamp": entry.timestamp instanceof Date ? entry.timestamp.getTime() : entry.timestamp,
        "originalImage": entry.originalImage,
        "cachedImage": entry.cachedImage
      });
    }
    return items;
  }

  // ── Media toast ────────────────────────────────────────────────────────

  property string previousMediaTitle: ""
  property string previousMediaArtist: ""
  property bool previousMediaIsPlaying: false
  property bool mediaToastInitialized: false

  Timer {
    id: mediaToastInitTimer
    interval: 3000
    running: true
    onTriggered: {
      root.mediaToastInitialized = true;
      root.previousMediaTitle = MediaService.trackTitle;
      root.previousMediaArtist = MediaService.trackArtist;
      root.previousMediaIsPlaying = MediaService.isPlaying;
    }
  }

  Timer {
    id: mediaToastDebounce
    interval: 250
    onTriggered: { root.checkMediaToast(); }
  }

  function checkMediaToast() {
    if (!Settings.data.notifications.enableMediaToast || !mediaToastInitialized) return;
    if (doNotDisturb) return;

    var player = (MediaService.playerIdentity || "").toLowerCase();
    var browsers = ["firefox", "chromium", "chrome", "brave", "edge", "opera", "vivaldi", "zen"];
    var isBrowser = browsers.some(function (b) { return player.includes(b); });

    if (isBrowser && mediaToastDebounce.interval < 1500) {
      mediaToastDebounce.interval = 1500;
      mediaToastDebounce.restart();
      return;
    }

    var title = MediaService.trackTitle || "";
    var artist = MediaService.trackArtist || "";
    var isPlaying = MediaService.isPlaying;
    var titleChanged = title !== previousMediaTitle && title !== "";
    var playStateChanged = isPlaying !== previousMediaIsPlaying;
    var hasMedia = title !== "" || artist !== "";

    if (isBrowser && !isPlaying && titleChanged) {
      previousMediaTitle = title;
      previousMediaArtist = artist;
      previousMediaIsPlaying = isPlaying;
      return;
    }

    if (hasMedia && (titleChanged || playStateChanged)) {
      var icon = isPlaying ? "media-play" : "media-pause";
      var message = "";
      if (artist && title) message = artist + " — " + title;
      else if (title) message = title;
      else if (artist) message = artist;
      if (message !== "") {
        ToastService.showNotice(isPlaying ? "Play" : "Pause", message, icon, 3000);
      }
    }

    previousMediaTitle = title;
    previousMediaArtist = artist;
    previousMediaIsPlaying = isPlaying;
  }

  Connections {
    target: MediaService
    function onTrackTitleChanged() { restartDebounce(); }
    function onTrackArtistChanged() { restartDebounce(); }
    function onIsPlayingChanged() { restartDebounce(); }
    function onPlayerIdentityChanged() { restartDebounce(); }
  }

  function restartDebounce() {
    var player = (MediaService.playerIdentity || "").toLowerCase();
    var browsers = ["firefox", "chromium", "chrome", "brave", "edge", "opera", "vivaldi"];
    var isBrowser = browsers.some(function (b) { return player.includes(b); });
    mediaToastDebounce.interval = isBrowser ? 1500 : 250;
    mediaToastDebounce.restart();
  }
}
