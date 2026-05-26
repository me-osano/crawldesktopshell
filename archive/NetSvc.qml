pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import QtQml
import qs.Common

Singleton {
  id: root

  // ── Network state ───────────────────────────────────────────────────────────
  property bool wifiEnabled: false
  property bool networkEnabled: false
  property string connectivity: "unknown"
  property string mode: "unknown"

  // ── WiFi ────────────────────────────────────────────────────────────────────
  property var networks: []
  property var activeWifi: null

  // ── WiFi scan ───────────────────────────────────────────────────────────────
  property bool scanning: false
  property bool scanPending: false
  property int scanElapsed: 0
  property Timer scanTimer: Timer {
    interval: 1000
    repeat: true
    running: root.scanning
    onTriggered: root.scanElapsed++
  }

  // ── Ethernet ────────────────────────────────────────────────────────────────
  property var ethernetInterfaces: []
  property var activeEthernet: null

  // ── Hotspot ─────────────────────────────────────────────────────────────────
  property var hotspot: null

  // ── Derived convenience ─────────────────────────────────────────────────────
  readonly property bool isConnected: root.connectivity === "full"
  readonly property bool isLimited: root.connectivity === "limited"
  readonly property bool isPortal: root.connectivity === "portal"
  readonly property bool isOffline: root.connectivity === "none"

  readonly property string connectedSsid: {
    if (root.activeWifi && root.activeWifi.ssid) return root.activeWifi.ssid
    return ""
  }

  readonly property string activeInterface: {
    if (root.activeWifi && root.activeWifi.interface) return root.activeWifi.interface
    return ""
  }

  // ── Events ──────────────────────────────────────────────────────────────────
  signal wifiScanStarted()
  signal wifiScanFinished()
  signal wifiListUpdated(var networks)
  signal wifiDetailsChanged(var details)
  signal connected(string ssid, string iface)
  signal disconnected(string iface)
  signal wifiEnabledChanged(bool enabled)
  signal modeChanged(string mode)
  signal connectivityChanged(string state)
  signal hotspotStarted(var status)
  signal hotspotStopped()
  signal hotspotStatusChanged(var status)
  signal hotspotClientJoined(var client)
  signal hotspotClientLeft(string mac)
  signal ethernetListUpdated(var interfaces)
  signal ethernetDetailsChanged(var details)

  // ── Subscribe to CrawlService singleton ────────────────────────────────────
  Component.onCompleted: {
    CrawlService.networkChanged.connect(root._onEvent)
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
      case "wifi_scan_started":
        root.scanning = true
        root.scanElapsed = 0
        root.wifiScanStarted()
        break

      case "wifi_scan_finished":
        root.scanning = false
        root.scanElapsed = 0
        root.wifiScanFinished()
        break

      case "wifi_list_updated":
        root.networks = eventData.networks || []
        root.wifiListUpdated(root.networks)
        break

      case "active_wifi_details_changed":
        root.activeWifi = eventData.details || null
        root.wifiDetailsChanged(root.activeWifi)
        break

      case "connected":
        root.connected(eventData.ssid, eventData.iface)
        break

      case "disconnected":
        root.disconnected(eventData.iface)
        break

      case "wifi_enabled":
        root.wifiEnabled = true
        root.wifiEnabledChanged(true)
        break

      case "wifi_disabled":
        root.wifiEnabled = false
        root.wifiEnabledChanged(false)
        break

      case "mode_changed":
        root.mode = eventData.mode || "unknown"
        root.modeChanged(root.mode)
        break

      case "connectivity_changed":
        root.connectivity = eventData.state || "unknown"
        root.connectivityChanged(root.connectivity)
        break

      case "hotspot_started":
        root.hotspot = eventData.status || null
        root.hotspotStarted(root.hotspot)
        break

      case "hotspot_stopped":
        root.hotspot = null
        root.hotspotStopped()
        break

      case "hotspot_status_changed":
        root.hotspot = eventData.status || null
        root.hotspotStatusChanged(root.hotspot)
        break

      case "hotspot_client_joined":
        root.hotspotClientJoined(eventData.client)
        break

      case "hotspot_client_left":
        root.hotspotClientLeft(eventData.mac)
        break

      case "ethernet_interfaces_changed":
        root.ethernetInterfaces = eventData.interfaces || []
        root.ethernetListUpdated(root.ethernetInterfaces)
        break

      case "active_ethernet_details_changed":
        root.activeEthernet = eventData.details || null
        root.ethernetDetailsChanged(root.activeEthernet)
        break
    }
  }

  // ── Fetch initial status from daemon ──────────────────────────────────────
  function _fetchStatus() {
    if (!CrawlService.connected) return
    CrawlService.getNetworkStatus(function(resp) {
      if (resp && !resp.error) {
        root.wifiEnabled = resp.wifi_enabled || false
        root.networkEnabled = resp.network_enabled || false
        root.connectivity = resp.connectivity || "unknown"
        root.mode = resp.mode || "unknown"
      }
    })
  }

  // ── Public API: WiFi control ──────────────────────────────────────────────
  function toggleWifi() {
    root.setWifiEnabled(!root.wifiEnabled)
  }

  function setWifiEnabled(on, callback) {
    CrawlService.sendRequest("NetworkEnable", { enabled: on }, callback || function() {})
  }

  // ── Public API: Scanning ──────────────────────────────────────────────────
  function requestScan(callback) {
    root.scanPending = true
    CrawlService.sendRequest("WifiScan", {}, callback || function() {})
  }

  // ── Public API: WiFi connections ──────────────────────────────────────────
  function connectToWifi(ssid, password, callback) {
    var params = { ssid: ssid }
    if (password !== undefined && password !== "") params.password = password
    CrawlService.sendRequest("WifiConnect", params, callback || function() {})
  }

  function disconnectWifi(callback) {
    CrawlService.sendRequest("WifiDisconnect", {}, callback || function() {})
  }

  function forgetWifi(ssid, callback) {
    CrawlService.sendRequest("WifiForget", { ssid: ssid }, callback || function() {})
  }

  function getWifiDetails(callback) {
    CrawlService.sendRequest("WifiDetails", {}, callback || function() {})
  }

  // ── Public API: Hotspot ───────────────────────────────────────────────────
  function startHotspot(config, callback) {
    CrawlService.sendRequest("HotspotStart", { config: config }, callback || function() {})
  }

  function stopHotspot(callback) {
    CrawlService.sendRequest("HotspotStop", {}, callback || function() {})
  }

  function getHotspotStatus(callback) {
    CrawlService.sendRequest("HotspotStatus", {}, callback || function() {})
  }

  // ── Public API: Ethernet ──────────────────────────────────────────────────
  function listEthernet(callback) {
    CrawlService.sendRequest("EthernetList", {}, callback || function() {})
  }

  function connectEthernet(iface, callback) {
    CrawlService.sendRequest("EthernetConnect", { iface: iface }, callback || function() {})
  }

  function disconnectEthernet(iface, callback) {
    CrawlService.sendRequest("EthernetDisconnect", { iface: iface }, callback || function() {})
  }

  function getEthernetDetails(iface, callback) {
    var params = {}
    if (iface !== undefined) params.iface = iface
    CrawlService.sendRequest("EthernetDetails", params, callback || function() {})
  }

  // ── WiFi network helpers ──────────────────────────────────────────────────
  function getNetworkIcon(network) {
    if (!network) return "wifi-off"
    if (network.connected) return "wifi-connected"
    var sig = network.signal || 0
    if (sig >= 80) return "wifi-high"
    if (sig >= 55) return "wifi-medium"
    if (sig >= 30) return "wifi-low"
    return "wifi-poor"
  }

  function getSignalPercent(network) {
    if (!network || network.signal === undefined) return null
    return Math.round(network.signal * 100)
  }

  function getSecurityLabel(network) {
    if (!network) return ""
    if (!network.secured) return "Open"
    return network.security || "Secured"
  }

  function isNetworkWpa2(network) {
    return network && network.security === "wpa2"
  }

  function isNetworkOpen(network) {
    return network && !network.secured
  }

  // ── Sorting / dedup ───────────────────────────────────────────────────────
  function sortNetworks(list) {
    if (!list || !Array.isArray(list)) return []
    return list.sort(function(a, b) {
      if (a.connected && !b.connected) return -1
      if (!a.connected && b.connected) return 1
      if (a.existing && !b.existing) return -1
      if (!a.existing && b.existing) return 1
      return (b.signal || 0) - (a.signal || 0)
    })
  }

  function dedupeNetworks(list) {
    if (!list || !Array.isArray(list)) return []
    var seen = {}
    var result = []
    for (var i = 0; i < list.length; i++) {
      var n = list[i]
      if (!n || !n.ssid) continue
      if (seen[n.ssid]) continue
      seen[n.ssid] = true
      result.push(n)
    }
    return result
  }

  // ── Refresh ───────────────────────────────────────────────────────────────
  function refresh() {
    root._fetchStatus()
  }
}
