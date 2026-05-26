pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import QtQml

Singleton {
  id: root

  // ── Adapter state ─────────────────────────────────────────────────────────
  property bool powered: false
  property bool discovering: false
  property bool discoverable: false
  property bool pairable: false

  // ── Devices ───────────────────────────────────────────────────────────────
  property var devices: []
  property var devicesByAddress: ({})

  // ── Scan state ────────────────────────────────────────────────────────────
  property bool scanning: false
  property int scanElapsed: 0
  property Timer scanTimer: Timer {
    interval: 1000
    repeat: true
    running: root.scanning
    onTriggered: root.scanElapsed++
  }

  // ── Derived convenience ───────────────────────────────────────────────────
  property var connectedDevices: root.devices.filter(function(d) { return d.connected })
  property var pairedDevices: root.devices.filter(function(d) { return d.paired })
  property var unpairedDevices: root.devices.filter(function(d) { return !d.paired })

  // ── Events ────────────────────────────────────────────────────────────────
  signal deviceDiscovered(var device)
  signal deviceConnected(var device)
  signal deviceDisconnected(var device)
  signal deviceRemoved(string address)
  signal deviceUpdated(var device)
  signal adapterStateChanged()

  // ── Subscribe to CrawlService singleton ───────────────────────────────────
  Component.onCompleted: {
    CrawlService.bluetoothChanged.connect(root._onEvent)
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
      case "adapter_powered":
        root.powered = eventData.on
        root.adapterStateChanged()
        break

      case "adapter_discoverable":
        root.discoverable = eventData.on
        root.adapterStateChanged()
        break

      case "adapter_pairable":
        root.pairable = eventData.on
        root.adapterStateChanged()
        break

      case "scan_started":
        root.scanning = true
        root.scanElapsed = 0
        root.discovering = true
        root.adapterStateChanged()
        break

      case "scan_stopped":
        root.scanning = false
        root.scanElapsed = 0
        root.discovering = false
        root.adapterStateChanged()
        break

      case "device_discovered":
        root._upsertDevice(eventData.device)
        root.deviceDiscovered(eventData.device)
        break

      case "device_connected":
        root._upsertDevice(eventData.device)
        root.deviceConnected(eventData.device)
        break

      case "device_disconnected":
        root._upsertDevice(eventData.device)
        root.deviceDisconnected(eventData.device)
        break

      case "device_updated":
        root._upsertDevice(eventData.device)
        root.deviceUpdated(eventData.device)
        break

      case "device_removed":
        root._removeDevice(eventData.address)
        root.deviceRemoved(eventData.address)
        break
    }
  }

  // ── Device list management ────────────────────────────────────────────────
  function _upsertDevice(device) {
    if (!device || !device.address) return

    devicesByAddress[device.address] = device

    var found = false
    var updated = root.devices.map(function(d) {
      if (d.address === device.address) {
        found = true
        return device
      }
      return d
    })

    if (!found) updated.push(device)
    root.devices = updated
  }

  function _removeDevice(address) {
    if (!address) return
    delete devicesByAddress[address]
    root.devices = root.devices.filter(function(d) { return d.address !== address })
  }

  // ── Fetch initial status from daemon ──────────────────────────────────────
  function _fetchStatus() {
    if (!CrawlService.connected) return
    CrawlService.getBluetoothStatus(function(resp) {
      if (resp && !resp.error) {
        root.powered = resp.powered || false
        root.discovering = resp.discovering || false
        root.devices = resp.devices || []
        root.devicesByAddress = {}
        var list = resp.devices || []
        for (var i = 0; i < list.length; i++) {
          if (list[i] && list[i].address) {
            root.devicesByAddress[list[i].address] = list[i]
          }
        }
        root.adapterStateChanged()
      }
    })
  }

  // ── Public API: Adapter control ───────────────────────────────────────────
  function togglePower() {
    root.setPowered(!root.powered)
  }

  function setPowered(on, callback) {
    CrawlService.sendRequest("BluetoothPower", { enabled: on }, callback || function() {})
  }

  function setDiscoverable(on, callback) {
    CrawlService.sendRequest("BluetoothDiscoverable", { enabled: on }, callback || function() {})
  }

  function setPairable(on, callback) {
    CrawlService.sendRequest("BluetoothPairable", { enabled: on }, callback || function() {})
  }

  // ── Public API: Scanning ──────────────────────────────────────────────────
  function startScan(timeout, callback) {
    var params = {}
    if (timeout !== undefined) params.timeout = timeout
    CrawlService.sendRequest("BluetoothScan", params, callback || function() {})
  }

  function stopScan(callback) {
    CrawlService.sendRequest("BluetoothScanStop", {}, callback || function() {})
  }

  // ── Public API: Device operations ─────────────────────────────────────────
  function connectToDevice(address, callback) {
    CrawlService.sendRequest("BluetoothConnect", { address: address }, callback || function() {})
  }

  function disconnectDevice(address, callback) {
    CrawlService.sendRequest("BluetoothDisconnect", { address: address }, callback || function() {})
  }

  function pairDevice(address, callback) {
    CrawlService.sendRequest("BluetoothPair", { address: address }, callback || function() {})
  }

  function removeDevice(address, callback) {
    CrawlService.sendRequest("BluetoothRemove", { address: address }, callback || function() {})
  }

  function setTrusted(address, trusted, callback) {
    CrawlService.sendRequest("BluetoothTrust", { address: address, trusted: trusted }, callback || function() {})
  }

  function setAlias(address, alias, callback) {
    CrawlService.sendRequest("BluetoothAlias", { address: address, alias: alias }, callback || function() {})
  }

  // ── Refresh ───────────────────────────────────────────────────────────────
  function refresh() {
    root._fetchStatus()
  }
}
