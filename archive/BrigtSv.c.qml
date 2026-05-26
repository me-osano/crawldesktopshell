pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import QtQml
import qs.Common

Singleton {
  id: root

  // ── Brightness state ───────────────────────────────────────────────────────
  property string device: ""
  property real percent: 0
  property real current: 0
  property real max: 0
  property bool available: false

  // ── Operation state ─────────────────────────────────────────────────────────
  property bool setting: false
  property string lastError: ""

  // ── Settings ───────────────────────────────────────────────────────────────
  readonly property int step: Settings.data.brightness.brightnessStep || 5
  readonly property bool enforceMinimum: Settings.data.brightness.enforceMinimum !== false

  // ── Derived ─────────────────────────────────────────────────────────────────
  readonly property int percentRounded: Math.round(root.percent)
  readonly property bool isZero: root.percentRounded <= 0
  readonly property bool isMax: root.percentRounded >= 100

  // ── Events ─────────────────────────────────────────────────────────────────
  signal changed(var status)

  // ── Subscribe to CrawlService singleton ────────────────────────────────────
  Component.onCompleted: {
    CrawlService.brightnessChanged.connect(root._onEvent)
    CrawlService.connectedChanged.connect(root._onServiceConnected)
    if (CrawlService.connected) root._fetchStatus()
  }

  function _onServiceConnected() {
    if (CrawlService.connected) root._fetchStatus()
  }

  // ── Event handler ─────────────────────────────────────────────────────────
  function _onEvent(eventData) {
    if (!eventData || !eventData.event) return

    var ev = eventData.event

    switch (ev) {
      case "changed":
        root._applyStatus(eventData.status)
        root.changed(eventData.status)
        break
    }
  }

  function _applyStatus(status) {
    if (!status) return
    root.percent = status.percent || 0
    root.current = status.current || 0
    root.max = status.max || 0
    root.device = status.device || ""
    root.available = true
  }

  // ── Fetch initial status from daemon ──────────────────────────────────────
  function _fetchStatus() {
    if (!CrawlService.connected) return
    CrawlService.getBrightness(function(resp) {
      if (resp && !resp.error) {
        root._applyStatus(resp)
      } else {
        root.available = false
      }
    })
  }

  // ── Public API ────────────────────────────────────────────────────────────
  function setBrightness(value, callback) {
    if (value === undefined || value === null) return
    root.setting = true
    root.lastError = ""
    CrawlService.setBrightness(value, function(resp) {
      root.setting = false
      if (resp && resp.error) {
        root.lastError = resp.error.message || resp.error
      } else if (resp) {
        root._applyStatus(resp)
      }
      if (callback) callback(resp)
    })
  }

  function increment(delta, callback) {
    var d = delta !== undefined ? delta : root.step
    CrawlService.incBrightness(d, function(resp) {
      if (resp && !resp.error) {
        root._applyStatus(resp)
      }
      if (callback) callback(resp)
    })
  }

  function decrement(delta, callback) {
    var d = delta !== undefined ? delta : root.step
    CrawlService.decBrightness(d, function(resp) {
      if (resp && !resp.error) {
        root._applyStatus(resp)
      }
      if (callback) callback(resp)
    })
  }

  function up(callback) {
    root.increment(root.step, callback)
  }

  function down(callback) {
    root.decrement(root.step, callback)
  }

  function refresh(callback) {
    root._fetchStatus()
    if (callback) callback()
  }
}
