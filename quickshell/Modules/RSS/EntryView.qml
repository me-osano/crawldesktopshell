import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  signal backRequested()

  ColumnLayout {
    anchors.fill: parent
    spacing: 0

    // ── Header bar ───────────────────────────────────────────────────────
    CrawlBox {
      Layout.fillWidth: true
      Layout.preferredHeight: header.implicitHeight + Style.margin2M

      RowLayout {
        id: header
        anchors.fill: parent
        anchors.margins: Style.marginM
        spacing: Style.marginM

        CrawlIconButton {
          icon: "arrow-left"
          tooltipText: "Back to list"
          baseSize: Style.baseWidgetSize * 0.7
          onClicked: root.backRequested()
        }

        CrawlIcon {
          icon: "rss"
          pointSize: Style.fontSizeL
          color: Theme.cPrimary
        }

        CrawlText {
          text: RssService.currentEntry ? RssService.currentEntry.title : "Entry"
          pointSize: Style.fontSizeL
          font.weight: Style.fontWeightBold
          color: Theme.cOnSurface
          Layout.fillWidth: true
          elide: Text.ElideRight
        }

        CrawlIconButton {
          icon: RssService.currentEntry && RssService.currentEntry.is_starred ? "star" : "star-off"
          tooltipText: RssService.currentEntry && RssService.currentEntry.is_starred ? "Unstar" : "Star"
          baseSize: Style.baseWidgetSize * 0.7
          colorFg: RssService.currentEntry && RssService.currentEntry.is_starred ? Theme.warning : Theme.cOnSurfaceVariant
          enabled: !!RssService.currentEntry
          onClicked: {
            if (RssService.currentEntry) {
              RssService.toggleStarred(RssService.currentEntry.id, RssService.currentEntry.is_starred)
            }
          }
        }

        CrawlIconButton {
          icon: "external-link"
          tooltipText: "Open in browser"
          baseSize: Style.baseWidgetSize * 0.7
          enabled: RssService.currentEntry && RssService.currentEntry.url && RssService.currentEntry.url.length > 0
          onClicked: {
            if (RssService.currentEntry) {
              RssService.openInBrowser(RssService.currentEntry.url)
            }
          }
        }

        CrawlIconButton {
          icon: "close"
          tooltipText: "Close"
          baseSize: Style.baseWidgetSize * 0.7
          onClicked: root.backRequested()
        }
      }
    }

    // ── Entry content ────────────────────────────────────────────────────
    CrawlScrollView {
      Layout.fillWidth: true
      Layout.fillHeight: true
      horizontalPolicy: ScrollBar.AsNeeded
      verticalPolicy: ScrollBar.AsNeeded
      gradientColor: Theme.cSurface

      ColumnLayout {
        width: Math.max(parent.width - Style.marginL - Style.marginL, 200)
        x: Style.marginM
        spacing: Style.marginM

        // Loading state
        Rectangle {
          visible: !RssService.currentEntry
          Layout.fillWidth: true
          Layout.preferredHeight: 200
          color: "transparent"

          ColumnLayout {
            anchors.centerIn: parent

            CrawlBusyIndicator {
              running: true
              color: Theme.cPrimary
              size: Style.baseWidgetSize
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlText {
              text: "Loading entry..."
              pointSize: Style.fontSizeS
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }
          }
        }

        // Entry header
        CrawlBox {
          visible: !!RssService.currentEntry
          Layout.fillWidth: true
          implicitHeight: metaCol.implicitHeight + Style.margin2M

          ColumnLayout {
            id: metaCol
            anchors.fill: parent
            anchors.margins: Style.marginM
            spacing: Style.marginS

            CrawlText {
              text: RssService.currentEntry ? RssService.currentEntry.title || "(no title)" : ""
              pointSize: Style.fontSizeXL
              font.weight: Style.fontWeightBold
              color: Theme.cOnSurface
              wrapMode: Text.Wrap
              Layout.fillWidth: true
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: Theme.cOutline; opacity: 0.2 }

            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginM

              ColumnLayout {
                spacing: Style.marginXS
                Layout.fillWidth: true

                RowLayout {
                  spacing: Style.marginS
                  visible: RssService.currentEntry && RssService.currentEntry.feed_title

                  CrawlText {
                    text: "Feed:"
                    pointSize: Style.fontSizeXS
                    color: Theme.cOnSurfaceVariant
                    font.weight: Style.fontWeightBold
                  }
                  CrawlText {
                    text: RssService.currentEntry ? RssService.currentEntry.feed_title : ""
                    pointSize: Style.fontSizeXS
                    color: Theme.cPrimary
                  }
                }

                RowLayout {
                  spacing: Style.marginS
                  visible: RssService.currentEntry && RssService.currentEntry.author

                  CrawlText {
                    text: "Author:"
                    pointSize: Style.fontSizeXS
                    color: Theme.cOnSurfaceVariant
                    font.weight: Style.fontWeightBold
                  }
                  CrawlText {
                    text: RssService.currentEntry ? RssService.currentEntry.author : ""
                    pointSize: Style.fontSizeXS
                    color: Theme.cOnSurface
                  }
                }
              }

              CrawlText {
                text: RssService.currentEntry ? _formatFullDate(RssService.currentEntry.published) : ""
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurfaceVariant
              }
            }

            RowLayout {
              visible: RssService.currentEntry && RssService.currentEntry.url && RssService.currentEntry.url.length > 0

              CrawlText {
                text: "Link:"
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurfaceVariant
                font.weight: Style.fontWeightBold
              }
              CrawlText {
                text: RssService.currentEntry ? RssService.currentEntry.url : ""
                pointSize: Style.fontSizeXS
                color: Theme.cPrimary
                elide: Text.ElideRight
                Layout.fillWidth: true

                MouseArea {
                  anchors.fill: parent
                  cursorShape: Qt.PointingHandCursor
                  hoverEnabled: true
                  onClicked: {
                    if (RssService.currentEntry) {
                      RssService.openInBrowser(RssService.currentEntry.url)
                    }
                  }
                }
              }
            }
          }
        }

        // ── Lead image ──────────────────────────────────────────────────
        Rectangle {
          visible: RssService.currentEntry && RssService.currentEntry.image_url && RssService.currentEntry.image_url.length > 0
          Layout.fillWidth: true
          Layout.preferredHeight: 200
          color: Theme.cSurfaceVariant
          radius: Style.radiusM
          clip: true

          Image {
            anchors.fill: parent
            source: RssService.currentEntry ? RssService.currentEntry.image_url : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
          }
        }

        // ── Summary / excerpt ───────────────────────────────────────────
        CrawlBox {
          visible: RssService.currentEntry && RssService.currentEntry.summary && RssService.currentEntry.summary.length > 0
          Layout.fillWidth: true
          implicitHeight: summaryText.implicitHeight + Style.margin2M

          CrawlText {
            id: summaryText
            anchors.fill: parent
            anchors.margins: Style.marginM
            text: RssService.currentEntry ? RssService.currentEntry.summary : ""
            pointSize: Style.fontSizeS
            color: Theme.cOnSurfaceVariant
            wrapMode: Text.Wrap
            font.italic: true
          }
        }

        // ── Full content ────────────────────────────────────────────────
        CrawlBox {
          visible: !!RssService.currentEntry
          Layout.fillWidth: true
          implicitHeight: contentFlickable.implicitHeight + Style.margin2L

          Flickable {
            id: contentFlickable
            anchors.fill: parent
            anchors.margins: Style.marginM
            contentHeight: contentText.implicitHeight
            clip: true
            interactive: false

            Text {
              id: contentText
              width: parent.width
              text: RssService.currentEntry ? (RssService.currentEntry.content || RssService.currentEntry.summary || "(No content)") : ""
              font.pointSize: Style.fontSizeS
              color: Theme.cOnSurface
              wrapMode: Text.Wrap
              textFormat: Text.RichText
              onLinkActivated: link => Qt.openUrlExternally(link)
            }
          }
        }

        // ── Open in browser button ──────────────────────────────────────
        RowLayout {
          Layout.fillWidth: true
          Layout.bottomMargin: Style.marginM
          spacing: Style.marginM

          Item { Layout.fillWidth: true }

          CrawlButton {
            text: "Open in browser"
            icon: "external-link"
            enabled: RssService.currentEntry && RssService.currentEntry.url && RssService.currentEntry.url.length > 0
            onClicked: {
              if (RssService.currentEntry) {
                RssService.openInBrowser(RssService.currentEntry.url)
              }
            }
          }
        }
      }
    }
  }

  function _formatFullDate(dateStr) {
    if (!dateStr) return ""
    try {
      var d = new Date(dateStr)
      if (isNaN(d.getTime())) return dateStr
      return d.toLocaleString(Qt.locale(), "yyyy-MM-dd hh:mm")
    } catch (e) { return dateStr }
  }
}
