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

  // Widget properties passed from Bar.qml for per-instance settings
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

  readonly property int totalUnread: {
    var accs = MailService.accounts || []
    var sum = 0
    for (var i = 0; i < accs.length; i++) sum += (accs[i].unread_count || 0)
    return sum
  }

  implicitWidth: pill.width
  implicitHeight: pill.height

  CrawlPopupContextMenu {
    id: contextMenu

    model: [
      {
        "label": "Compose",
        "action": "compose",
        "icon": "edit"
      },
      {
        "label": MailService.syncing ? "Syncing..." : "Sync now",
        "action": "sync",
        "icon": "refresh",
        "enabled": !MailService.syncing
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

                   if (action === "compose") {
                     PanelService.getPanel("mailPanel", screen)?.open();
                     Qt.callLater(function () {
                       var panel = PanelService.getPanel("mailPanel", screen);
                       if (panel) panel.activeView = "compose";
                     });
                   } else if (action === "sync") {
                     var accs = MailService.accounts || [];
                     for (var i = 0; i < accs.length; i++) {
                       MailService.syncNow(accs[i].id);
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
    icon: root.totalUnread > 0 ? "mail" : "mail-off"
    text: root.totalUnread > 0 ? String(root.totalUnread) : ""
    autoHide: false
    forceOpen: !isBarVertical && root.displayMode === "alwaysShow"
    forceClose: isBarVertical || root.displayMode === "alwaysHide"
    onClicked: {
      PanelService.getPanel("mailPanel", screen)?.toggle(this);
    }
    onRightClicked: {
      PanelService.showContextMenu(contextMenu, pill, screen);
    }
    tooltipText: {
      var count = root.totalUnread;
      if (count > 0) {
        return count + " unread" + (count === 1 ? " message" : " messages");
      }
      return "Mail";
    }
  }
}
