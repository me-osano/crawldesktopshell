import QtQuick
import QtQuick.Controls
import Quickshell
import qs.Common
import qs.Modules.Bar.Extras
import qs.Modules.Panels.Settings
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
  readonly property string iconColorKey: widgetSettings.iconColor !== undefined ? widgetSettings.iconColor : widgetMetadata.iconColor

  implicitWidth: pill.width
  implicitHeight: pill.height

  CrawlPopupContextMenu {
    id: contextMenu

    model: [
      {
        "label": "Clear clipboard history",
        "action": "clear",
        "icon": "trash",
        "enabled": ClipboardService.active && ClipboardService.items.length > 0
      },
      {
        "label": "Open clipboard panel",
        "action": "open-panel",
        "icon": "clipboard"
      },
      {
        "label": "Widget settings",
        "action": "widget-settings",
        "icon": "settings"
      }
    ]

    onTriggered: action => {
      contextMenu.close();
      PanelService.closeContextMenu(screen);

      if (action === "clear") {
        ClipboardService.wipeAll();
      } else if (action === "open-panel") {
        var p = PanelService.getPanel("clipboardPanel", screen);
        if (p) p.open();
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
    icon: ClipboardService.items && ClipboardService.items.length > 0 ? "clipboard" : "clipboard-off"
    text: {
      if (!ClipboardService.active) return "";
      if (ClipboardService.items && ClipboardService.items.length > 0) {
        return String(ClipboardService.items.length);
      }
      return "";
    }
    suffix: ""
    autoHide: false
    forceOpen: !isBarVertical
    forceClose: isBarVertical
    onClicked: {
      var p = PanelService.getPanel("clipboardPanel", screen);
      if (p) p.toggle(this);
    }
    onRightClicked: {
      PanelService.showContextMenu(contextMenu, pill, screen);
    }
    tooltipText: {
      if (!ClipboardService.active) return "Clipboard unavailable";
      return "Clipboard history";
    }
  }
}
