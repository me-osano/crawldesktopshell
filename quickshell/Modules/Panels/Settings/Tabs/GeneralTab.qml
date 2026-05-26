import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import "../../../../Common/Helpers/QtObj2JS.js" as QtObj2JS
import "General"
import qs.Common
import qs.Services
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
      text: "Basics"
      tabIndex: 0
      checked: subTabBar.currentIndex === 0
    }
    CrawlTabButton {
      text: "Keybinds"
      tabIndex: 1
      checked: subTabBar.currentIndex === 1
    }
  }

  Item {
    Layout.fillWidth: true
    Layout.preferredHeight: Style.marginL
  }

  CrawlTabView {
    id: tabView
    currentIndex: subTabBar.currentIndex
    BasicsSubTab {}
    KeybindsSubTab {}
  }
}
