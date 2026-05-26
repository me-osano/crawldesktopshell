import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  signal entrySelected(string entryId)

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.marginS

    // ── Count ────────────────────────────────────────────────────────────
    CrawlText {
      text: RssService.totalEntries > 0 ? RssService.totalEntries + " entries" : ""
      pointSize: Style.fontSizeXXS
      color: Theme.cOnSurfaceVariant
      visible: text.length > 0
    }

    // ── Entry list ──────────────────────────────────────────────────────
    CrawlScrollView {
      Layout.fillWidth: true
      Layout.fillHeight: true
      horizontalPolicy: ScrollBar.AlwaysOff
      verticalPolicy: ScrollBar.AsNeeded
      gradientColor: Theme.cSurface

      ColumnLayout {
        width: parent.width
        spacing: 0

        // Loading state
        Rectangle {
          visible: RssService.loadingEntries
          Layout.fillWidth: true
          Layout.preferredHeight: 100

          ColumnLayout {
            anchors.centerIn: parent
            spacing: Style.marginM

            CrawlBusyIndicator {
              running: true
              color: Theme.cPrimary
              size: Style.baseWidgetSize
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlText {
              text: "Loading entries..."
              pointSize: Style.fontSizeS
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }
          }
        }

        // Empty state
        Rectangle {
          visible: !RssService.loadingEntries && (!RssService.entries || RssService.entries.length === 0)
          Layout.fillWidth: true
          Layout.preferredHeight: 100

          ColumnLayout {
            anchors.centerIn: parent
            spacing: Style.marginM

            CrawlIcon {
              icon: RssService.onlyUnread ? "check-circle" : "rss"
              pointSize: 36
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlText {
              text: RssService.onlyUnread ? "No unread entries" : "No entries"
              pointSize: Style.fontSizeS
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }
          }
        }

        // Entry items
        Repeater {
          model: RssService.entries || []

          delegate: EntryItem {
            required property var modelData
            width: parent.width
            entryData: modelData

            onClicked: root.entrySelected(modelData.id)
          }
        }
      }
    }
  }
}
