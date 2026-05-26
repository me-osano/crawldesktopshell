import QtQuick.Layouts
import Quickshell
import Quickshell.Services.UPower
import qs.Common
import qs.Services
import qs.Widgets

// Performance
CrawlIconButtonHot {
  property ShellScreen screen

  readonly property bool hasPP: PowerProfileService.available

  enabled: hasPP
  icon: PowerProfileService.getIcon()
  hot: !PowerProfileService.isDefault()
  tooltipText: "Power Profile"
  onClicked: PowerProfileService.cycleProfile()
}
