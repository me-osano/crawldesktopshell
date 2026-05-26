import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  required property var entryData
  property bool selected: false

  signal clicked()

  implicitHeight: box.implicitHeight + Style.marginXS

  CrawlBox {
    id: box
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.verticalCenter: parent.verticalCenter
    implicitHeight: col.implicitHeight + Style.marginM
    color: root.selected
      ? Qt.alpha(Theme.cPrimary, 0.15)
      : (area.containsMouse ? Qt.alpha(Theme.cPrimary, 0.05) : "transparent")
    radius: Style.radiusM
    border.width: root.selected ? Style.borderS : 0
    border.color: root.selected ? Theme.cPrimary : "transparent"
    opacity: entryData.is_read ? 0.65 : 1.0

    ColumnLayout {
      id: col
      anchors.fill: parent
      anchors.margins: Style.marginS
      spacing: 2

      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginXS

        CrawlIcon {
          icon: "star"
          pointSize: Style.fontSizeXS
          color: entryData.is_starred ? Theme.warning : "transparent"
          visible: entryData.is_starred
        }

        CrawlText {
          text: entryData.title || "(no title)"
          pointSize: Style.fontSizeS
          font.weight: entryData.is_read ? Style.fontWeightNormal : Style.fontWeightBold
          color: entryData.is_read ? Theme.cOnSurfaceVariant : Theme.cOnSurface
          elide: Text.ElideRight
          Layout.fillWidth: true
          maximumLineCount: 1
        }

        CrawlText {
          text: _formatDate(entryData.published)
          pointSize: Style.fontSizeXXS
          color: Theme.cOnSurfaceVariant
          opacity: 0.7
        }
      }

      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginXS

        CrawlText {
          text: entryData.feed_title || ""
          pointSize: Style.fontSizeXXS
          color: Theme.cPrimary
          opacity: 0.7
          visible: text.length > 0
        }

        CrawlText {
          text: entryData.author || ""
          pointSize: Style.fontSizeXXS
          color: Theme.cOnSurfaceVariant
          visible: text.length > 0
        }

        Item { Layout.fillWidth: true }

        CrawlIcon {
          icon: "link"
          pointSize: Style.fontSizeXXS
          color: Theme.cOnSurfaceVariant
          visible: entryData.image_url && entryData.image_url.length > 0
        }
      }

      CrawlText {
        text: entryData.summary || ""
        pointSize: Style.fontSizeXXS
        color: Theme.cOnSurfaceVariant
        elide: Text.ElideRight
        Layout.fillWidth: true
        visible: text.length > 0
        maximumLineCount: 1
      }
    }

    MouseArea {
      id: area
      anchors.fill: parent
      hoverEnabled: true
      cursorShape: Qt.PointingHandCursor
      onClicked: root.clicked()
    }
  }

  function _formatDate(dateStr) {
    if (!dateStr) return ""
    try {
      var d = new Date(dateStr)
      if (isNaN(d.getTime())) return dateStr.substring(0, 10)
      var now = new Date()
      var diff = now - d
      if (diff < 86400000) {
        return d.toLocaleTimeString(Qt.locale(), "hh:mm")
      } else if (diff < 604800000) {
        return d.toLocaleDateString(Qt.locale(), "ddd")
      } else {
        return d.toLocaleDateString(Qt.locale(), "dd/MM")
      }
    } catch (e) { return dateStr.substring(0, 10) }
  }
}
