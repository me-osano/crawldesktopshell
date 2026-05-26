import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: 0

  CrawlTabBar {
    id: subTabBar
    Layout.fillWidth: true
    Layout.bottomMargin: Style.marginM
    distributeEvenly: true
    currentIndex: tabView.currentIndex

    CrawlTabButton {
      text: "Volumes"
      tabIndex: 0
      checked: subTabBar.currentIndex === 0
    }
    CrawlTabButton {
      text: "Devices"
      tabIndex: 1
      checked: subTabBar.currentIndex === 1
    }
    CrawlTabButton {
      text: "Media"
      tabIndex: 2
      checked: subTabBar.currentIndex === 2
    }
    CrawlTabButton {
      text: "Visualizer"
      tabIndex: 3
      checked: subTabBar.currentIndex === 3
    }
  }

  Item {
    Layout.fillWidth: true
    Layout.preferredHeight: Style.marginL
  }

  CrawlTabView {
    id: tabView
    currentIndex: subTabBar.currentIndex

    VolumesSubTab {}
    DevicesSubTab {}
    MediaSubTab {}
    VisualizerSubTab {}
  }
}
