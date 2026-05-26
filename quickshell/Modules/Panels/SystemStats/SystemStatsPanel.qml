import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Modules.MainScreen
import qs.Services
import qs.Widgets

SmartPanel {
  id: root

  Component.onCompleted: SystemStatService.registerComponent("panel-systemstats")
  Component.onDestruction: SystemStatService.unregisterComponent("panel-systemstats")

  preferredWidth: Math.round(440 * Style.uiScaleRatio)

  panelContent: Item {
    id: panelContent
    property real contentPreferredHeight: mainColumn.implicitHeight + Style.margin2L
    readonly property real cardHeight: 90 * Style.uiScaleRatio

    // Get diskPath from bar's SystemMonitor widget if available, otherwise use "/"
    readonly property string diskPath: {
      const sysMonWidget = BarService.lookupWidget("SystemMonitor");
      if (sysMonWidget && sysMonWidget.diskPath) {
        return sysMonWidget.diskPath;
      }
      return "/";
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
            icon: "device-analytics"
            pointSize: Style.fontSizeXXL
            color: Theme.cPrimary
          }

          CrawlText {
            text: "System Monitor"
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

      // CPU Card (dual-line: usage % + temperature °C)
      CrawlBox {
        Layout.fillWidth: true
        Layout.preferredHeight: panelContent.cardHeight

        ColumnLayout {
          anchors.fill: parent
          anchors.margins: Style.marginS
          anchors.bottomMargin: Style.radiusM * 0.5
          spacing: Style.marginXS

          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginXS

            CrawlIcon {
              icon: "cpu-usage"
              pointSize: Style.fontSizeXS
              color: Theme.cPrimary
            }

            CrawlText {
              text: `${Math.round(SystemStatService.cpuUsage)}% (${SystemStatService.cpuFreq.replace(/[^0-9.]/g, "")} GHz)`
              pointSize: Style.fontSizeXS
              color: Theme.cPrimary
              font.family: Settings.data.ui.fontFixed
            }

            CrawlIcon {
              icon: "cpu-temperature"
              pointSize: Style.fontSizeXS
              color: Theme.cSecondary
            }

            CrawlText {
              text: `${Math.round(SystemStatService.cpuTemp)}°C`
              pointSize: Style.fontSizeXS
              color: Theme.cSecondary
              font.family: Settings.data.ui.fontFixed
              Layout.rightMargin: Style.marginS
            }

            Item {
              Layout.fillWidth: true
            }

            CrawlText {
              text: "CPU usage"
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
            }
          }

          CrawlGraph {
            Layout.fillWidth: true
            Layout.fillHeight: true
            values: SystemStatService.cpuHistory
            values2: SystemStatService.cpuTempHistory
            minValue: 0
            maxValue: 100
            minValue2: Math.max(SystemStatService.cpuTempHistoryMin - 5, 0)
            maxValue2: Math.max(SystemStatService.cpuTempHistoryMax + 5, 1)
            color: Theme.cPrimary
            color2: Theme.cSecondary
            fill: true
            fillOpacity: 0.15
            updateInterval: SystemStatService.cpuUsageIntervalMs
          }
        }
      }

      // Memory Card (single-line + optional swap indicator)
      CrawlBox {
        Layout.fillWidth: true
        Layout.preferredHeight: panelContent.cardHeight

        ColumnLayout {
          anchors.fill: parent
          anchors.margins: Style.marginS
          anchors.bottomMargin: Style.radiusM * 0.5
          spacing: Style.marginXS

          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginXS

            CrawlIcon {
              icon: "memory"
              pointSize: Style.fontSizeXS
              color: Theme.cPrimary
            }

            CrawlText {
              text: `${Math.round(SystemStatService.memPercent)}% (${SystemStatService.formatGigabytes(SystemStatService.memGb).replace(/[^0-9.]/g, "")} GB)`
              pointSize: Style.fontSizeXS
              color: Theme.cPrimary
              font.family: Settings.data.ui.fontFixed
            }

            Item {
              Layout.fillWidth: true
            }

            CrawlText {
              text: "Memory"
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
            }
          }

          CrawlGraph {
            Layout.fillWidth: true
            Layout.fillHeight: true
            values: SystemStatService.memHistory
            minValue: 0
            maxValue: 100
            color: Theme.cPrimary
            fill: true
            fillOpacity: 0.15
            updateInterval: SystemStatService.memIntervalMs
          }
        }
      }

      // Network Card (dual-line: RX + TX speeds)
      CrawlBox {
        Layout.fillWidth: true
        Layout.preferredHeight: panelContent.cardHeight

        ColumnLayout {
          anchors.fill: parent
          anchors.margins: Style.marginS
          anchors.bottomMargin: Style.radiusM * 0.5
          spacing: Style.marginXS

          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginXS

            CrawlIcon {
              icon: "download-speed"
              pointSize: Style.fontSizeXS
              color: Theme.cPrimary
            }

            CrawlText {
              text: SystemStatService.formatSpeed(SystemStatService.rxSpeed).replace(/([0-9.]+)([A-Za-z]+)/, "$1 $2") + "/s"
              pointSize: Style.fontSizeXS
              color: Theme.cPrimary
              font.family: Settings.data.ui.fontFixed
              Layout.rightMargin: Style.marginS
            }

            CrawlIcon {
              icon: "upload-speed"
              pointSize: Style.fontSizeXS
              color: Theme.cSecondary
            }

            CrawlText {
              text: SystemStatService.formatSpeed(SystemStatService.txSpeed).replace(/([0-9.]+)([A-Za-z]+)/, "$1 $2") + "/s"
              pointSize: Style.fontSizeXS
              color: Theme.cSecondary
              font.family: Settings.data.ui.fontFixed
            }

            Item {
              Layout.fillWidth: true
            }

            CrawlText {
              text: "Network"
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
            }
          }

          CrawlGraph {
            Layout.fillWidth: true
            Layout.fillHeight: true
            values: SystemStatService.rxSpeedHistory
            values2: SystemStatService.txSpeedHistory
            minValue: 0
            maxValue: SystemStatService.rxMaxSpeed
            minValue2: 0
            maxValue2: SystemStatService.txMaxSpeed
            color: Theme.cPrimary
            color2: Theme.cSecondary
            fill: true
            fillOpacity: 0.15
            updateInterval: SystemStatService.networkIntervalMs
            animateScale: true
          }
        }
      }

      // Detailed Stats section
      CrawlBox {
        Layout.fillWidth: true
        implicitHeight: detailsColumn.implicitHeight + Style.margin2M

        ColumnLayout {
          id: detailsColumn
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.top: parent.top
          anchors.margins: Style.marginM
          spacing: Style.marginXS

          // Load Average
          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginS
            visible: SystemStatService.nproc > 0

            CrawlIcon {
              icon: "cpu-usage"
              pointSize: Style.fontSizeM
              color: Theme.cPrimary
            }

            CrawlText {
              text: "Load average" + ":"
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
            }

            CrawlText {
              text: `${SystemStatService.loadAvg1.toFixed(2)} • ${SystemStatService.loadAvg5.toFixed(2)} • ${SystemStatService.loadAvg15.toFixed(2)}`
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurface
              Layout.fillWidth: true
              horizontalAlignment: Text.AlignRight
            }
          }

          // GPU Temperature (only if available)
          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginS
            visible: SystemStatService.gpuAvailable

            CrawlIcon {
              icon: "gpu-temperature"
              pointSize: Style.fontSizeM
              color: Theme.cPrimary
            }

            CrawlText {
              text: "GPU temp" + ":"
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
            }

            CrawlText {
              text: `${Math.round(SystemStatService.gpuTemp)}°C`
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurface
              Layout.fillWidth: true
              horizontalAlignment: Text.AlignRight
            }
          }

          // Disk usage
          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginS

            CrawlIcon {
              icon: "storage"
              pointSize: Style.fontSizeM
              color: Theme.cPrimary
            }

            CrawlText {
              text: "Disk" + ":"
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
            }

            CrawlText {
              text: {
                const usedGb = SystemStatService.diskUsedGb[panelContent.diskPath] || 0;
                const sizeGb = SystemStatService.diskSizeGb[panelContent.diskPath] || 0;
                const percent = SystemStatService.diskPercents[panelContent.diskPath] || 0;
                return `${percent}% (${usedGb.toFixed(1)} / ${sizeGb.toFixed(1)} GB)`;
              }
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurface
              Layout.fillWidth: true
              horizontalAlignment: Text.AlignRight
              elide: Text.ElideMiddle
            }
          }

          // Swap details (only visible if swap is enabled)
          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginS
            visible: SystemStatService.swapTotalGb > 0

            CrawlIcon {
              icon: "exchange"
              pointSize: Style.fontSizeM
              color: Theme.cPrimary
            }

            CrawlText {
              text: "Swap usage" + ":"
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
            }

            CrawlText {
              text: `${SystemStatService.formatGigabytes(SystemStatService.swapGb).replace(/[^0-9.]/g, "")} / ${SystemStatService.formatGigabytes(SystemStatService.swapTotalGb).replace(/[^0-9.]/g, "")} GB`
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurface
              Layout.fillWidth: true
              horizontalAlignment: Text.AlignRight
            }
          }
        }
      }
    }
  }
}
