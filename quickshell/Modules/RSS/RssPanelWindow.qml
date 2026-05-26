import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import qs.Common
import qs.Modules.RSS
import qs.Services
import qs.Widgets

FloatingWindow {
  id: root

  title: "RSS Feeds"
  minimumSize: Qt.size(Math.round(900 * Style.uiScaleRatio), Math.round(600 * Style.uiScaleRatio))
  implicitWidth: Math.round(900 * Style.uiScaleRatio)
  implicitHeight: Math.round(600 * Style.uiScaleRatio)
  color: "transparent"

  visible: false

  property string activeView: "list"
  property bool showAddFeedDialog: false

  Component.onCompleted: {
    RssService.rssWindow = root
    RssService.listFeeds()
    RssService.listCategories()
    RssService.listEntries()
  }

  Shortcut {
    sequence: "Escape"
    onActivated: RssService.closeWindow()
  }

  onVisibleChanged: {
    if (visible) {
      RssService.isWindowOpen = true
    } else {
      RssService.isWindowOpen = false
    }
  }

  Rectangle {
    anchors.fill: parent
    color: Theme.cSurface
    radius: Style.radiusL
    clip: true

    RowLayout {
      anchors {
        fill: parent
        margins: Style.marginM
      }
      spacing: 0

      FeedList {
        Layout.preferredWidth: Math.round(220 * Style.uiScaleRatio)
        Layout.fillHeight: true
        onFeedSelected: feedId => { RssService.selectFeed(feedId); root.activeView = "list" }
        onCategorySelected: category => { RssService.selectCategory(category); root.activeView = "list" }
        onAddFeedRequested: root.showAddFeedDialog = true
      }

      Rectangle {
        width: 1; Layout.fillHeight: true; color: Theme.cOutline; opacity: 0.3
      }

      Item {
        Layout.fillWidth: true; Layout.fillHeight: true

        StackLayout {
          anchors.fill: parent
          currentIndex: root.activeView === "list" ? 0 : 1

          ColumnLayout {
            spacing: 0
            CrawlBox {
              Layout.fillWidth: true; Layout.preferredHeight: header.implicitHeight + Style.margin2M
              RowLayout {
                id: header; anchors.fill: parent; anchors.margins: Style.marginM; spacing: Style.marginM
                CrawlIcon { icon: "rss"; pointSize: Style.fontSizeL; color: Theme.cPrimary }
                CrawlLabel {
                  label: { if (RssService.selectedFeedId) { var f = RssService.getFeedById(RssService.selectedFeedId); return f ? f.title : "Feed" }; if (RssService.selectedCategory) return RssService.selectedCategory; return "All Entries" }
                  Layout.fillWidth: true
                }
                CrawlToggle { checked: RssService.onlyUnread; baseSize: Style.baseWidgetSize * 0.7; onToggled: c => { RssService.onlyUnread = c; RssService.listEntries() } }
                CrawlIconButton { icon: "star"; tooltipText: "Starred only"; baseSize: Style.baseWidgetSize * 0.7; colorBg: RssService.onlyStarred ? Theme.cPrimary : "transparent"; colorFg: RssService.onlyStarred ? Theme.cOnPrimary : Theme.cOnSurfaceVariant; onClicked: { RssService.onlyStarred = !RssService.onlyStarred; RssService.listEntries() } }
                CrawlIconButton { icon: "refresh"; tooltipText: "Refresh"; baseSize: Style.baseWidgetSize * 0.7; enabled: !RssService.refreshing; onClicked: { RssService.selectedFeedId ? RssService.refreshFeed(RssService.selectedFeedId) : RssService.refreshAll(); Qt.callLater(function() { RssService.listEntries() }) } }
                CrawlIconButton { icon: "close"; tooltipText: "Close"; baseSize: Style.baseWidgetSize * 0.7; onClicked: RssService.closeWindow() }
              }
            }
            Rectangle {
              visible: RssService.refreshing
              Layout.fillWidth: true
              Layout.preferredHeight: 3
              color: Theme.cPrimary
              SequentialAnimation on opacity {
                loops: Animation.Infinite
                PropertyAnimation { from: 1; to: 0.3; duration: 800 }
                PropertyAnimation { from: 0.3; to: 1; duration: 800 }
              }
            }
            EntryList {
              Layout.fillWidth: true
              Layout.fillHeight: true
              Layout.leftMargin: Style.marginS
              Layout.rightMargin: Style.marginS
              Layout.bottomMargin: Style.marginS
              onEntrySelected: entryId => {
                RssService.selectEntry(entryId)
                root.activeView = "entry"
              }
            }
          }

          EntryView {
            onBackRequested: { root.activeView = "list"; RssService.listEntries() }
          }
        }
      }

      AddFeedDialog {
        visible: root.showAddFeedDialog
        onClosed: root.showAddFeedDialog = false
      }
    }
  }
}
