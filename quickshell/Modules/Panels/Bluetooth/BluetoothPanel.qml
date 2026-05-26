import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Bluetooth
import "../Settings/Tabs/Bluetooth" as BluetoothPrefs
import qs.Common
import qs.Modules.MainScreen
import qs.Modules.Panels.Settings
import qs.Services
import qs.Widgets

SmartPanel {
  id: root

  preferredWidth: Math.round(440 * Style.uiScaleRatio)
  preferredHeight: Math.round(500 * Style.uiScaleRatio)

  panelContent: Rectangle {
    id: panelContent
    color: "transparent"

    property real contentPreferredHeight: Math.min(root.preferredHeight, mainColumn.implicitHeight + Style.margin2L)

    ColumnLayout {
      id: mainColumn
      anchors.fill: parent
      anchors.margins: Style.marginL
      spacing: Style.marginM

      // Header
      CrawlBox {
        Layout.fillWidth: true
        Layout.preferredHeight: headerRow.implicitHeight + Style.margin2M

        RowLayout {
          id: headerRow
          anchors.fill: parent
          anchors.margins: Style.marginM

          CrawlIcon {
            icon: BluetoothService.enabled ? "bluetooth" : "bluetooth-off"
            pointSize: Style.fontSizeXXL
            color: BluetoothService.enabled ? Theme.cPrimary : Theme.cOnSurfaceVariant
          }

          CrawlLabel {
            label: "Bluetooth"
            Layout.fillWidth: true
          }

          CrawlToggle {
            id: bluetoothSwitch
            checked: BluetoothService.enabled
            enabled: !Settings.data.network.airplaneModeEnabled && BluetoothService.bluetoothAvailable
            onToggled: checked => BluetoothService.setBluetoothEnabled(checked)
            baseSize: Style.baseWidgetSize * 0.65
          }

          CrawlIconButton {
            icon: "settings"
            tooltipText: "Settings"
            baseSize: Style.baseWidgetSize * 0.8
            onClicked: SettingsPanelService.openToTab(SettingsPanel.Tab.Connections, 1, screen)
          }

          CrawlIconButton {
            icon: "close"
            tooltipText: "Close"
            baseSize: Style.baseWidgetSize * 0.8
            onClicked: {
              root.close();
            }
          }
        }
      }

      CrawlScrollView {
        id: bluetoothScrollView
        Layout.fillWidth: true
        Layout.fillHeight: true
        horizontalPolicy: ScrollBar.AlwaysOff
        verticalPolicy: ScrollBar.AsNeeded
        reserveScrollbarSpace: false
        gradientColor: Theme.cSurface

        ColumnLayout {
          id: devicesList
          width: bluetoothScrollView.availableWidth
          spacing: Style.marginM

          // Adapter not available of disabled
          CrawlBox {
            id: disabledBox
            visible: !BluetoothService.enabled
            Layout.fillWidth: true
            Layout.preferredHeight: disabledColumn.implicitHeight + Style.margin2M

            ColumnLayout {
              id: disabledColumn
              anchors.fill: parent
              anchors.margins: Style.marginM
              spacing: Style.marginL

              Item {
                Layout.fillHeight: true
              }

              CrawlIcon {
                icon: "bluetooth-off"
                pointSize: 48
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlText {
                text: "Bluetooth is disabled"
                pointSize: Style.fontSizeL
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlText {
                text: "Enable Bluetooth to see available devices."
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
              }

              Item {
                Layout.fillHeight: true
              }
            }
          }

          // Empty state when no paired devices
          CrawlBox {
            id: emptyBox
            visible: {
              if (!BluetoothService.enabled || !BluetoothService.devices)
                return false;
              // Pulling pairedDevices count from the source component
              return (btSource.pairedDevices.length === 0 && btSource.connectedDevices.length === 0);
            }
            Layout.fillWidth: true
            Layout.preferredHeight: emptyColumn.implicitHeight + Style.margin2M

            ColumnLayout {
              id: emptyColumn
              anchors.fill: parent
              anchors.margins: Style.marginM
              spacing: Style.marginL

              Item {
                Layout.fillHeight: true
              }

              CrawlIcon {
                icon: "bluetooth"
                pointSize: 48
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlText {
                text: "No devices available"
                pointSize: Style.fontSizeL
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlButton {
                text: "Settings"
                icon: "settings"
                Layout.alignment: Qt.AlignHCenter
                onClicked: SettingsPanelService.openToTab(SettingsPanel.Tab.Connections, 1, screen)
              }

              Item {
                Layout.fillHeight: true
              }
            }
          }

          // Pull connected/paired lists from BluetoothSubTab
          BluetoothPrefs.BluetoothTab {
            id: btSource
            Layout.fillWidth: true
            showOnlyLists: true
            visible: !disabledBox.visible && !emptyBox.visible
          }
        }
      }
    }
  }
}
