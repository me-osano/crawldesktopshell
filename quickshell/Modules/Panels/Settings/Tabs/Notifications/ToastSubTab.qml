import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  CrawlComboBox {
    label: "Position"
    description: "Where toast messages appear on screen."
    model: [
      { "key": "top", "name": "Top center" },
      { "key": "top_left", "name": "Top left" },
      { "key": "top_right", "name": "Top right" },
      { "key": "bottom", "name": "Bottom center" },
      { "key": "bottom_left", "name": "Bottom left" },
      { "key": "bottom_right", "name": "Bottom right" }
    ]
    currentKey: Settings.data.notifications.toastLocation || "top"
    onSelected: key => Settings.data.notifications.toastLocation = key
    defaultValue: Settings.getDefaultValue("notifications.toastLocation")
  }

  CrawlDivider {
    Layout.fillWidth: true
  }

  CrawlCheckbox {
    Layout.fillWidth: true
    label: "Media"
    description: "Show a toast when media playback state changes."
    checked: Settings.data.notifications.enableMediaToast
    onToggled: checked => Settings.data.notifications.enableMediaToast = checked
  }

  CrawlCheckbox {
    Layout.fillWidth: true
    label: "Keyboard layout"
    description: "Show a toast when the keyboard layout changes."
    checked: Settings.data.notifications.enableKeyboardLayoutToast
    onToggled: checked => Settings.data.notifications.enableKeyboardLayoutToast = checked
  }

  CrawlCheckbox {
    Layout.fillWidth: true
    label: "Battery warning"
    description: "Show a warning when the battery level falls below this percentage."
    checked: Settings.data.notifications.enableBatteryToast
    onToggled: checked => Settings.data.notifications.enableBatteryToast = checked
  }
}
