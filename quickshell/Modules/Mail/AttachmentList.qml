import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  property var attachments: []
  property string accountId: ""
  property int messageUid: 0

  implicitHeight: attachBox.implicitHeight + Style.marginM

  CrawlBox {
    id: attachBox
    anchors.left: parent.left
    anchors.right: parent.right
    implicitHeight: attachCol.implicitHeight + Style.marginM

    ColumnLayout {
      id: attachCol
      anchors.fill: parent
      anchors.margins: Style.marginM
      spacing: Style.marginS

      CrawlText {
        text: "Attachments (" + (root.attachments.length || 0) + ")"
        pointSize: Style.fontSizeS
        font.weight: Style.fontWeightBold
        color: Theme.cOnSurface
      }

      Repeater {
        model: root.attachments

        delegate: Item {
          required property var modelData
          Layout.fillWidth: true
          Layout.preferredHeight: attRow.implicitHeight + Style.marginXS

          Rectangle {
            id: attRow
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            implicitHeight: attContent.implicitHeight + Style.marginS
            radius: Style.radiusS
            color: Qt.alpha(Theme.cPrimary, 0.05)

            RowLayout {
              id: attContent
              anchors.fill: parent
              anchors.margins: Style.marginS
              spacing: Style.marginS

              CrawlIcon {
                icon: "paperclip"
                pointSize: Style.fontSizeS
                color: Theme.cPrimary
              }

              ColumnLayout {
                Layout.fillWidth: true
                spacing: 1

                CrawlText {
                  text: modelData.filename || "attachment"
                  pointSize: Style.fontSizeXS
                  color: Theme.cOnSurface
                  elide: Text.ElideRight
                  Layout.fillWidth: true
                }

                CrawlText {
                  text: modelData.mime_type || ""
                  pointSize: Style.fontSizeXXS
                  color: Theme.cOnSurfaceVariant
                  visible: text.length > 0
                }
              }

              CrawlText {
                text: _formatSize(modelData.size)
                pointSize: Style.fontSizeXXS
                color: Theme.cOnSurfaceVariant
              }

              CrawlIconButton {
                icon: "download"
                tooltipText: "Save attachment"
                baseSize: Style.baseWidgetSize * 0.5
                visible: !modelData.cached
                onClicked: {
                  MailService.saveAttachment(root.accountId, root.messageUid, modelData.id, "", function(success) {
                    if (success) {
                      ToastService.showNotice("Mail", "Attachment saved", "mail")
                    }
                  })
                }
              }

              CrawlIcon {
                icon: "check"
                pointSize: Style.fontSizeXS
                color: Theme.cPrimary
                visible: !!modelData.cached
              }
            }
          }
        }
      }
    }
  }

  function _formatSize(bytes) {
    if (!bytes) return ""
    if (bytes < 1024) return bytes + " B"
    if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB"
    return (bytes / 1048576).toFixed(1) + " MB"
  }
}
