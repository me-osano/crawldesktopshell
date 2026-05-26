import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM
  Layout.fillWidth: true

  CrawlLabel {
    Layout.fillWidth: true
    description: "Adjust warning/critical thresholds for each system metric."
  }

  GridLayout {
    Layout.fillWidth: true
    Layout.topMargin: Style.marginM
    columns: 3
    columnSpacing: Style.marginM
    rowSpacing: Style.marginM

    // Header row
    Item {
      Layout.fillWidth: true
    }

    CrawlText {
      Layout.alignment: Qt.AlignHCenter
      horizontalAlignment: Text.AlignHCenter
      text: "Warning threshold"
      pointSize: Style.fontSizeS
      color: Theme.cOnSurfaceVariant
    }

    CrawlText {
      Layout.alignment: Qt.AlignHCenter
      horizontalAlignment: Text.AlignHCenter
      text: "Critical threshold"
      pointSize: Style.fontSizeS
      color: Theme.cOnSurfaceVariant
    }

    // CPU Usage
    CrawlText {
      text: "CPU usage"
      pointSize: Style.fontSizeM
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.cpuWarningThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.cpuWarningThreshold")
      suffix: "%"
      onValueChanged: {
        Settings.data.systemMonitor.cpuWarningThreshold = value;
        if (Settings.data.systemMonitor.cpuCriticalThreshold < value) {
          Settings.data.systemMonitor.cpuCriticalThreshold = value;
        }
      }
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: Settings.data.systemMonitor.cpuWarningThreshold
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.cpuCriticalThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.cpuCriticalThreshold")
      suffix: "%"
      onValueChanged: Settings.data.systemMonitor.cpuCriticalThreshold = value
    }

    // CPU Temperature
    CrawlText {
      text: "CPU temperature"
      pointSize: Style.fontSizeM
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.tempWarningThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.tempWarningThreshold")
      suffix: "°C"
      onValueChanged: {
        Settings.data.systemMonitor.tempWarningThreshold = value;
        if (Settings.data.systemMonitor.tempCriticalThreshold < value) {
          Settings.data.systemMonitor.tempCriticalThreshold = value;
        }
      }
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: Settings.data.systemMonitor.tempWarningThreshold
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.tempCriticalThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.tempCriticalThreshold")
      suffix: "°C"
      onValueChanged: Settings.data.systemMonitor.tempCriticalThreshold = value
    }

    // GPU Temperature
    CrawlText {
      visible: SystemStatService.gpuAvailable
      text: "GPU temperature"
      pointSize: Style.fontSizeM
    }

    CrawlSpinBox {
      visible: SystemStatService.gpuAvailable
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 120
      stepSize: 5
      value: Settings.data.systemMonitor.gpuWarningThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.gpuWarningThreshold")
      suffix: "°C"
      onValueChanged: {
        Settings.data.systemMonitor.gpuWarningThreshold = value;
        if (Settings.data.systemMonitor.gpuCriticalThreshold < value) {
          Settings.data.systemMonitor.gpuCriticalThreshold = value;
        }
      }
    }

    CrawlSpinBox {
      visible: SystemStatService.gpuAvailable
      Layout.alignment: Qt.AlignHCenter
      from: Settings.data.systemMonitor.gpuWarningThreshold
      to: 120
      stepSize: 5
      value: Settings.data.systemMonitor.gpuCriticalThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.gpuCriticalThreshold")
      suffix: "°C"
      onValueChanged: Settings.data.systemMonitor.gpuCriticalThreshold = value
    }

    // Memory Usage
    CrawlText {
      text: "Memory usage"
      pointSize: Style.fontSizeM
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.memWarningThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.memWarningThreshold")
      suffix: "%"
      onValueChanged: {
        Settings.data.systemMonitor.memWarningThreshold = value;
        if (Settings.data.systemMonitor.memCriticalThreshold < value) {
          Settings.data.systemMonitor.memCriticalThreshold = value;
        }
      }
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: Settings.data.systemMonitor.memWarningThreshold
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.memCriticalThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.memCriticalThreshold")
      suffix: "%"
      onValueChanged: Settings.data.systemMonitor.memCriticalThreshold = value
    }

    // Swap Usage
    CrawlText {
      text: "Swap usage"
      pointSize: Style.fontSizeM
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.swapWarningThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.swapWarningThreshold")
      suffix: "%"
      onValueChanged: {
        Settings.data.systemMonitor.swapWarningThreshold = value;
        if (Settings.data.systemMonitor.swapCriticalThreshold < value) {
          Settings.data.systemMonitor.swapCriticalThreshold = value;
        }
      }
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: Settings.data.systemMonitor.swapWarningThreshold
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.swapCriticalThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.swapCriticalThreshold")
      suffix: "%"
      onValueChanged: Settings.data.systemMonitor.swapCriticalThreshold = value
    }

    // Disk Usage
    CrawlText {
      text: "Disk usage"
      pointSize: Style.fontSizeM
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.diskWarningThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.diskWarningThreshold")
      suffix: "%"
      onValueChanged: {
        Settings.data.systemMonitor.diskWarningThreshold = value;
        if (Settings.data.systemMonitor.diskCriticalThreshold < value) {
          Settings.data.systemMonitor.diskCriticalThreshold = value;
        }
      }
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: Settings.data.systemMonitor.diskWarningThreshold
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.diskCriticalThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.diskCriticalThreshold")
      suffix: "%"
      onValueChanged: Settings.data.systemMonitor.diskCriticalThreshold = value
    }

    // Disk Available
    CrawlText {
      text: "Disk available"
      pointSize: Style.fontSizeM
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.diskAvailWarningThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.diskAvailWarningThreshold")
      suffix: "%"
      onValueChanged: {
        Settings.data.systemMonitor.diskAvailWarningThreshold = value;
        if (Settings.data.systemMonitor.diskAvailCriticalThreshold > value) {
          Settings.data.systemMonitor.diskAvailCriticalThreshold = value;
        }
      }
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 20
      stepSize: 5
      value: Settings.data.systemMonitor.diskAvailCriticalThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.diskAvailCriticalThreshold")
      suffix: "%"
      onValueChanged: Settings.data.systemMonitor.diskAvailCriticalThreshold = value
    }

    // Battery
    CrawlText {
      text: "Battery warning"
      pointSize: Style.fontSizeM
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: 100
      stepSize: 5
      value: Settings.data.systemMonitor.batteryWarningThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.batteryWarningThreshold")
      suffix: "%"
      onValueChanged: {
        Settings.data.systemMonitor.batteryWarningThreshold = value;
        if (Settings.data.systemMonitor.batteryCriticalThreshold > value) {
          Settings.data.systemMonitor.batteryCriticalThreshold = value;
        }
      }
    }

    CrawlSpinBox {
      Layout.alignment: Qt.AlignHCenter
      from: 0
      to: Settings.data.systemMonitor.batteryWarningThreshold
      stepSize: 5
      value: Settings.data.systemMonitor.batteryCriticalThreshold
      defaultValue: Settings.getDefaultValue("systemMonitor.batteryCriticalThreshold")
      suffix: "%"
      onValueChanged: Settings.data.systemMonitor.batteryCriticalThreshold = value
    }
  }
}
