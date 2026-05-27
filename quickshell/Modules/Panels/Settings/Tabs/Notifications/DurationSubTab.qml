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
    label: "Respect expire timeout"
    description: "Use the expire timeout set in the notification."
    checked: root._policy ? root._policy.respect_expire_timeout : false
    onToggled: checked => {
      if (!root._policy) return;
      root._policy.respect_expire_timeout = checked;
      CrawlService.notificationSetPolicy(root._policy);
    }
    defaultValue: Settings.getDefaultValue("notifications.respectExpireTimeout")
  }

  CrawlValueSlider {
    Layout.fillWidth: true
    label: "Low urgency"
    description: "How long low priority notifications stay visible."
    from: 1
    to: 30
    stepSize: 1
    showReset: true
    value: root._policy ? (root._policy.low_urgency_duration_ms / 1000) : 3
    onMoved: value => {
      if (!root._policy) return;
      root._policy.low_urgency_duration_ms = value * 1000;
      CrawlService.notificationSetPolicy(root._policy);
    }
    text: root._policy ? (root._policy.low_urgency_duration_ms / 1000) + "s" : "3s"
    defaultValue: Settings.getDefaultValue("notifications.lowUrgencyDuration")
  }

  CrawlValueSlider {
    Layout.fillWidth: true
    label: "Normal urgency"
    description: "How long normal priority notifications stay visible."
    from: 1
    to: 30
    stepSize: 1
    showReset: true
    value: root._policy ? (root._policy.normal_urgency_duration_ms / 1000) : 8
    onMoved: value => {
      if (!root._policy) return;
      root._policy.normal_urgency_duration_ms = value * 1000;
      CrawlService.notificationSetPolicy(root._policy);
    }
    text: root._policy ? (root._policy.normal_urgency_duration_ms / 1000) + "s" : "8s"
    defaultValue: Settings.getDefaultValue("notifications.normalUrgencyDuration")
  }

  CrawlValueSlider {
    Layout.fillWidth: true
    label: "Critical urgency"
    description: "How long critical priority notifications stay visible."
    from: 1
    to: 30
    stepSize: 1
    showReset: true
    value: root._policy ? (root._policy.critical_urgency_duration_ms / 1000) : 15
    onMoved: value => {
      if (!root._policy) return;
      root._policy.critical_urgency_duration_ms = value * 1000;
      CrawlService.notificationSetPolicy(root._policy);
    }
    text: root._policy ? (root._policy.critical_urgency_duration_ms / 1000) + "s" : "15s"
    defaultValue: Settings.getDefaultValue("notifications.criticalUrgencyDuration")
  }
}
