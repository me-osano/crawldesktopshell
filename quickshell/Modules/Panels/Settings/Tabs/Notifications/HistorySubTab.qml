import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true
  enabled: Settings.data.notifications.enabled

  property var _policy: null

  Component.onCompleted: {
    if (typeof CrawlService !== "undefined" && CrawlService && CrawlService.connected) {
      CrawlService.notificationGetPolicy(function (policy) {
        root._policy = policy;
      });
    }
  }

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

  CrawlDivider {
    Layout.fillWidth: true
  }

  CrawlText {
    text: "Save to history by urgency"
    wrapMode: Text.WordWrap
    Layout.fillWidth: true
    pointSize: Style.fontSizeS
    font.weight: Style.fontWeightBold
    color: Theme.cOnSurface
  }

  CrawlToggle {
    label: "Save low urgency to history"
    description: "Save low priority notifications to history."
    checked: root._policy ? root._policy.save_to_history.low : true
    onToggled: checked => {
      if (!root._policy) return;
      root._policy.save_to_history.low = checked;
      CrawlService.notificationSetPolicy(root._policy);
    }
    defaultValue: Settings.getDefaultValue("notifications.saveToHistory.low")
  }

  CrawlToggle {
    label: "Save normal urgency to history"
    description: "Save normal priority notifications to history."
    checked: root._policy ? root._policy.save_to_history.normal : true
    onToggled: checked => {
      if (!root._policy) return;
      root._policy.save_to_history.normal = checked;
      CrawlService.notificationSetPolicy(root._policy);
    }
    defaultValue: Settings.getDefaultValue("notifications.saveToHistory.normal")
  }

  CrawlToggle {
    label: "Save critical urgency to history"
    description: "Save critical priority notifications to history."
    checked: root._policy ? root._policy.save_to_history.critical : true
    onToggled: checked => {
      if (!root._policy) return;
      root._policy.save_to_history.critical = checked;
      CrawlService.notificationSetPolicy(root._policy);
    }
    defaultValue: Settings.getDefaultValue("notifications.saveToHistory.critical")
  }
}
