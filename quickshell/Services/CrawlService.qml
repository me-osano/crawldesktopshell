pragma Singleton
import Quickshell
import Quickshell.Io
import QtQml

Singleton {
  id: root

  // ── Config ──────────────────────────────────────────────────────────────
  property string socketPath: {
    const s = Quickshell.env("CRAWL_SOCKET")
    if (s) return s
    const dir = Quickshell.env("XDG_RUNTIME_DIR")
    if (dir) return `${dir}/crawl.sock`
    return "/tmp/crawl.sock"
  }

  property bool active: true

  // ── Connection state ────────────────────────────────────────────────────
  property bool connected: false
  property string daemonVersion: ""

  // ── Internal ────────────────────────────────────────────────────────────
  property var _pending: ({})
  property var _nextId: 1

  // ── Request socket (individual commands) ────────────────────────────────
  CrawlSocket {
    id: reqSock
    path: root.socketPath
    connected: root.active
    parser: SplitParser {
      onRead: msg => root._onResponse(msg)
    }
    onConnectionStateChanged: {
      root.connected = reqSock.connected || subSock.connected
      if (!reqSock.connected && !subSock.connected) root._resetState()
    }
  }

  // ── Subscribe socket (long-lived event stream) ─────────────────────────
  CrawlSocket {
    id: subSock
    path: root.socketPath
    connected: root.active
    parser: SplitParser {
      onRead: msg => root._onSubData(msg)
    }
    onConnectionStateChanged: {
      root.connected = reqSock.connected || subSock.connected
      if (subSock.connected) root._sendSubscribe()
    }
  }

  // ── Subscribe on connect ───────────────────────────────────────────────
  function _sendSubscribe() {
    subSock.send({
      jsonrpc: "2.0",
      method: "Subscribe",
      params: { topics: [] },
      id: 0
    })
  }

  // ── Handle subscribe socket data (response + events) ───────────────────
  function _onSubData(data) {
    try {
      const msg = JSON.parse(data)
      if (msg.id != null) {
        // Subscribe confirmation
        if (msg.result) root._bootstrap()
      } else if (msg.method === "event" && msg.params) {
        root._dispatchEvent(msg.params)
      }
    } catch (e) {
      console.warn("CrawlService: sub socket parse error:", e)
    }
  }

  // ── Bootstrap after subscribe confirmed ─────────────────────────────────
  function _bootstrap() {
    root.daemonVersion = "connected"
    root.getTheme()
    root.getStatus()
  }

  // ── Send JSON-RPC request (on request socket) ──────────────────────────
  function sendRequest(method, params, callback) {
    if (!root.connected) {
      if (callback) callback({ error: "not connected" })
      return
    }
    const id = root._nextId++
    if (callback) root._pending[id] = callback
    reqSock.send({
      jsonrpc: "2.0",
      method: method,
      params: params || {},
      id: id
    })
  }

  // ── Handle request responses ───────────────────────────────────────────
  function _onResponse(data) {
    try {
      const msg = JSON.parse(data)
      if (msg.id != null) {
        const cb = root._pending[msg.id]
        if (cb) {
          delete root._pending[msg.id]
          cb(msg.error ? { error: msg.error.message } : msg.result)
        }
      }
    } catch (e) {
      console.warn("CrawlService: response parse error:", e)
    }
  }

  // ── State reset ────────────────────────────────────────────────────────
  function _resetState() {
    root._pending = ({})
    root.daemonVersion = ""
  }

  // ── Event dispatch ─────────────────────────────────────────────────────
  function _dispatchEvent(params) {
    const domain = params.domain
    const data = params.data
    switch (domain) {
      case "theme":      root.themeChanged(data); break
      case "audio":      root.audioChanged(data); break
      case "bluetooth":  root.bluetoothChanged(data); break
      case "brightness": root.brightnessChanged(data); break
      case "idle":       root.idleChanged(data); break
      case "network":    root.networkChanged(data); break
      case "daemon":     root.daemonEvent(data); break
      case "sysmon":     root.sysmonChanged(data); break
      case "proc":       root.procEvent(data); break
      case "sysinfo":    root.sysinfoChanged(data); break
      case "wallpaper":  root.wallpaperChanged(data); break
      case "notification": root.notificationChanged(data); break
      case "rss":         root.rssChanged(data); break
      case "wallhaven":   root.wallhavenChanged(data); break
      case "mail":        root.mailChanged(data); break
      case "clipboard":   root.clipboardChanged(data); break
      default:            root.unknownEvent(domain, data); break
    }
  }

  // ── Event signals ──────────────────────────────────────────────────────
  signal themeChanged(var data)
  signal audioChanged(var data)
  signal bluetoothChanged(var data)
  signal brightnessChanged(var data)
  signal idleChanged(var data)
  signal networkChanged(var data)
  signal daemonEvent(var data)
  signal sysmonChanged(var data)
  signal procEvent(var data)
  signal sysinfoChanged(var data)
  signal wallpaperChanged(var data)
  signal notificationChanged(var data)
  signal rssChanged(var data)
  signal wallhavenChanged(var data)
  signal mailChanged(var data)
  signal clipboardChanged(var data)
  signal unknownEvent(string domain, var data)

  // ── Convenience wrappers ───────────────────────────────────────────────
  function getTheme(callback) {
    root.sendRequest("ThemeGet", {}, callback || (() => {}))
  }

  function setTheme(name, callback) {
    root.sendRequest("ThemeSet", { name: name }, callback || (() => {}))
  }

  function generateTheme(color, scheme, callback) {
    const params = { color: color }
    if (scheme) params.scheme = scheme
    root.sendRequest("ThemeGenerate", params, callback || (() => {}))
  }

  function listThemes(callback) {
    root.sendRequest("ThemeList", {}, callback || (() => {}))
  }

  function getStatus(callback) {
    root.sendRequest("Status", {}, callback || (() => {}))
  }

  function getHealth(callback) {
    root.sendRequest("Health", {}, callback || (() => {}))
  }

  function getSysinfo(callback) {
    root.sendRequest("Sysinfo", {}, callback || (() => {}))
  }

  function getAudioSinks(callback) {
    root.sendRequest("AudioSinks", {}, callback || (() => {}))
  }

  function setAudioVolume(percent, callback) {
    root.sendRequest("AudioVolume", { percent: percent }, callback || (() => {}))
  }

  function getAudioSources(callback) {
    root.sendRequest("AudioSources", {}, callback || (() => {}))
  }

  function setAudioInputVolume(percent, callback) {
    root.sendRequest("AudioInputVolume", { percent: percent }, callback || (() => {}))
  }

  function setAudioMuted(muted, callback) {
    const method = muted ? "AudioMute" : "AudioUnmute"
    root.sendRequest(method, {}, callback || (() => {}))
  }

  function setAudioInputMuted(muted, callback) {
    const method = muted ? "AudioMuteInput" : "AudioUnmuteInput"
    root.sendRequest(method, {}, callback || (() => {}))
  }

  function setDefaultSink(name, callback) {
    root.sendRequest("AudioSetDefaultSink", { name: name }, callback || (() => {}))
  }

  function setDefaultSource(name, callback) {
    root.sendRequest("AudioSetDefaultSource", { name: name }, callback || (() => {}))
  }

  function getBrightness(callback) {
    root.sendRequest("BrightnessGet", {}, callback || (() => {}))
  }

  function setBrightness(value, callback) {
    root.sendRequest("BrightnessSet", { value: value }, callback || (() => {}))
  }

  function incBrightness(value, callback) {
    root.sendRequest("BrightnessInc", { value: value || 5 }, callback || (() => {}))
  }

  function decBrightness(value, callback) {
    root.sendRequest("BrightnessDec", { value: value || 5 }, callback || (() => {}))
  }

  function getNetworkStatus(callback) {
    root.sendRequest("NetworkStatus", {}, callback || (() => {}))
  }

  function setNetworkEnabled(enabled, callback) {
    root.sendRequest("NetworkEnable", { enabled: enabled }, callback || (() => {}))
  }

  function getWifiList(callback) {
    root.sendRequest("WifiList", {}, callback || (() => {}))
  }

  function scanWifi(callback) {
    root.sendRequest("WifiScan", {}, callback || (() => {}))
  }

  function getWifiDetails(callback) {
    root.sendRequest("WifiDetails", {}, callback || (() => {}))
  }

  function connectWifi(params, callback) {
    root.sendRequest("WifiConnect", params, callback || (() => {}))
  }

  function disconnectWifi(callback) {
    root.sendRequest("WifiDisconnect", {}, callback || (() => {}))
  }

  function forgetWifi(ssid, callback) {
    root.sendRequest("WifiForget", { ssid: ssid }, callback || (() => {}))
  }

  function getEthernetList(callback) {
    root.sendRequest("EthernetList", {}, callback || (() => {}))
  }

  function getEthernetDetails(iface, callback) {
    root.sendRequest("EthernetDetails", { iface: iface || null }, callback || (() => {}))
  }

  function connectEthernet(iface, callback) {
    root.sendRequest("EthernetConnect", { iface: iface || null }, callback || (() => {}))
  }

  function disconnectEthernet(iface, callback) {
    root.sendRequest("EthernetDisconnect", { iface: iface || null }, callback || (() => {}))
  }

  function getHotspotStatus(callback) {
    root.sendRequest("HotspotStatus", {}, callback || (() => {}))
  }

  function startHotspot(config, callback) {
    root.sendRequest("HotspotStart", { config: config }, callback || (() => {}))
  }

  function stopHotspot(callback) {
    root.sendRequest("HotspotStop", {}, callback || (() => {}))
  }

  function getBluetoothStatus(callback) {
    root.sendRequest("BluetoothStatus", {}, callback || (() => {}))
  }

  function getProcessList(callback) {
    root.sendRequest("ProcList", {}, callback || (() => {}))
  }

  function idleInhibit(callback) {
    root.sendRequest("IdleInhibit", {}, callback || (() => {}))
  }

  function idleUninhibit(callback) {
    root.sendRequest("IdleUninhibit", {}, callback || (() => {}))
  }

  function idleStatus(callback) {
    root.sendRequest("IdleStatus", {}, callback || (() => {}))
  }

  function idleInhibitWithTimeout(seconds, callback) {
    root.sendRequest("IdleInhibitWithTimeout", { seconds: seconds }, callback || (() => {}))
  }

  // ── Notifications ───────────────────────────────────────────────────────
  function notificationGetState(callback) {
    root.sendRequest("NotificationGetState", {}, callback || (() => {}))
  }

  function notificationDismiss(id, callback) {
    root.sendRequest("NotificationDismiss", { id: id }, callback || (() => {}))
  }

  function notificationDismissAll(callback) {
    root.sendRequest("NotificationDismissAll", {}, callback || (() => {}))
  }

  function notificationRemoveHistory(id, callback) {
    root.sendRequest("NotificationRemoveHistory", { id: id }, callback || (() => {}))
  }

  function notificationClearHistory(callback) {
    root.sendRequest("NotificationClearHistory", {}, callback || (() => {}))
  }

  function notificationInvokeAction(id, actionId, callback) {
    root.sendRequest("NotificationInvokeAction", { id: id, action_id: actionId }, callback || (() => {}))
  }

  function notificationSetDnd(enabled, callback) {
    root.sendRequest("NotificationSetDnd", { enabled: enabled }, callback || (() => {}))
  }

  function notificationSetLastSeen(ts, callback) {
    root.sendRequest("NotificationSetLastSeen", { ts: ts }, callback || (() => {}))
  }

  function notificationGetPolicy(callback) {
    root.sendRequest("NotificationGetPolicy", {}, callback || (() => {}))
  }

  function notificationSetPolicy(policy, callback) {
    root.sendRequest("NotificationSetPolicy", { policy: policy }, callback || (() => {}))
  }

  function notificationGetRules(callback) {
    root.sendRequest("NotificationGetRules", {}, callback || (() => {}))
  }

  function notificationSetRules(rulesJson, callback) {
    root.sendRequest("NotificationSetRules", { rules_json: rulesJson }, callback || (() => {}))
  }

  // ── Clipboard ───────────────────────────────────────────────────────────
  function clipboardList(callback) {
    root.sendRequest("ClipboardList", {}, callback || (() => {}))
  }

  function clipboardGetContent(id, callback) {
    root.sendRequest("ClipboardGetContent", { id: id }, callback || (() => {}))
  }

  function clipboardCopy(id, callback) {
    root.sendRequest("ClipboardCopy", { id: id }, callback || (() => {}))
  }

  function clipboardDelete(id, callback) {
    root.sendRequest("ClipboardDelete", { id: id }, callback || (() => {}))
  }

  function clipboardWipe(callback) {
    root.sendRequest("ClipboardWipe", {}, callback || (() => {}))
  }

  function clipboardPin(id, callback) {
    root.sendRequest("ClipboardPin", { id: id }, callback || (() => {}))
  }

  function clipboardPasteText(text, callback) {
    root.sendRequest("ClipboardPasteText", { text: text }, callback || (() => {}))
  }

  function clipboardSet(text, mime, callback) {
    root.sendRequest("ClipboardSet", { text: text, mime: mime || "text/plain" }, callback || (() => {}))
  }

}
