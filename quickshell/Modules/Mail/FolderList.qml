import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  property string accountId: ""

  signal folderSelected(string accountId, string folder)

  implicitHeight: folderCol.implicitHeight

  ColumnLayout {
    id: folderCol
    anchors.left: parent.left
    anchors.right: parent.right
    spacing: 1

    Repeater {
      model: {
        var allFolders = MailService.folders || {}
        var list = allFolders[root.accountId] || []
        // Sort: inbox first, then flagged, then by kind
        var ordered = []
        var inbox = null
        var others = []
        for (var i = 0; i < list.length; i++) {
          var f = list[i]
          if (f.kind === "inbox") inbox = f
          else others.push(f)
        }
        // Sort others: sent, drafts, archive, spam, trash, custom
        var kindOrder = { sent: 1, drafts: 2, archive: 3, spam: 4, trash: 5 }
        others.sort(function(a, b) {
          var ka = kindOrder[a.kind] || 99
          var kb = kindOrder[b.kind] || 99
          if (ka !== kb) return ka - kb
          return (a.display_name || a.name).localeCompare(b.display_name || b.name)
        })
        if (inbox) ordered.push(inbox)
        ordered = ordered.concat(others)
        return ordered
      }

      delegate: Item {
        id: folderDelegate
        required property var modelData
        Layout.fillWidth: true
        Layout.preferredHeight: folderRow.implicitHeight + Style.marginS

        Rectangle {
          id: folderRow
          anchors.left: parent.left
          anchors.right: parent.right
          anchors.verticalCenter: parent.verticalCenter
          anchors.leftMargin: Style.marginL
          anchors.rightMargin: Style.marginS
          implicitHeight: folderLabel.implicitHeight + Style.marginS
          radius: Style.radiusS
          color: folderMouse.containsMouse
            ? Qt.alpha(Theme.cPrimary, 0.06)
            : (MailService.selectedFolder === modelData.name ? Qt.alpha(Theme.cPrimary, 0.1) : "transparent")

          RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Style.marginS
            anchors.rightMargin: Style.marginS
            spacing: Style.marginS

            CrawlIcon {
              icon: root._folderIcon(modelData.kind, modelData.name)
              pointSize: Style.fontSizeS
              color: MailService.selectedFolder === modelData.name ? Theme.cPrimary : Theme.cOnSurfaceVariant
            }

            CrawlText {
              id: folderLabel
              text: modelData.display_name || modelData.name
              pointSize: Style.fontSizeXS
              color: MailService.selectedFolder === modelData.name ? Theme.cPrimary : Theme.cOnSurface
              font.weight: MailService.selectedFolder === modelData.name ? Style.fontWeightBold : Style.fontWeightNormal
              elide: Text.ElideRight
              Layout.fillWidth: true
            }

            Rectangle {
              visible: (modelData.unread || 0) > 0
              color: Theme.cPrimary
              radius: height * 0.5
              implicitWidth: badge.implicitWidth + Style.marginS
              implicitHeight: badge.implicitHeight + Style.margin2XXS

              CrawlText {
                id: badge
                anchors.centerIn: parent
                text: modelData.unread > 99 ? "99+" : String(modelData.unread || 0)
                pointSize: Style.fontSizeXXS
                color: Theme.cOnPrimary
              }
            }
          }

          MouseArea {
            id: folderMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.folderSelected(root.accountId, modelData.name)
          }
        }
      }
    }
  }

  function _folderIcon(kind, name) {
    var lower = (kind || "").toLowerCase()
    if (lower === "inbox") return "inbox"
    if (lower === "sent") return "send"
    if (lower === "drafts") return "file-text"
    if (lower === "trash") return "trash"
    if (lower === "spam") return "alert-octagon"
    if (lower === "archive") return "archive"
    return "folder"
  }
}
