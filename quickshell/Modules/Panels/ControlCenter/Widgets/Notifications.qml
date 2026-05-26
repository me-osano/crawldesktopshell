import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

CrawlIconButtonHot {
  property ShellScreen screen

  icon: NotificationService.doNotDisturb ? "bell-off" : "bell"
  hot: NotificationService.doNotDisturb
  tooltipText: "Notifications"
  onClicked: {
    NotificationService.updateLastSeenTs();
    PanelService.getPanel("controlCenterPanel", screen)?.open();
  }
  onRightClicked: NotificationService.doNotDisturb = !NotificationService.doNotDisturb
}
