import QtQuick
import QtQuick.Controls
import Quickshell
import qs.Common
import qs.Modules.Bar.Extras
import qs.Services
import qs.Widgets

Item {
  id: root

  property ShellScreen screen

  property string widgetId: ""
  property string section: ""
  property int sectionWidgetIndex: -1
  property int sectionWidgetsCount: 0

  property var widgetMetadata: BarWidgetRegistry.widgetMetadata[widgetId]
  readonly property string screenName: screen ? screen.name : ""
  property var widgetSettings: {
    if (section && sectionWidgetIndex >= 0 && screenName) {
      var widgets = Settings.getBarWidgetsForScreen(screenName)[section];
      if (widgets && sectionWidgetIndex < widgets.length) {
        return widgets[sectionWidgetIndex];
      }
    }
    return {};
  }

  readonly property string barPosition: Settings.getBarPositionForScreen(screenName)
  readonly property bool isBarVertical: barPosition === "left" || barPosition === "right"
  readonly property string displayMode: widgetSettings.displayMode !== undefined ? widgetSettings.displayMode : widgetMetadata.displayMode
  readonly property string iconColorKey: widgetSettings.iconColor !== undefined ? widgetSettings.iconColor : widgetMetadata.iconColor
  readonly property string textColorKey: widgetSettings.textColor !== undefined ? widgetSettings.textColor : widgetMetadata.textColor
  readonly property bool showUnreadBadge: widgetSettings.showUnreadBadge !== undefined ? widgetSettings.showUnreadBadge : widgetMetadata.showUnreadBadge
  readonly property bool hideWhenZeroUnread: widgetSettings.hideWhenZeroUnread !== undefined ? widgetSettings.hideWhenZeroUnread : widgetMetadata.hideWhenZeroUnread

  readonly property int totalUnread: RssService.totalUnreadCount()

  implicitWidth: pill.width
  implicitHeight: pill.height

  CrawlPopupContextMenu {
    id: contextMenu

    model: [
      {
        "label": "Refresh all",
        "action": "refresh",
        "icon": "refresh"
      },
      {
        "label": "Add feed",
        "action": "add-feed",
        "icon": "plus"
      },
      {
        "label": "Widget settings",
        "action": "widget-settings",
        "icon": "settings"
      },
    ]

    onTriggered: action => {
                   contextMenu.close();
                   PanelService.closeContextMenu(screen);

                   if (action === "refresh") {
                     RssService.refreshAll();
                   } else if (action === "add-feed") {
                     var panel = PanelService.getPanel("rssPanel", screen);
                     if (panel) {
                       panel.open();
                       Qt.callLater(function () { panel.showAddFeedDialog = true; });
                     }
                   } else if (action === "widget-settings") {
                     BarService.openWidgetSettings(screen, section, sectionWidgetIndex, widgetId, widgetSettings);
                   }
                 }
  }

  BarPill {
    id: pill

    screen: root.screen
    oppositeDirection: BarService.getPillDirection(root)
    customIconColor: Theme.resolveColorKeyOptional(root.iconColorKey)
    customTextColor: Theme.resolveColorKeyOptional(root.textColorKey)
    icon: root.totalUnread > 0 ? "rss" : "rss"
    text: root.totalUnread > 0 ? String(root.totalUnread) : ""
    autoHide: false
    forceOpen: !isBarVertical && root.displayMode === "alwaysShow"
    forceClose: isBarVertical || root.displayMode === "alwaysHide"
    onClicked: {
      PanelService.getPanel("rssPanel", screen)?.toggle(this);
    }
    onRightClicked: {
      PanelService.showContextMenu(contextMenu, pill, screen);
    }
    tooltipText: {
      var count = root.totalUnread;
      if (count > 0) {
        return count + " unread" + (count === 1 ? " entry" : " entries");
      }
      return "RSS Feeds";
    }
  }
}
