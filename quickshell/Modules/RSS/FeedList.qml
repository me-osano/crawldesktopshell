import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  signal feedSelected(string feedId)
  signal categorySelected(string category)
  signal addFeedRequested()

  ColumnLayout {
    anchors.fill: parent
    spacing: 0

    // ── Header ──────────────────────────────────────────────────────────
    CrawlBox {
      Layout.fillWidth: true
      Layout.preferredHeight: header.implicitHeight + Style.margin2M

      RowLayout {
        id: header
        anchors.fill: parent
        anchors.margins: Style.marginM
        spacing: Style.marginS

        CrawlIcon {
          icon: "rss"
          pointSize: Style.fontSizeM
          color: Theme.cPrimary
        }

        CrawlLabel {
          label: "Feeds"
          Layout.fillWidth: true
        }

        CrawlIconButton {
          icon: "plus"
          tooltipText: "Add feed"
          baseSize: Style.baseWidgetSize * 0.7
          onClicked: root.addFeedRequested()
        }
      }
    }

    // ── Scrollable content ──────────────────────────────────────────────
    CrawlScrollView {
      Layout.fillWidth: true
      Layout.fillHeight: true
      horizontalPolicy: ScrollBar.AlwaysOff
      verticalPolicy: ScrollBar.AsNeeded
      gradientColor: Theme.cSurface

      ColumnLayout {
        width: parent.width
        spacing: 0

        // ── All Entries ─────────────────────────────────────────────────
        FeedItem {
          feedData: ({ id: "__all__", title: "All Entries", icon_url: "", unread_count: RssService.totalUnreadCount(), category: "" })
          isSelected: !RssService.selectedFeedId && !RssService.selectedCategory
          isAllEntries: true
          onClicked: {
            RssService.selectedFeedId = ""
            RssService.selectedCategory = ""
            root.feedSelected("")
          }
        }

        // ── Category groups ─────────────────────────────────────────────
        Repeater {
          model: RssService.categories || []

          delegate: ColumnLayout {
            required property var modelData
            Layout.fillWidth: true
            spacing: 0

            FeedItem {
              feedData: ({ id: "__cat__" + modelData, title: modelData, icon_url: "", unread_count: _catUnread(modelData), category: modelData })
              isSelected: RssService.selectedCategory === modelData
              isCategory: true
              onClicked: root.categorySelected(modelData)
            }

            // Feeds in this category
            Repeater {
              model: _feedsInCategory(modelData)

              delegate: FeedItem {
                required property var modelData
                Layout.leftMargin: Style.marginL
                feedData: modelData
                isSelected: RssService.selectedFeedId === modelData.id
                onClicked: root.feedSelected(modelData.id)
              }
            }
          }
        }

        // ── Uncategorized feeds ─────────────────────────────────────────
        Repeater {
          model: _uncategorizedFeeds()

          delegate: FeedItem {
            required property var modelData
            feedData: modelData
            isSelected: RssService.selectedFeedId === modelData.id
            onClicked: root.feedSelected(modelData.id)
          }
        }

        // ── Empty state ────────────────────────────────────────────────
        CrawlBox {
          visible: !RssService.feeds || RssService.feeds.length === 0
          Layout.fillWidth: true
          Layout.preferredHeight: emptyCol.implicitHeight + Style.margin2L
          Layout.topMargin: Style.marginL

          ColumnLayout {
            id: emptyCol
            anchors.centerIn: parent
            spacing: Style.marginM

            CrawlIcon {
              icon: "rss"
              pointSize: 36
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlText {
              text: "No feeds"
              pointSize: Style.fontSizeM
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlButton {
              text: "Add feed"
              icon: "plus"
              Layout.alignment: Qt.AlignHCenter
              onClicked: root.addFeedRequested()
            }
          }
        }
      }
    }
  }

  function _catUnread(category) {
    var list = RssService.feeds || []
    var sum = 0
    for (var i = 0; i < list.length; i++) {
      if (list[i].category === category) sum += (list[i].unread_count || 0)
    }
    return sum
  }

  function _feedsInCategory(category) {
    var list = RssService.feeds || []
    var result = []
    for (var i = 0; i < list.length; i++) {
      if (list[i].category === category) result.push(list[i])
    }
    return result
  }

  function _uncategorizedFeeds() {
    var list = RssService.feeds || []
    var result = []
    for (var i = 0; i < list.length; i++) {
      if (!list[i].category || list[i].category === "") result.push(list[i])
    }
    return result
  }
}
