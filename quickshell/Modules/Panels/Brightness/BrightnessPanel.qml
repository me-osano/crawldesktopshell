import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Modules.MainScreen
import qs.Services
import qs.Widgets

SmartPanel {
  id: root

  preferredWidth: Math.round(440 * Style.uiScaleRatio)
  preferredHeight: Math.round(420 * Style.uiScaleRatio)

  panelContent: Item {
    id: panelContent
    property real contentPreferredHeight: mainColumn.implicitHeight + Style.margin2L

    property var brightnessWidgetInstance: BarService.lookupWidget("Brightness", screen ? screen.name : null)
    readonly property var brightnessWidgetSettings: brightnessWidgetInstance ? brightnessWidgetInstance.widgetSettings : null
    readonly property var brightnessWidgetMetadata: BarWidgetRegistry.widgetMetadata["Brightness"]

    function resolveWidgetSetting(key, defaultValue) {
      if (brightnessWidgetSettings && brightnessWidgetSettings[key] !== undefined)
        return brightnessWidgetSettings[key];
      if (brightnessWidgetMetadata && brightnessWidgetMetadata[key] !== undefined)
        return brightnessWidgetMetadata[key];
      return defaultValue;
    }

    Connections {
      target: BarService
      function onActiveWidgetsChanged() {
        panelContent.brightnessWidgetInstance = BarService.lookupWidget("Brightness", screen ? screen.name : null);
      }
    }

    property real globalBrightness: 0
    property bool globalBrightnessChanging: false
    property int globalBrightnessCapableMonitors: 0

    function getIcon(brightness) {
      return brightness <= 0.5 ? "brightness-low" : "brightness-high";
    }

    function getControllableMonitors() {
      var monitors = BrightnessService.monitors || [];
      return monitors.filter(m => m && m.brightnessControlAvailable);
    }

    function updateGlobalBrightness() {
      var monitors = getControllableMonitors();
      panelContent.globalBrightnessCapableMonitors = monitors.length;

      if (panelContent.globalBrightnessChanging)
        return;

      if (monitors.length === 0) {
        panelContent.globalBrightness = 0;
        return;
      }

      var total = 0;
      monitors.forEach(m => {
                         var brightnessValue = isNaN(m.brightness) ? 0 : m.brightness;
                         total += brightnessValue;
                       });
      panelContent.globalBrightness = total / monitors.length;
    }

    function applyGlobalBrightness(value) {
      var monitors = BrightnessService.monitors || [];
      monitors.forEach(m => {
                         if (m && m.brightnessControlAvailable) {
                           m.setBrightness(value);
                         }
                       });
    }

    Component.onCompleted: updateGlobalBrightness()

    Connections {
      target: BrightnessService
      function onMonitorBrightnessChanged(monitor, newBrightness) {
        panelContent.updateGlobalBrightness();
      }
      function onMonitorsChanged() {
        panelContent.updateGlobalBrightness();
      }
      function onDdcMonitorsChanged() {
        panelContent.updateGlobalBrightness();
      }
    }

    ColumnLayout {
      id: mainColumn
      anchors.fill: parent
      anchors.margins: Style.marginL
      spacing: Style.marginM

      // HEADER
      CrawlBox {
        Layout.fillWidth: true
        implicitHeight: headerRow.implicitHeight + Style.margin2M

        RowLayout {
          id: headerRow
          anchors.fill: parent
          anchors.margins: Style.marginM
          spacing: Style.marginM

          CrawlIcon {
            icon: "settings-display"
            pointSize: Style.fontSizeXXL
            color: Theme.cPrimary
          }

          CrawlText {
            text: "Display"
            pointSize: Style.fontSizeL
            font.weight: Style.fontWeightBold
            color: Theme.cOnSurface
            Layout.fillWidth: true
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
        id: brightnessScrollView
        Layout.fillWidth: true
        Layout.fillHeight: true
        horizontalPolicy: ScrollBar.AlwaysOff
        verticalPolicy: ScrollBar.AsNeeded
        contentWidth: availableWidth
        reserveScrollbarSpace: false
        gradientColor: Theme.cSurface

        // AudioService Devices
        ColumnLayout {
          spacing: Style.marginM
          width: brightnessScrollView.availableWidth

          CrawlBox {
            Layout.fillWidth: true
            visible: panelContent.globalBrightnessCapableMonitors > 1 && panelContent.resolveWidgetSetting("applyToAllMonitors", false)
            implicitHeight: globalBrightnessContent.implicitHeight + (Style.marginXL)

            ColumnLayout {
              id: globalBrightnessContent
              anchors.left: parent.left
              anchors.right: parent.right
              anchors.top: parent.top
              anchors.margins: Style.marginM
              spacing: Style.marginS

              CrawlLabel {
                label: "All monitors"
                description: "Adjust brightness for all monitors at once."
              }

              RowLayout {
                Layout.fillWidth: true
                spacing: Style.marginS

                CrawlIcon {
                  icon: panelContent.getIcon(panelContent.globalBrightness)
                  pointSize: Style.fontSizeXL
                  color: Theme.cOnSurface
                }

                CrawlValueSlider {
                  id: globalBrightnessSlider
                  from: 0
                  to: 1
                  value: panelContent.globalBrightness
                  stepSize: 0.01
                  enabled: panelContent.globalBrightnessCapableMonitors > 0
                  onMoved: value => {
                             panelContent.globalBrightness = value;
                             panelContent.applyGlobalBrightness(value);
                           }
                  onPressedChanged: (pressed, value) => {
                                      panelContent.globalBrightnessChanging = pressed;
                                      panelContent.globalBrightness = value;
                                      panelContent.applyGlobalBrightness(value);
                                    }
                  Layout.fillWidth: true
                  text: ""
                }

                CrawlText {
                  text: panelContent.globalBrightnessCapableMonitors > 0 ? Math.round(panelContent.globalBrightness * 100) + "%" : "N/A"
                  Layout.preferredWidth: 55
                  horizontalAlignment: Text.AlignRight
                  Layout.alignment: Qt.AlignVCenter
                }
              }
            }
          }

          Repeater {
            model: Quickshell.screens || []
            delegate: CrawlBox {
              Layout.fillWidth: true
              Layout.preferredHeight: outputColumn.implicitHeight + Style.margin2M

              property var brightnessMonitor: BrightnessService.getMonitorForScreen(modelData)

              ColumnLayout {
                id: outputColumn
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: Style.marginM
                spacing: Style.marginS

                CrawlLabel {
                  label: modelData.name || "Unknown"
                  labelColor: Theme.cPrimary
                  description: {
                    const compositorScale = CompositorService.getDisplayScale(modelData.name);
                    "{model} ({width}x{height} @ {scale}x)";
                  }
                }

                RowLayout {

                  Layout.fillWidth: true
                  spacing: Style.marginS
                  CrawlIcon {
                    icon: getIcon(brightnessMonitor ? brightnessMonitor.brightness : 0)
                    pointSize: Style.fontSizeXL
                    color: Theme.cOnSurface
                  }

                  CrawlValueSlider {
                    id: brightnessSlider
                    from: 0
                    to: 1
                    value: brightnessMonitor ? brightnessMonitor.brightness : 0.5
                    stepSize: 0.01
                    enabled: brightnessMonitor ? brightnessMonitor.brightnessControlAvailable : false
                    onMoved: value => {
                               if (brightnessMonitor && brightnessMonitor.brightnessControlAvailable) {
                                 brightnessMonitor.setBrightness(value);
                               }
                             }
                    onPressedChanged: (pressed, value) => {
                                        if (brightnessMonitor && brightnessMonitor.brightnessControlAvailable) {
                                          brightnessMonitor.setBrightness(value);
                                        }
                                      }
                    Layout.fillWidth: true
                    text: brightnessMonitor ? Math.round(brightnessSlider.value * 100) + "%" : "N/A"
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
