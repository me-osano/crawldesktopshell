import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Bluetooth
import qs.Common
import qs.Modules.Panels.Settings
import "../Panels/Settings/Tabs/Bluetooth" as BluetoothPrefs
import qs.Services
import qs.Widgets

// Bluetooth card: inline expandable bluetooth panel for control center
CrawlBox {
  id: root

  property ShellScreen screen
  property bool expanded: false
  
  // Expose the target height (non-animated) for parent layout calculations
  readonly property real targetHeight: expanded ? contentColumn.implicitHeight + Style.margin2M : 0

  clip: true
  visible: expanded
  
  Behavior on implicitHeight {
    NumberAnimation {
      duration: Style.animationNormal
      easing.type: Easing.InOutQuad
    }
  }

  implicitHeight: expanded ? contentColumn.implicitHeight + Style.margin2M : 0

  ColumnLayout {
    id: contentColumn
    anchors.fill: parent
    anchors.margins: Style.marginM
    spacing: Style.marginM

    // Header row with bluetooth status
    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginS

      CrawlIcon {
        id: headerIcon
        pointSize: Style.fontSizeXL
        color: BluetoothService.enabled ? Theme.cPrimary : Theme.cOnSurfaceVariant
        icon: {
          if (!BluetoothService.enabled) {
            return "bluetooth-off";
          }
          if (BluetoothService.connectedDevices && BluetoothService.connectedDevices.length > 0) {
            return "bluetooth-connected";
          }
          return "bluetooth";
        }
      }

      ColumnLayout {
        Layout.fillWidth: true
        spacing: 0

        CrawlText {
          text: {
            if (!BluetoothService.enabled) {
              return "Bluetooth Off";
            }
            if (BluetoothService.connectedDevices && BluetoothService.connectedDevices.length > 0) {
              const firstDevice = BluetoothService.connectedDevices[0];
              return firstDevice.name || firstDevice.deviceName || "Connected";
            }
            return "Bluetooth";
          }
          font.weight: Style.fontWeightBold
          pointSize: Style.fontSizeS
        }

        CrawlText {
          text: {
            if (!BluetoothService.enabled) {
              return "Enable to connect devices";
            }
            if (BluetoothService.connectedDevices && BluetoothService.connectedDevices.length > 1) {
              return (BluetoothService.connectedDevices.length - 1) + " more device(s) connected";
            }
            if (BluetoothService.connectedDevices && BluetoothService.connectedDevices.length === 1) {
              return "Connected";
            }
            return "No devices connected";
          }
          pointSize: Style.fontSizeXS
          color: Theme.cOnSurfaceVariant
        }
      }

      Item {
        Layout.fillWidth: true
      }

      // Bluetooth settings icon button
            // Settings button
      CrawlIconButton {
        icon: "settings"
        baseSize: Style.baseWidgetSize * 0.8
        tooltipText: "Bluetooth Settings"
        onClicked: SettingsPanelService.openToTab(SettingsPanel.Tab.Connections, 0, screen)
      }

      // CrawlIconButton {
      //   //baseSize: Style.fontSizeM * 2
      //   icon: "settings"
      //   colorBg: "transparent"
      //   colorFg: Theme.cOnSurfaceVariant
      //   tooltipText: "Bluetooth Settings"
      //   onClicked: SettingsPanelService.openToTab(SettingsPanel.Tab.Connections, 1, screen)
      // }

      // Bluetooth toggle
      CrawlToggle {
        id: bluetoothSwitch
        checked: BluetoothService.enabled
        enabled: !Settings.data.network.airplaneModeEnabled && BluetoothService.bluetoothAvailable
        onToggled: checked => BluetoothService.setBluetoothEnabled(checked)
        baseSize: Style.baseWidgetSize * 0.65
      }
    }

    CrawlDivider {
      Layout.fillWidth: true
    }

    // Bluetooth disabled state
    ColumnLayout {
      visible: !BluetoothService.enabled
      Layout.fillWidth: true
      Layout.alignment: Qt.AlignHCenter
      spacing: Style.marginM

      CrawlIcon {
        icon: "bluetooth-off"
        pointSize: Style.fontSizeXXL
        color: Theme.cOnSurfaceVariant
        Layout.alignment: Qt.AlignHCenter
      }

      CrawlText {
        text: "Bluetooth is disabled"
        pointSize: Style.fontSizeM
        color: Theme.cOnSurfaceVariant
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
        Layout.alignment: Qt.AlignHCenter
      }

      CrawlText {
        text: "Enable Bluetooth to see devices"
        pointSize: Style.fontSizeXS
        color: Theme.cOnSurfaceVariant
        horizontalAlignment: Text.AlignHCenter
        Layout.fillWidth: true
        Layout.alignment: Qt.AlignHCenter
      }
    }

    // Devices list when enabled
    CrawlScrollView {
      visible: BluetoothService.enabled
      Layout.fillWidth: true
      Layout.preferredHeight: Math.min(devicesColumn.implicitHeight, Math.round(250 * Style.uiScaleRatio))
      horizontalPolicy: ScrollBar.AlwaysOff
      verticalPolicy: ScrollBar.AsNeeded
      reserveScrollbarSpace: false

      ColumnLayout {
        id: devicesColumn
        width: parent.width
        spacing: Style.marginS

        // Connected devices section
        CrawlText {
          visible: BluetoothService.connectedDevices && BluetoothService.connectedDevices.length > 0
          text: "Connected"
          font.weight: Style.fontWeightBold
          pointSize: Style.fontSizeXS
          color: Theme.cOnSurfaceVariant
          Layout.fillWidth: true
        }

        Repeater {
          model: BluetoothService.connectedDevices || []

          delegate: CrawlBox {
            required property var modelData
            
            Layout.fillWidth: true
            implicitHeight: deviceRow.implicitHeight + Style.marginM
            color: Qt.alpha(Theme.cPrimary, 0.08)

            RowLayout {
              id: deviceRow
              anchors.fill: parent
              anchors.margins: Style.marginS
              spacing: Style.marginS

              CrawlIcon {
                icon: "bluetooth-connected"
                pointSize: Style.fontSizeL
                color: Theme.cPrimary
              }

              ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                CrawlText {
                  text: modelData.name || modelData.deviceName || "Unknown Device"
                  font.weight: Style.fontWeightSemiBold
                  pointSize: Style.fontSizeS
                  elide: Text.ElideRight
                  Layout.fillWidth: true
                }

                CrawlText {
                  text: "Connected"
                  pointSize: Style.fontSizeXXS
                  color: Theme.cPrimary
                }
              }

              CrawlIconButton {
                icon: "bluetooth-off"
                baseSize: Style.baseWidgetSize * 0.7
                tooltipText: "Disconnect"
                onClicked: {
                  if (modelData.disconnect) {
                    modelData.disconnect();
                  }
                }
              }
            }
          }
        }

        // Paired devices section
        CrawlText {
          visible: pairedDevicesRepeater.count > 0
          text: "Paired Devices"
          font.weight: Style.fontWeightBold
          pointSize: Style.fontSizeXS
          color: Theme.cOnSurfaceVariant
          Layout.fillWidth: true
          Layout.topMargin: Style.marginS
        }

        Repeater {
          id: pairedDevicesRepeater
          model: {
            if (!BluetoothService.adapter || !BluetoothService.adapter.devices)
              return [];
            var filtered = BluetoothService.adapter.devices.values.filter(dev => dev && !dev.blocked && !dev.connected && (dev.paired || dev.trusted));
            return BluetoothService.sortDevices(BluetoothService.dedupeDevices(filtered));
          }

          delegate: CrawlBox {
            required property var modelData
            
            Layout.fillWidth: true
            implicitHeight: pairedDeviceRow.implicitHeight + Style.marginM
            color: deviceMouseArea.containsMouse ? Theme.cHover : Theme.cSurface

            RowLayout {
              id: pairedDeviceRow
              anchors.fill: parent
              anchors.margins: Style.marginS
              spacing: Style.marginS

              CrawlIcon {
                icon: "bluetooth"
                pointSize: Style.fontSizeL
                color: Theme.cOnSurfaceVariant
              }

              ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                CrawlText {
                  text: modelData.name || modelData.deviceName || "Unknown Device"
                  font.weight: Style.fontWeightMedium
                  pointSize: Style.fontSizeS
                  color: deviceMouseArea.containsMouse ? Theme.cOnHover : Theme.cOnSurface
                  elide: Text.ElideRight
                  Layout.fillWidth: true
                }

                CrawlText {
                  text: "Paired"
                  pointSize: Style.fontSizeXXS
                  color: deviceMouseArea.containsMouse ? Theme.cOnHover : Theme.cOnSurfaceVariant
                }
              }

              CrawlIconButton {
                icon: "bluetooth-connected"
                baseSize: Style.baseWidgetSize * 0.7
                tooltipText: "Connect"
                visible: deviceMouseArea.containsMouse
                onClicked: {
                  if (modelData.connect) {
                    modelData.connect();
                  }
                }
              }
            }

            MouseArea {
              id: deviceMouseArea
              anchors.fill: parent
              hoverEnabled: true
              cursorShape: Qt.PointingHandCursor
              onClicked: {
                if (modelData.connect) {
                  modelData.connect();
                }
              }
            }
          }
        }

        // Empty state when no devices
        ColumnLayout {
          visible: (!BluetoothService.connectedDevices || BluetoothService.connectedDevices.length === 0) && pairedDevicesRepeater.count === 0
          Layout.fillWidth: true
          spacing: Style.marginS

          Item { Layout.preferredHeight: Style.marginM }

          CrawlIcon {
            icon: "bluetooth"
            pointSize: Style.fontSizeXXL
            color: Theme.cOnSurfaceVariant
            Layout.alignment: Qt.AlignHCenter
          }

          CrawlText {
            text: "No devices"
            pointSize: Style.fontSizeM
            color: Theme.cOnSurfaceVariant
            horizontalAlignment: Text.AlignHCenter
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignHCenter
          }

          CrawlText {
            text: "Pair devices in Settings"
            pointSize: Style.fontSizeXS
            color: Theme.cOnSurfaceVariant
            horizontalAlignment: Text.AlignHCenter
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignHCenter
          }

          Item { Layout.preferredHeight: Style.marginM }
        }
      }
    }
  }
}
