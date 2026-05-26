import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

CrawlIconButtonHot {
  property ShellScreen screen

  enabled: Settings.data.wallpaper.enabled
  icon: "wallpaper-selector"
  tooltipText: "Wallpaper Selector"
  onClicked: PanelService.getPanel("wallpaperPanel", screen)?.toggle()
  onRightClicked: WallpaperService.setRandomWallpaper()
}
