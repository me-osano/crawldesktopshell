import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import qs.Common
import qs.Modules.MainScreen
import qs.Modules.RSS
import qs.Services
import qs.Widgets

SmartPanel {
  id: root

  preferredWidth: Math.round(900 * Style.uiScaleRatio)
  preferredHeight: Math.round(600 * Style.uiScaleRatio)

  readonly property string panelMode: Settings.data.ui.rssPanelMode
  readonly property bool attachToBar: panelMode === "attached"
  readonly property bool isWindowMode: panelMode === "window"

  forceAttachToBar: attachToBar
  panelAnchorHorizontalCenter: !attachToBar
  panelAnchorVerticalCenter: !attachToBar

  // ── State ──────────────────────────────────────────────────────────────
  property string activeView: "list" // "list" | "entry"
  property bool showAddFeedDialog: false

  // ── Window mode ───────────────────────────────────────────────────────
  function toggle(buttonItem, buttonName) {
    if (isWindowMode) {
      RssService.toggleWindow()
      return
    }
    if (isPanelOpen) {
      close()
    } else if (!attachToBar) {
      open(null, null)
    } else {
      open(buttonItem, buttonName)
    }
  }

  // ── Panel lifecycle ────────────────────────────────────────────────────
  onOpened: {
    RssService.listFeeds()
    RssService.listCategories()
    RssService.listEntries()
  }

  panelContent: Rectangle {
    color: "transparent"
    property bool allowAttach: root.attachToBar

    RowLayout {
      anchors {
        fill: parent
        margins: Style.marginM
      }
      spacing: 0

      // ── Left pane: Feeds + Categories ──────────────────────────────────
      FeedList {
        id: feedList
        Layout.preferredWidth: Math.round(220 * Style.uiScaleRatio)
        Layout.fillHeight: true

        onFeedSelected: feedId => {
          RssService.selectFeed(feedId)
          root.activeView = "list"
        }
        onCategorySelected: category => {
          RssService.selectCategory(category)
          root.activeView = "list"
        }
        onAddFeedRequested: root.showAddFeedDialog = true
      }

      // ── Divider ────────────────────────────────────────────────────────
      Rectangle {
        width: 1
        Layout.fillHeight: true
        color: Theme.cOutline
        opacity: 0.3
      }

      // ── Center + Right panes ───────────────────────────────────────────
      Item {
        Layout.fillWidth: true
        Layout.fillHeight: true

        StackLayout {
          anchors.fill: parent
          currentIndex: root.activeView === "list" ? 0 : 1

          // ── Entry list ──────────────────────────────────────────────────
          ColumnLayout {
            spacing: 0

            // Header
            CrawlBox {
              Layout.fillWidth: true
              Layout.preferredHeight: header.implicitHeight + Style.margin2M

              RowLayout {
                id: header
                anchors.fill: parent
                anchors.margins: Style.marginM
                spacing: Style.marginM

                CrawlIcon {
                  icon: "rss"
                  pointSize: Style.fontSizeL
                  color: Theme.cPrimary
                }

                CrawlLabel {
                  label: {
                    if (RssService.selectedFeedId) {
                      var feed = RssService.getFeedById(RssService.selectedFeedId)
                      return feed ? feed.title : "Feed"
                    }
                    if (RssService.selectedCategory) return RssService.selectedCategory
                    return "All Entries"
                  }
                  Layout.fillWidth: true
                }

                CrawlToggle {
                  id: unreadToggle
                  checked: RssService.onlyUnread
                  baseSize: Style.baseWidgetSize * 0.7
                  onToggled: checked => {
                    RssService.onlyUnread = checked
                    RssService.listEntries()
                  }
                }

                CrawlIconButton {
                  icon: "star"
                  tooltipText: "Starred only"
                  baseSize: Style.baseWidgetSize * 0.7
                  colorBg: RssService.onlyStarred ? Theme.cPrimary : "transparent"
                  colorFg: RssService.onlyStarred ? Theme.cOnPrimary : Theme.cOnSurfaceVariant
                  onClicked: {
                    RssService.onlyStarred = !RssService.onlyStarred
                    RssService.listEntries()
                  }
                }

                CrawlIconButton {
                  icon: "refresh"
                  tooltipText: "Refresh"
                  baseSize: Style.baseWidgetSize * 0.7
                  enabled: !RssService.refreshing
                  onClicked: {
                    if (RssService.selectedFeedId) {
                      RssService.refreshFeed(RssService.selectedFeedId)
                    } else {
                      RssService.refreshAll()
                    }
                    Qt.callLater(function () {
                      RssService.listEntries()
                    })
                  }
                }

                CrawlIconButton {
                  icon: "close"
                  tooltipText: "Close"
                  baseSize: Style.baseWidgetSize * 0.7
                  onClicked: root.close()
                }
              }
            }

            // Refreshing indicator
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

            // Entry list
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

          // ── Entry view ──────────────────────────────────────────────────
          EntryView {
            id: entryView
            onBackRequested: {
              root.activeView = "list"
              RssService.listEntries()
            }
          }
        }
      }
    }

    // ── Add feed dialog ──────────────────────────────────────────────────
    AddFeedDialog {
      visible: root.showAddFeedDialog
      onClosed: root.showAddFeedDialog = false
    }
  }
}
