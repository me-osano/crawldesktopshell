import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM
  width: 700

  property var screen: null
  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  property string valueDisplayMode: widgetData.displayMode !== undefined ? widgetData.displayMode : widgetMetadata.displayMode
  property string valueIconColor: widgetData.iconColor !== undefined ? widgetData.iconColor : widgetMetadata.iconColor
  property string valueTextColor: widgetData.textColor !== undefined ? widgetData.textColor : widgetMetadata.textColor
  property bool valueShowUnreadBadge: widgetData.showUnreadBadge !== undefined ? widgetData.showUnreadBadge : widgetMetadata.showUnreadBadge
  property bool valueHideWhenZeroUnread: widgetData.hideWhenZeroUnread !== undefined ? widgetData.hideWhenZeroUnread : widgetMetadata.hideWhenZeroUnread

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.displayMode = valueDisplayMode;
    settings.iconColor = valueIconColor;
    settings.textColor = valueTextColor;
    settings.showUnreadBadge = valueShowUnreadBadge;
    settings.hideWhenZeroUnread = valueHideWhenZeroUnread;
    settingsChanged(settings);
    return settings;
  }

  CrawlHeader {
    label: "RSS Widget"
    description: "Configure the RSS bar widget appearance and behavior."
  }

  CrawlComboBox {
    Layout.fillWidth: true
    label: "Display mode"
    description: "How the icon is displayed in the bar."
    minimumWidth: 200
    model: [
      { "key": "always", "name": "Always show" },
      { "key": "onhover", "name": "On hover" },
      { "key": "hidden", "name": "Hidden" },
      { "key": "expanded", "name": "Expanded" }
    ]
    currentKey: valueDisplayMode
    onSelected: key => { valueDisplayMode = key; saveSettings(); }
  }

  CrawlToggle {
    Layout.fillWidth: true
    label: "Show unread badge"
    description: "Display the unread count badge on the RSS icon."
    checked: valueShowUnreadBadge
    onToggled: checked => { valueShowUnreadBadge = checked; saveSettings(); }
  }

  CrawlToggle {
    Layout.fillWidth: true
    label: "Hide when zero unread"
    description: "Hide the widget entirely when there are no unread entries."
    checked: valueHideWhenZeroUnread
    onToggled: checked => { valueHideWhenZeroUnread = checked; saveSettings(); }
  }
}
