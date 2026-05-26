import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  required property var messageData
  property bool selected: false

  signal clicked()
  signal doubleClicked()

  implicitHeight: itemBox.implicitHeight + Style.marginXS

  CrawlBox {
    id: itemBox
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.verticalCenter: parent.verticalCenter
    implicitHeight: itemColumn.implicitHeight + Style.marginM
    color: root.selected
      ? Qt.alpha(Theme.cPrimary, 0.15)
      : (itemMouse.containsMouse ? Qt.alpha(Theme.cPrimary, 0.05) : "transparent")
    radius: Style.radiusM
    border.width: root.selected ? Style.borderS : 0
    border.color: root.selected ? Theme.cPrimary : "transparent"

    ColumnLayout {
      id: itemColumn
      anchors.fill: parent
      anchors.margins: Style.marginS
      spacing: 2

      // ── Row 1: From + Date + Flags ────────────────────────────────────
      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginS

        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginXS

          CrawlIcon {
            icon: "mail-open"
            pointSize: Style.fontSizeXS
            color: _hasFlag("seen") ? Theme.cOnSurfaceVariant : Theme.cPrimary
            visible: !_hasFlag("seen")
          }

          CrawlIcon {
            icon: "star"
            pointSize: Style.fontSizeXS
            color: _hasFlag("flagged") ? Theme.warning : Theme.cOnSurfaceVariant
            opacity: _hasFlag("flagged") ? 1 : 0.3
          }

          CrawlText {
            text: root.messageData.from || "Unknown"
            pointSize: Style.fontSizeS
            font.weight: _hasFlag("seen") ? Style.fontWeightNormal : Style.fontWeightBold
            color: Theme.cOnSurface
            elide: Text.ElideRight
            Layout.fillWidth: true
          }
        }

        CrawlText {
          text: root._formatDate(root.messageData.date)
          pointSize: Style.fontSizeXS
          color: Theme.cOnSurfaceVariant
          opacity: 0.7
        }
      }

      // ── Row 2: Subject ────────────────────────────────────────────────
      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginS

        CrawlText {
          text: root.messageData.subject || "(no subject)"
          pointSize: Style.fontSizeXS
          font.weight: _hasFlag("seen") ? Style.fontWeightNormal : Style.fontWeightBold
          color: _hasFlag("seen") ? Theme.cOnSurfaceVariant : Theme.cOnSurface
          elide: Text.ElideRight
          Layout.fillWidth: true
        }

        RowLayout {
          spacing: 2
          visible: root.messageData.has_attachments

          CrawlIcon {
            icon: "paperclip"
            pointSize: Style.fontSizeXXS
            color: Theme.cOnSurfaceVariant
          }
        }
      }

      // ── Row 3: Snippet ────────────────────────────────────────────────
      CrawlText {
        text: root.messageData.snippet || ""
        pointSize: Style.fontSizeXXS
        color: Theme.cOnSurfaceVariant
        elide: Text.ElideRight
        Layout.fillWidth: true
        visible: text.length > 0 && _hasFlag("seen")
        maximumLineCount: 1
      }
    }

    MouseArea {
      id: itemMouse
      anchors.fill: parent
      hoverEnabled: true
      cursorShape: Qt.PointingHandCursor
      onClicked: root.clicked()
      onDoubleClicked: root.doubleClicked()
    }
  }

  function _hasFlag(flag) {
    var flags = root.messageData.flags || []
    for (var i = 0; i < flags.length; i++) {
      if (flags[i] === flag) return true
    }
    return false
  }

  function _formatDate(dateStr) {
    if (!dateStr) return ""
    try {
      var d = new Date(dateStr)
      if (isNaN(d.getTime())) return dateStr.substring(0, 10)
      var now = new Date()
      var diff = now - d
      var dayMs = 86400000
      if (diff < dayMs) {
        return d.toLocaleTimeString(Qt.locale(), "hh:mm")
      } else if (diff < 7 * dayMs) {
        return d.toLocaleDateString(Qt.locale(), "ddd")
      } else {
        return d.toLocaleDateString(Qt.locale(), "dd/MM")
      }
    } catch (e) {
      return dateStr.substring(0, 10)
    }
  }
}
