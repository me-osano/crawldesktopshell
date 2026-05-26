import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  // RSS Settings
  CrawlBox {
    Layout.fillWidth: true
    Layout.preferredHeight: rssCol.implicitHeight + Style.margin2L
    color: Theme.cSurface

    ColumnLayout {
      id: rssCol
      spacing: Style.marginM
      anchors.fill: parent
      anchors.margins: Style.marginL

      CrawlHeader {
        label: "RSS Feeds"
      }

      CrawlToggle {
        Layout.fillWidth: true
        label: "Enable RSS fetching"
        description: "Periodically fetch and sync RSS feed entries"
        icon: "rss"
        checked: RssService.rssEnabled
        onToggled: checked => RssService.setRssEnabled(checked)
      }
    }
  }

  // Wallhaven Settings
  CrawlBox {
    Layout.fillWidth: true
    Layout.preferredHeight: whCol.implicitHeight + Style.margin2L
    color: Theme.cSurface

    ColumnLayout {
      id: whCol
      spacing: Style.marginM
      anchors.fill: parent
      anchors.margins: Style.marginL

      CrawlHeader {
        label: "Wallhaven"
      }

      CrawlToggle {
        Layout.fillWidth: true
        label: "Enable Wallhaven integration"
        description: "Allow searching and downloading wallpapers from Wallhaven"
        icon: "wallpaper"
        checked: WallhavenService.whEnabled
        onToggled: checked => WallhavenService.setWhEnabled(checked)
      }
    }
  }

  Item {
    Layout.fillHeight: true
  }
}
