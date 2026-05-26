import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  signal closed()

  anchors.fill: parent
  visible: false

  Rectangle {
    anchors.fill: parent
    color: Qt.alpha(Theme.cSurface, 0.92)
    z: 20

    ColumnLayout {
      anchors.centerIn: parent
      width: parent.width * 0.5
      spacing: Style.marginL

      CrawlIcon {
        icon: "rss"
        pointSize: 36
        color: Theme.cPrimary
        Layout.alignment: Qt.AlignHCenter
      }

      CrawlLabel {
        label: "Subscribe to Feed"
        Layout.alignment: Qt.AlignHCenter
      }

      CrawlBox {
        Layout.fillWidth: true
        implicitHeight: formCol.implicitHeight + Style.margin2M

        ColumnLayout {
          id: formCol
          anchors.fill: parent
          anchors.margins: Style.marginM
          spacing: Style.marginM

          ColumnLayout {
            Layout.fillWidth: true
            spacing: Style.marginXS

            CrawlText { text: "Feed URL"; pointSize: Style.fontSizeXS; color: Theme.cOnSurfaceVariant }
            TextField {
              id: _url
              Layout.fillWidth: true
              placeholderText: "https://example.com/feed.xml"
              color: Theme.cOnSurface
              background: Rectangle {
                color: Theme.cSurfaceVariant
                radius: Style.radiusS
                border.width: Style.borderS
                border.color: Theme.cOutline
              }
            }
          }

          ColumnLayout {
            Layout.fillWidth: true
            spacing: Style.marginXS

            CrawlText { text: "Category (optional)"; pointSize: Style.fontSizeXS; color: Theme.cOnSurfaceVariant }
            TextField {
              id: _category
              Layout.fillWidth: true
              placeholderText: "News, Tech, Blogs..."
              color: Theme.cOnSurface
              background: Rectangle {
                color: Theme.cSurfaceVariant
                radius: Style.radiusS
                border.width: Style.borderS
                border.color: Theme.cOutline
              }
            }
          }

          // Error message
          Rectangle {
            visible: _errorText.text.length > 0
            Layout.fillWidth: true
            Layout.preferredHeight: _errorText.implicitHeight + Style.marginM
            color: Qt.alpha(Theme.cError, 0.1)
            radius: Style.radiusS

            CrawlText {
              id: _errorText
              anchors.fill: parent
              anchors.margins: Style.marginS
              pointSize: Style.fontSizeXS
              color: Theme.cError
              wrapMode: Text.Wrap
            }
          }
        }
      }

      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginM

        // OPML Import hint
        CrawlText {
          text: "Tip: You can also import an OPML file"
          pointSize: Style.fontSizeXXS
          color: Theme.cOnSurfaceVariant
          Layout.fillWidth: true
        }

        CrawlButton {
          text: "Cancel"
          onClicked: {
            _url.text = ""
            _category.text = ""
            _errorText.text = ""
            root.closed()
          }
        }

        CrawlButton {
          text: "Add Feed"
          icon: "plus"
          enabled: _url.text.trim().length > 0
          onClicked: {
            var url = _url.text.trim()
            var cat = _category.text.trim()
            RssService.addFeed(url, cat, function(result, error) {
              if (error) {
                _errorText.text = error.message || "Failed to add feed"
              } else {
                _url.text = ""
                _category.text = ""
                _errorText.text = ""
                root.closed()
              }
            })
          }
        }
      }
    }
  }

  MouseArea {
    anchors.fill: parent
    z: -1
    onClicked: {
      _url.text = ""
      _category.text = ""
      _errorText.text = ""
      root.closed()
    }
  }
}
