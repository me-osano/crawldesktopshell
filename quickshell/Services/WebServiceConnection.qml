import QtQuick
import Quickshell
import Quickshell.Io

Item {
  id: root

  // ── Config ──────────────────────────────────────────────────────────────
  property string socketPath: {
    const s = Quickshell.env("CRAWL_WEBSERVICE_SOCKET")
    if (s) return s
    const dir = Quickshell.env("XDG_RUNTIME_DIR")
    if (dir) return `${dir}/crawl-webservice.sock`
    return "/tmp/crawl-webservice.sock"
  }

  property bool active: true

  // ── Connection state ────────────────────────────────────────────────────
  property bool connected: false

  // ── Internal ────────────────────────────────────────────────────────────
  property var _pending: ({})
  property var _nextId: 1
  property var _eventListeners: ({})

  // ── Request socket ──────────────────────────────────────────────────────
  CrawlSocket {
    id: reqSock
    path: root.socketPath
    connected: root.active
    parser: SplitParser {
      onRead: msg => root._onResponse(msg)
    }
    onConnectionStateChanged: {
      root.connected = reqSock.connected || subSock.connected
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
      id: 0,
    })
  }

  // ── Handle subscribe socket data ────────────────────────────────────────
  function _onSubData(data) {
    try {
      const msg = JSON.parse(data)
      if (msg.method === "event" && msg.params) {
        root._dispatchEvent(msg.params)
      }
    } catch (e) {
      console.warn("WebService: sub socket parse error:", e)
    }
  }

  // ── Dispatch events to registered listeners ─────────────────────────────
  function _dispatchEvent(event) {
    const domain = event.domain
    const data = event.data
    if (!domain || !data) return

    const eventType = data.event || data.type

    // Call listeners registered for "domain.eventType" or "domain.*"
    const key = `${domain}.${eventType}`
    const wildKey = `${domain}.*`

    var listeners = root._eventListeners[key] || []
    for (var i = 0; i < listeners.length; i++) {
      listeners[i](data.data || data)
    }

    listeners = root._eventListeners[wildKey] || []
    for (var i = 0; i < listeners.length; i++) {
      listeners[i](data.data || data)
    }
  }

  // ── Send JSON-RPC request ────────────────────────────────────────────────
  function sendRequest(method, params, callback) {
    if (!root.connected) {
      if (callback) callback(null, { error: { message: "WebService not connected" } })
      return
    }
    const id = root._nextId++
    if (callback) root._pending[id] = callback
    reqSock.send({
      jsonrpc: "2.0",
      method: method,
      params: params || {},
      id: id,
    })
  }

  // ── Handle request socket response ───────────────────────────────────────
  function _onResponse(data) {
    try {
      const msg = JSON.parse(data)
      if (msg.id != null && root._pending[msg.id]) {
        const cb = root._pending[msg.id]
        delete root._pending[msg.id]
        if (msg.error) {
          cb(null, msg)
        } else {
          cb(msg.result, null)
        }
      }
    } catch (e) {
      console.warn("WebService: response parse error:", e)
    }
  }

  // ── Register event listener ──────────────────────────────────────────────
  function onEvent(eventPattern, callback) {
    if (!root._eventListeners[eventPattern]) {
      root._eventListeners[eventPattern] = []
    }
    root._eventListeners[eventPattern].push(callback)
  }

  // ── Remove event listener ────────────────────────────────────────────────
  function offEvent(eventPattern, callback) {
    const listeners = root._eventListeners[eventPattern]
    if (!listeners) return
    const idx = listeners.indexOf(callback)
    if (idx >= 0) listeners.splice(idx, 1)
  }

  // ── Reset state on disconnect ────────────────────────────────────────────
  function _resetState() {
    root._pending = {}
  }
}
