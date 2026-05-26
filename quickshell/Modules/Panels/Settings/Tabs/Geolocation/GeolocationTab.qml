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
      text: "Location"
      tabIndex: 0
      checked: subTabBar.currentIndex === 0
    }
    CrawlTabButton {
      text: "Date"
      tabIndex: 1
      checked: subTabBar.currentIndex === 1
    }
    CrawlTabButton {
      text: "Calendar Panel"
      tabIndex: 2
      checked: subTabBar.currentIndex === 2
    }
  }

  Item {
    Layout.fillWidth: true
    Layout.preferredHeight: Style.marginL
  }

  CrawlTabView {
    id: tabView
    currentIndex: subTabBar.currentIndex

    LocationSubTab {}
    DateSubTab {}
    ClockPanelSubTab {}
  }
}
