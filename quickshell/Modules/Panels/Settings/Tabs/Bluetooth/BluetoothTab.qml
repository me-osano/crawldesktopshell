import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import Quickshell
import Quickshell.Bluetooth

import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root
  Layout.fillWidth: true
  implicitHeight: mainLayout.implicitHeight

  // Configuration for shared use (e.g. by BluetoothPanel)
  property bool showOnlyLists: false

  readonly property bool isScanningActive: BluetoothService.scanningActive
  readonly property bool isDiscoverable: BluetoothService.discoverable

  // Device lists with local filtering logic
  readonly property var connectedDevices: {
    if (!BluetoothService.adapter || !BluetoothService.adapter.devices)
      return [];
    var filtered = BluetoothService.adapter.devices.values.filter(dev => dev && !dev.blocked && dev.connected);
    filtered = BluetoothService.dedupeDevices(filtered);
    return BluetoothService.sortDevices(filtered);
  }

  readonly property var pairedDevices: {
    if (!BluetoothService.adapter || !BluetoothService.adapter.devices)
      return [];
    var filtered = BluetoothService.adapter.devices.values.filter(dev => dev && !dev.blocked && !dev.connected && (dev.paired || dev.trusted));
    filtered = BluetoothService.dedupeDevices(filtered);
    return BluetoothService.sortDevices(filtered);
  }

  readonly property var unnamedAvailableDevices: {
    if (!BluetoothService.adapter || !BluetoothService.adapter.devices)
      return [];
    return BluetoothService.adapter.devices.values.filter(dev => dev && !dev.blocked && !dev.paired && !dev.trusted);
  }

  readonly property var availableDevices: {
    var list = root.unnamedAvailableDevices;

    if (Settings.data.network.bluetoothHideUnnamedDevices) {
      list = list.filter(function (dev) {
        var dn = dev.name || dev.deviceName || "";
        var s = String(dn).trim();
        if (s.length === 0)
          return false;
        var lower = s.toLowerCase();
        if (lower === "unknown" || lower === "unnamed" || lower === "n/a" || lower === "na")
          return false;
        var addr = dev.address || dev.bdaddr || dev.mac || "";
        if (addr.length > 0) {
          var normName = s.toLowerCase().replace(/[^0-9a-z]/g, "");
          var normAddr = String(addr).toLowerCase().replace(/[^0-9a-z]/g, "");
          if (normName.length > 0 && normName === normAddr)
            return false;
        }
        var macRegexComb = /^(([0-9A-Fa-f]{2}[:\-]){5}[0-9A-Fa-f]{2}|([0-9A-Fa-f]{4}\.){2}[0-9A-Fa-f]{4}|[0-9A-Fa-f]{12})$/;
        if (macRegexComb.test(s)) {
          return false;
        }
        return true;
      });
    }
    list = BluetoothService.dedupeDevices(list);
    return BluetoothService.sortDevices(list);
  }

  // For managing expanded device details
  property string expandedDeviceKey: ""
  property bool detailsGrid: (Settings.data.network.bluetoothDetailsViewMode === "grid")

  // Combined visibility check: tab must be visible AND the window must be visible
  readonly property bool effectivelyVisible: root.visible && Window.window && Window.window.visible

  Connections {
    target: BluetoothService
    function onEnabledChanged() {
      stateChangeDebouncer.restart();
    }
    function onDiscoverableChanged() {
      stateChangeDebouncer.restart();
    }
  }

  onEffectivelyVisibleChanged: stateChangeDebouncer.restart()

  Timer {
    id: stateChangeDebouncer
    interval: 100 // 100ms debounce
    repeat: false
    onTriggered: root._updateScanningState()
  }

  function _updateScanningState() {
    if (effectivelyVisible && BluetoothService.enabled && !showOnlyLists) {
      Logger.d("BluetoothPrefs", "Panel/tab active");
      if (!isScanningActive) {
        BluetoothService.setScanActive(true);
      }
      if (!Settings.data.network.disableDiscoverability && !isDiscoverable) {
        BluetoothService.setDiscoverable(true);
      }
    } else {
      Logger.d("BluetoothPrefs", "Panel/tab inactive");
      if (isScanningActive && !showOnlyLists) {
        BluetoothService.setScanActive(false);
      }
      if (isDiscoverable && !showOnlyLists) {
        BluetoothService.setDiscoverable(false);
      }
    }
  }

  Component.onDestruction: {
    // Ensure scanning is stopped when component is closed
    if (isScanningActive && !showOnlyLists) {
      BluetoothService.setScanActive(false);
    }
    // Ensure discoverable is disabled when component is closed
    if (isDiscoverable && !showOnlyLists) {
      BluetoothService.setDiscoverable(false);
    }
    Logger.d("BluetoothPrefs", "Panel closed");
  }

  ColumnLayout {
    id: mainLayout
    anchors.left: parent.left
    anchors.right: parent.right
    spacing: Style.marginL

    // Master Control Section
    CrawlBox {
      visible: !root.showOnlyLists
      Layout.fillWidth: true
      Layout.preferredHeight: masterControlCol.implicitHeight + Style.margin2L
      color: Theme.cSurface

      ColumnLayout {
        id: masterControlCol
        anchors.fill: parent
        anchors.margins: Style.marginL
        spacing: Style.marginM

        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginM

          CrawlToggle {
            label: "Bluetooth"
            icon: BluetoothService.enabled ? "bluetooth" : "bluetooth-off"
            checked: BluetoothService.enabled
            enabled: !Settings.data.network.airplaneModeEnabled && BluetoothService.bluetoothAvailable
            onToggled: checked => BluetoothService.setBluetoothEnabled(checked)
            Layout.alignment: Qt.AlignVCenter
          }
        }

        CrawlDivider {
          Layout.fillWidth: true
          visible: BluetoothService.enabled && isDiscoverable
        }

        CrawlText {
          visible: BluetoothService.enabled && isDiscoverable
          Layout.fillWidth: true
          text: "This device is discoverable as <b>" + (HostService.hostName) + "</b> while this settings tab is open."
          color: Theme.cOnSurfaceVariant
          richTextEnabled: true
          wrapMode: Text.WordWrap
          horizontalAlignment: Text.AlignHCenter
        }
      }
    }

    // Bluetooth Status Section
    CrawlBox {
      visible: !root.showOnlyLists && BluetoothService.enabled
      Layout.fillWidth: true
      Layout.preferredHeight: statusCol.implicitHeight + Style.margin2L
      color: Theme.cSurface

      ColumnLayout {
        id: statusCol
        anchors.fill: parent
        anchors.margins: Style.marginL
        spacing: Style.marginM

        CrawlHeader {
          label: "Bluetooth Status"
        }

        // Adapter info
        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginM

          CrawlIcon {
            icon: "bluetooth"
            pointSize: Style.fontSizeXL
            color: BluetoothService.enabled ? Theme.cPrimary : Theme.cOnSurfaceVariant
          }

          ColumnLayout {
            Layout.fillWidth: true
            spacing: 0

            CrawlText {
              text: BluetoothService.adapter ? (BluetoothService.adapter.name || HostService.hostName) : HostService.hostName
              font.weight: Style.fontWeightBold
              pointSize: Style.fontSizeM
            }

            CrawlText {
              text: {
                var parts = [];
                if (BluetoothService.adapter && BluetoothService.adapter.address)
                  parts.push(BluetoothService.adapter.address);
                if (BluetoothService.backendUsed)
                  parts.push("Backend: " + BluetoothService.backendUsed);
                return parts.length > 0 ? parts.join(" · ") : "Bluetooth adapter";
              }
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
            }
          }
        }

        CrawlDivider {
          Layout.fillWidth: true
        }

        // Status indicators
        GridLayout {
          Layout.fillWidth: true
          columns: 2
          columnSpacing: Style.marginL
          rowSpacing: Style.marginS

          // Connected count
          RowLayout {
            spacing: Style.marginS
            CrawlIcon {
              icon: "bluetooth-connected"
              pointSize: Style.fontSizeS
              color: root.connectedDevices.length > 0 ? Theme.cPrimary : Theme.cOnSurfaceVariant
            }
            CrawlText {
              text: root.connectedDevices.length + " connected"
              pointSize: Style.fontSizeS
              color: Theme.cOnSurface
            }
          }

          // Paired count
          RowLayout {
            spacing: Style.marginS
            CrawlIcon {
              icon: "link"
              pointSize: Style.fontSizeS
              color: Theme.cOnSurfaceVariant
            }
            CrawlText {
              text: root.pairedDevices.length + " paired"
              pointSize: Style.fontSizeS
              color: Theme.cOnSurface
            }
          }

          // Scanning status
          RowLayout {
            spacing: Style.marginS
            CrawlIcon {
              icon: root.isScanningActive ? "radar-2" : "radar-off"
              pointSize: Style.fontSizeS
              color: root.isScanningActive ? Theme.cPrimary : Theme.cOnSurfaceVariant
            }
            CrawlText {
              text: root.isScanningActive ? "Scanning" : "Not scanning"
              pointSize: Style.fontSizeS
              color: Theme.cOnSurface
            }
          }

          // Discoverable status
          RowLayout {
            spacing: Style.marginS
            CrawlIcon {
              icon: root.isDiscoverable ? "eye" : "eye-off"
              pointSize: Style.fontSizeS
              color: root.isDiscoverable ? Theme.cPrimary : Theme.cOnSurfaceVariant
            }
            CrawlText {
              text: root.isDiscoverable ? "Visible" : "Hidden"
              pointSize: Style.fontSizeS
              color: Theme.cOnSurface
            }
          }
        }
      }
    }

    // Quick Actions
    CrawlBox {
      visible: !root.showOnlyLists && BluetoothService.enabled
      Layout.fillWidth: true
      Layout.preferredHeight: actionsCol.implicitHeight + Style.margin2L
      color: Theme.cSurface

      ColumnLayout {
        id: actionsCol
        anchors.fill: parent
        anchors.margins: Style.marginL
        spacing: Style.marginM

        CrawlHeader {
          label: "Quick Actions"
        }

        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginM

          CrawlButton {
            text: root.isScanningActive ? "Scanning..." : "Scan"
            icon: "radar-2"
            enabled: BluetoothService.enabled && !root.isScanningActive
            onClicked: BluetoothService.setScanActive(true)
          }

          CrawlButton {
            text: "Open Bluetooth Panel"
            icon: "bluetooth"
            onClicked: {
              var panel = PanelService.getPanel("bluetoothPanel", null);
              if (panel) panel.toggle();
            }
          }
        }
      }
    }

    Item {
      visible: !showOnlyLists
      Layout.fillWidth: true
    }

    // Device List [1] (Connected)
    CrawlBox {
      id: connectedDevicesBox
      visible: root.connectedDevices.length > 0 && BluetoothService.enabled
      Layout.fillWidth: true
      Layout.preferredHeight: connectedDevicesCol.implicitHeight + Style.margin2M
      border.color: showOnlyLists ? Style.boxBorderColor : "transparent"

      ColumnLayout {
        id: connectedDevicesCol
        anchors.fill: parent
        anchors.topMargin: Style.marginM
        anchors.bottomMargin: Style.marginM
        anchors.leftMargin: showOnlyLists ? Style.marginL : 0
        anchors.rightMargin: showOnlyLists ? Style.marginL : 0
        spacing: Style.marginM

        CrawlLabel {
          label: "Connected devices"
          Layout.fillWidth: true
          Layout.leftMargin: Style.marginS
        }

        Repeater {
          model: root.connectedDevices
          delegate: nboxDelegate
        }
      }
    }

    // Devices List [2] (Paired)
    CrawlBox {
      id: pairedDevicesBox
      visible: root.pairedDevices.length > 0 && BluetoothService.enabled
      Layout.fillWidth: true
      Layout.preferredHeight: pairedDevicesCol.implicitHeight + Style.margin2M
      border.color: showOnlyLists ? Style.boxBorderColor : "transparent"

      ColumnLayout {
        id: pairedDevicesCol
        anchors.fill: parent
        anchors.topMargin: Style.marginM
        anchors.bottomMargin: Style.marginM
        anchors.leftMargin: showOnlyLists ? Style.marginL : 0
        anchors.rightMargin: showOnlyLists ? Style.marginL : 0
        spacing: Style.marginM

        CrawlLabel {
          label: "Paired devices"
          Layout.fillWidth: true
          Layout.leftMargin: Style.marginS
        }

        Repeater {
          model: root.pairedDevices
          delegate: nboxDelegate
        }
      }
    }

    // Device List [3] (Available)
    CrawlBox {
      id: availableDevicesBox
      visible: !root.showOnlyLists && root.unnamedAvailableDevices.length > 0 && BluetoothService.enabled
      Layout.fillWidth: true
      Layout.preferredHeight: availableDevicesCol.implicitHeight + Style.margin2M
      border.color: "transparent"

      ColumnLayout {
        id: availableDevicesCol
        anchors.fill: parent
        anchors.topMargin: Style.marginM
        anchors.bottomMargin: Style.marginM
        spacing: Style.marginM

        RowLayout {
          Layout.fillWidth: true
          Layout.leftMargin: Style.marginS
          spacing: Style.marginS

          CrawlLabel {
            label: "Available devices"
            description: BluetoothService.scanningActive ? "Scanning for devices..." : ""
            Layout.fillWidth: true
          }
        }

        Repeater {
          model: root.availableDevices
          delegate: nboxDelegate
        }

        CrawlText {
          visible: root.availableDevices.length === 0 && root.unnamedAvailableDevices.length > 0
          text: "Unnamed devices are not shown."
          pointSize: Style.fontSizeS
          color: Theme.cOnSurfaceVariant
          horizontalAlignment: Text.AlignHCenter
          Layout.fillWidth: true
          Layout.margins: Style.marginL
        }
      }
    }

    Item {
      visible: !showOnlyLists
      Layout.fillWidth: true
    }

    CrawlBox {
      id: miscSettingsBox
      visible: !root.showOnlyLists && BluetoothService.enabled
      Layout.fillWidth: true
      Layout.preferredHeight: miscSettingsCol.implicitHeight + Style.margin2XL
      color: Theme.cSurface

      ColumnLayout {
        id: miscSettingsCol
        anchors.fill: parent
        anchors.margins: Style.marginXL
        spacing: Style.marginM

        CrawlHeader {
          label: "Settings"
        }

        CrawlToggle {
          label: "Hide unnamed devices"
          description: "Hide devices that appear only as Bluetooth addresses."
          checked: Settings.data.network.bluetoothHideUnnamedDevices
          onToggled: checked => Settings.data.network.bluetoothHideUnnamedDevices = checked
        }

        CrawlToggle {
          label: "Disable device visibility"
          description: "Hide your device from nearby Bluetooth devices."
          checked: Settings.data.network.disableDiscoverability
          onToggled: checked => {
                       Settings.data.network.disableDiscoverability = checked;
                       BluetoothService.setDiscoverable(!checked);
                     }
        }

        CrawlDivider {
          Layout.fillWidth: true
        }

        CrawlHeader {
          label: "Signal Monitoring"
        }

        // RSSI Polling
        CrawlToggle {
          label: "Bluetooth signal polling"
          description: "Periodically sample RSSI for connected devices via bluetoothctl. May not be available for all devices; uses minimal resources when enabled."
          checked: Settings.data.network.bluetoothRssiPollingEnabled
          onToggled: checked => Settings.data.network.bluetoothRssiPollingEnabled = checked
        }

        CrawlSpinBox {
          label: "Polling interval"
          description: "Configure how often to update signal strength for connected devices."
          from: 10000
          to: 120000
          stepSize: 1000
          value: Settings.data.network.bluetoothRssiPollIntervalMs
          defaultValue: Settings.getDefaultValue("network.bluetoothRssiPollIntervalMs")
          onValueChanged: Settings.data.network.bluetoothRssiPollIntervalMs = value
          suffix: " ms"
          Layout.alignment: Qt.AlignVCenter
          visible: Settings.data.network.bluetoothRssiPollingEnabled
        }
      }
    }
  }

  // Shared Delegate
  Component {
    id: nboxDelegate
    CrawlBox {
      id: device

      readonly property bool canConnect: BluetoothService.canConnect(modelData)
      readonly property bool canDisconnect: BluetoothService.canDisconnect(modelData)
      readonly property bool canPair: BluetoothService.canPair(modelData)
      readonly property bool isBusy: BluetoothService.isDeviceBusy(modelData)
      readonly property bool isExpanded: root.expandedDeviceKey === BluetoothService.deviceKey(modelData)

      function getContentColor(defaultColor = Theme.cOnSurface) {
        if (modelData.pairing || modelData.state === BluetoothDeviceState.Connecting)
          return Theme.cPrimary;
        if (modelData.blocked || modelData.state === BluetoothDeviceState.Disconnecting)
          return Theme.cError;
        return defaultColor;
      }

      Layout.fillWidth: true
      Layout.preferredHeight: deviceColumn.implicitHeight + (Style.marginXL)
      radius: Style.radiusM
      clip: true

      color: (modelData.connected && modelData.state !== BluetoothDeviceState.Disconnecting) ? Qt.alpha(Theme.cPrimary, 0.15) : Theme.cSurface

      ColumnLayout {
        id: deviceColumn
        anchors.fill: parent
        anchors.margins: Style.marginM
        spacing: Style.marginS

        RowLayout {
          id: deviceLayout
          Layout.fillWidth: true
          spacing: Style.marginM
          Layout.alignment: Qt.AlignVCenter

          CrawlIcon {
            icon: BluetoothService.getDeviceIcon(modelData)
            pointSize: Style.fontSizeXXL
            color: modelData.connected ? Theme.cPrimary : device.getContentColor(Theme.cOnSurface)
            Layout.alignment: Qt.AlignVCenter
          }

          ColumnLayout {
            Layout.fillWidth: true
            spacing: Style.marginXXS

            CrawlText {
              text: modelData.name || modelData.deviceName
              pointSize: Style.fontSizeM
              font.weight: modelData.connected ? Style.fontWeightBold : Style.fontWeightMedium
              elide: Text.ElideRight
              color: device.getContentColor(Theme.cOnSurface)
              Layout.fillWidth: true
            }

            CrawlText {
              text: {
                const k = BluetoothService.getStatusKey(modelData);
                if (k === "pairing")
                  return "Pairing...";
                if (k === "blocked")
                  return "Blocked";
                if (k === "connecting")
                  return "Connecting...";
                if (k === "disconnecting")
                  return "Disconnecting...";
                return "";
              }
              visible: text !== ""
              pointSize: Style.fontSizeXS
              color: device.getContentColor(Theme.cOnSurfaceVariant)
            }

            RowLayout {
              visible: modelData.batteryAvailable
              spacing: Style.marginS
              CrawlIcon {
                icon: {
                  var b = BluetoothService.getBatteryPercent(modelData);
                  return BatteryService.getIcon(b !== null ? b : 0, false, false, b !== null);
                }
                pointSize: Style.fontSizeXS
                color: device.getContentColor(Theme.cOnSurface)
              }
              CrawlText {
                text: {
                  var b = BluetoothService.getBatteryPercent(modelData);
                  return b === null ? "-" : (b + "%");
                }
                pointSize: Style.fontSizeXS
                color: device.getContentColor(Theme.cOnSurfaceVariant)
              }
            }
          }

          Item {
            Layout.fillWidth: true
          }

          RowLayout {
            spacing: Style.marginS

            CrawlIconButton {
              visible: modelData.connected
              icon: "info"
              tooltipText: "Info"
              baseSize: Style.baseWidgetSize * 0.8
              onClicked: {
                const key = BluetoothService.deviceKey(modelData);
                root.expandedDeviceKey = (root.expandedDeviceKey === key) ? "" : key;
              }
            }

            CrawlIconButton {
              visible: !root.showOnlyLists && (modelData.paired || modelData.trusted) && !modelData.connected && !isBusy && !modelData.blocked
              icon: "trash"
              tooltipText: "Unpair"
              baseSize: Style.baseWidgetSize * 0.8
              onClicked: BluetoothService.unpairDevice(modelData)
            }

            CrawlButton {
              id: button
              visible: (modelData.state !== BluetoothDeviceState.Connecting)
              enabled: (canConnect || canDisconnect || (root.showOnlyLists ? false : canPair)) && !isBusy
              outlined: !button.hovered
              fontSize: Style.fontSizeS
              backgroundColor: modelData.connected ? Theme.cError : Theme.cPrimary
              text: {
                if (modelData.pairing)
                  return "Pairing...";
                if (modelData.blocked)
                  return "Blocked";
                if (modelData.connected)
                  return "Disconnect";
                if (!root.showOnlyLists && device.canPair)
                  return "Pair";
                return "Connect";
              }
              icon: (isBusy ? "busy" : null)
              onClicked: {
                if (modelData.connected) {
                  BluetoothService.disconnectDevice(modelData);
                } else {
                  if (!root.showOnlyLists && device.canPair) {
                    BluetoothService.pairDevice(modelData);
                  } else {
                    BluetoothService.connectDeviceWithTrust(modelData);
                  }
                }
              }
            }
          }
        }

        // Expanded info section
        Rectangle {
          visible: device.isExpanded
          Layout.fillWidth: true
          implicitHeight: infoColumn.implicitHeight + Style.margin2S
          radius: Style.radiusS
          color: Theme.cSurfaceVariant
          border.width: Style.borderS
          border.color: Theme.cOutline
          clip: true

          CrawlIconButton {
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: Style.marginS
            icon: root.detailsGrid ? "layout-list" : "layout-grid"
            tooltipText: root.detailsGrid ? "List view" : "Grid view"
            baseSize: Style.baseWidgetSize * 0.8
            onClicked: {
              root.detailsGrid = !root.detailsGrid;
              Settings.data.network.bluetoothDetailsViewMode = root.detailsGrid ? "grid" : "list";
            }
            z: 1
          }

          GridLayout {
            id: infoColumn
            anchors.fill: parent
            anchors.margins: Style.marginS
            columns: root.detailsGrid ? 2 : 1
            columnSpacing: Style.marginM
            rowSpacing: Style.marginXS

            RowLayout {
              Layout.fillWidth: true
              Layout.preferredWidth: 1
              spacing: Style.marginXS
              CrawlIcon {
                icon: BluetoothService.getSignalIcon(modelData)
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
              }
              CrawlText {
                text: BluetoothService.getSignalStrength(modelData)
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
                Layout.fillWidth: true
              }
            }
            RowLayout {
              Layout.fillWidth: true
              Layout.preferredWidth: 1
              spacing: Style.marginXS
              CrawlIcon {
                icon: {
                  var b = BluetoothService.getBatteryPercent(modelData);
                  return BatteryService.getIcon(b !== null ? b : 0, false, false, b !== null);
                }
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
              }
              CrawlText {
                text: {
                  var b = BluetoothService.getBatteryPercent(modelData);
                  return b === null ? "-" : (b + "%");
                }
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
                Layout.fillWidth: true
              }
            }
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginXS
              CrawlIcon {
                icon: "link"
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
              }
              CrawlText {
                text: modelData.paired ? "Yes" : "No"
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
                Layout.fillWidth: true
              }
            }
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginXS
              CrawlIcon {
                icon: "shield-check"
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
              }
              CrawlText {
                text: modelData.trusted ? "Yes" : "No"
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
                Layout.fillWidth: true
              }
            }
            RowLayout {
              Layout.fillWidth: true
              Layout.columnSpan: infoColumn.columns === 2 ? 2 : 1
              spacing: Style.marginXS
              CrawlIcon {
                icon: "hash"
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
              }
              CrawlText {
                text: modelData.address || "-"
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurface
                Layout.fillWidth: true
              }
            }
          }
        }
      }
    }
  }

  // PIN Authentication Overlay (This part needs some love :P)
  Rectangle {
    id: pinOverlay
    visible: !root.showOnlyLists && BluetoothService.pinRequired
    anchors.centerIn: parent
    width: Math.min(parent.width * 0.9, 400)
    height: pinCol.implicitHeight + Style.margin2L
    color: Theme.cSurface
    radius: Style.radiusM
    border.color: Style.boxBorderColor
    border.width: Style.borderS
    z: 1000

    MouseArea {
      anchors.fill: parent
      acceptedButtons: Qt.AllButtons
      onClicked: mouse => mouse.accepted = true
      onWheel: wheel => wheel.accepted = true
    }

    ColumnLayout {
      id: pinCol
      anchors.fill: parent
      anchors.margins: Style.marginL
      spacing: Style.marginL

      CrawlIcon {
        icon: "lock"
        pointSize: 48
        color: Theme.cPrimary
        Layout.alignment: Qt.AlignHCenter
      }
      CrawlText {
        text: "Authentication required"
        pointSize: Style.fontSizeXL
        font.weight: Style.fontWeightBold
        color: Theme.cOnSurface
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
      }
      CrawlText {
        text: "Please enter the PIN code displayed on your device."
        pointSize: Style.fontSizeM
        color: Theme.cOnSurfaceVariant
        wrapMode: Text.WordWrap
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
      }
      CrawlTextInput {
        id: pinInput
        Layout.fillWidth: true
        placeholderText: "123456"
        inputIconName: "key"
        onVisibleChanged: {
          if (visible) {
            text = "";
            inputItem.forceActiveFocus();
          }
        }
        inputItem.onAccepted: {
          if (text.length > 0) {
            BluetoothService.submitPin(text);
            text = "";
          }
        }
      }
      RowLayout {
        Layout.alignment: Qt.AlignHCenter
        spacing: Style.marginM
        CrawlButton {
          text: "Cancel"
          icon: "x"
          onClicked: BluetoothService.cancelPairing()
        }
        CrawlButton {
          text: "Confirm"
          icon: "check"
          backgroundColor: Theme.cPrimary
          textColor: Theme.cOnPrimary
          enabled: pinInput.text.length > 0
          onClicked: {
            BluetoothService.submitPin(pinInput.text);
            pinInput.text = "";
          }
        }
      }
    }
  }
}
