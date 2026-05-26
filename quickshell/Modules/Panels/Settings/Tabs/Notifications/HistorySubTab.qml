import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true
  enabled: Settings.data.notifications.enabled

  CrawlToggle {
    label: "Clear on dismissed"
    description: "Clear notification from history when dismissed."
    checked: Settings.data.notifications.clearDismissed
    onToggled: checked => Settings.data.notifications.clearDismissed = checked
    defaultValue: Settings.getDefaultValue("notifications.clearDismissed")
  }

  CrawlToggle {
    label: "Enable Markdown"
    description: "Render notification content using Markdown formatting."
    checked: Settings.data.notifications.enableMarkdown
    onToggled: checked => Settings.data.notifications.enableMarkdown = checked
    defaultValue: Settings.getDefaultValue("notifications.enableMarkdown")
  }

  CrawlToggle {
    label: "Save low urgency to history"
    description: "Save low priority notifications to history."
    checked: Settings.data.notifications?.saveToHistory?.low !== false
    onToggled: checked => Settings.data.notifications.saveToHistory.low = checked
    defaultValue: Settings.getDefaultValue("notifications.saveToHistory.low")
  }

  CrawlToggle {
    label: "Save normal urgency to history"
    description: "Save normal priority notifications to history."
    checked: Settings.data.notifications?.saveToHistory?.normal !== false
    onToggled: checked => Settings.data.notifications.saveToHistory.normal = checked
    defaultValue: Settings.getDefaultValue("notifications.saveToHistory.normal")
  }

  CrawlToggle {
    label: "Save critical urgency to history"
    description: "Save critical priority notifications to history."
    checked: Settings.data.notifications?.saveToHistory?.critical !== false
    onToggled: checked => Settings.data.notifications.saveToHistory.critical = checked
    defaultValue: Settings.getDefaultValue("notifications.saveToHistory.critical")
  }
}
