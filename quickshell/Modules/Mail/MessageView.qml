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
  signal replyRequested(string messageId, string fromAddr, string subject)
  signal forwardRequested(string messageId, string subject)

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
          icon: "mail"
          pointSize: Style.fontSizeL
          color: Theme.cPrimary
        }

        CrawlText {
          text: MailService.currentMessage ? MailService.currentMessage.subject : "Message"
          pointSize: Style.fontSizeL
          font.weight: Style.fontWeightBold
          color: Theme.cOnSurface
          Layout.fillWidth: true
          elide: Text.ElideRight
        }

        CrawlIconButton {
          icon: "message-reply"
          tooltipText: "Reply"
          baseSize: Style.baseWidgetSize * 0.7
          enabled: !!MailService.currentMessage
          onClicked: {
            var msg = MailService.currentMessage
            root.replyRequested(msg.message_id || "", msg.from || "", msg.subject || "")
          }
        }

        CrawlIconButton {
          icon: "mail-forward"
          tooltipText: "Forward"
          baseSize: Style.baseWidgetSize * 0.7
          enabled: !!MailService.currentMessage
          onClicked: {
            var msg = MailService.currentMessage
            root.forwardRequested(msg.message_id || "", msg.subject || "")
          }
        }

        CrawlIconButton {
          icon: "trash"
          tooltipText: "Delete"
          baseSize: Style.baseWidgetSize * 0.7
          enabled: !!MailService.currentMessage
          onClicked: {
            var msg = MailService.currentMessage
            MailService.deleteMessage(MailService.selectedAccountId, MailService.selectedFolder, msg.uid, function() {
              MailService.listMessages(0, 50)
              root.backRequested()
            })
          }
        }

        Item { width: Style.marginM }

        CrawlIconButton {
          icon: "close"
          tooltipText: "Close"
          baseSize: Style.baseWidgetSize * 0.7
          onClicked: root.backRequested()
        }
      }
    }

    // ── Message content ──────────────────────────────────────────────────
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
          visible: !MailService.currentMessage
          Layout.fillWidth: true
          Layout.preferredHeight: 200
          color: "transparent"

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
              text: "Loading message..."
              pointSize: Style.fontSizeS
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }
          }
        }

        // Message header details
        CrawlBox {
          visible: !!MailService.currentMessage
          Layout.fillWidth: true
          implicitHeight: detailsCol.implicitHeight + Style.margin2M

          ColumnLayout {
            id: detailsCol
            anchors.fill: parent
            anchors.margins: Style.marginM
            spacing: Style.marginS

            // Subject
            CrawlText {
              text: MailService.currentMessage ? MailService.currentMessage.subject || "(no subject)" : ""
              pointSize: Style.fontSizeXL
              font.weight: Style.fontWeightBold
              color: Theme.cOnSurface
              wrapMode: Text.Wrap
              Layout.fillWidth: true
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: Theme.cOutline; opacity: 0.2 }

            // From
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS

              CrawlText {
                text: "From:"
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                font.weight: Style.fontWeightBold
              }

              CrawlText {
                text: MailService.currentMessage ? MailService.currentMessage.from : ""
                pointSize: Style.fontSizeS
                color: Theme.cOnSurface
                Layout.fillWidth: true
                elide: Text.ElideRight
              }
            }

            // To
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS
              visible: MailService.currentMessage && MailService.currentMessage.to && MailService.currentMessage.to.length > 0

              CrawlText {
                text: "To:"
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                font.weight: Style.fontWeightBold
              }

              CrawlText {
                text: MailService.currentMessage ? MailService.currentMessage.to.join(", ") : ""
                pointSize: Style.fontSizeS
                color: Theme.cOnSurface
                Layout.fillWidth: true
                elide: Text.ElideRight
                wrapMode: Text.Wrap
              }
            }

            // CC
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS
              visible: MailService.currentMessage && MailService.currentMessage.cc && MailService.currentMessage.cc.length > 0

              CrawlText {
                text: "Cc:"
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                font.weight: Style.fontWeightBold
              }

              CrawlText {
                text: MailService.currentMessage ? MailService.currentMessage.cc.join(", ") : ""
                pointSize: Style.fontSizeS
                color: Theme.cOnSurface
                Layout.fillWidth: true
                elide: Text.ElideRight
                wrapMode: Text.Wrap
              }
            }

            // Date
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS

              CrawlText {
                text: "Date:"
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                font.weight: Style.fontWeightBold
              }

              CrawlText {
                text: MailService.currentMessage ? _formatFullDate(MailService.currentMessage.date) : ""
                pointSize: Style.fontSizeS
                color: Theme.cOnSurface
                Layout.fillWidth: true
              }
            }

            // Flags
            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginS
              visible: MailService.currentMessage && MailService.currentMessage.flags && MailService.currentMessage.flags.length > 0

              CrawlText {
                text: "Flags:"
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                font.weight: Style.fontWeightBold
              }

              RowLayout {
                spacing: Style.marginXS
                Repeater {
                  model: MailService.currentMessage ? (MailService.currentMessage.flags || []) : []
                  delegate: Rectangle {
                    color: Qt.alpha(Theme.cPrimary, 0.1)
                    radius: Style.radiusS
                    implicitWidth: flagText.implicitWidth + Style.marginS
                    implicitHeight: flagText.implicitHeight + Style.margin2XXS

                    CrawlText {
                      id: flagText
                      anchors.centerIn: parent
                      text: modelData
                      pointSize: Style.fontSizeXXS
                      color: Theme.cPrimary
                    }
                  }
                }
              }
            }
          }
        }

        // ── Attachments ──────────────────────────────────────────────────
        AttachmentList {
          visible: MailService.currentMessage
            && MailService.currentMessage.attachments
            && MailService.currentMessage.attachments.length > 0
          Layout.fillWidth: true
          attachments: MailService.currentMessage ? (MailService.currentMessage.attachments || []) : []
          accountId: MailService.selectedAccountId
          messageUid: MailService.currentMessage ? MailService.currentMessage.uid : 0
        }

        // ── Body text ────────────────────────────────────────────────────
        CrawlBox {
          visible: !!MailService.currentMessage
          Layout.fillWidth: true
          implicitHeight: bodyText.implicitHeight + Style.margin2L

          Flickable {
            id: bodyFlickable
            anchors.fill: parent
            anchors.margins: Style.marginM
            contentHeight: bodyText.implicitHeight
            clip: true
            interactive: false

            CrawlText {
              id: bodyText
              width: parent.width
              text: MailService.currentMessage ? (MailService.currentMessage.body_text || MailService.currentMessage.body_html || "(No content)") : ""
              pointSize: Style.fontSizeS
              color: Theme.cOnSurface
              wrapMode: Text.Wrap
              textFormat: MailService.currentMessage && MailService.currentMessage.body_html
                ? Text.RichText : Text.PlainText
              onLinkActivated: link => Qt.openUrlExternally(link)
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
      return d.toLocaleString(Qt.locale(), "yyyy-MM-dd hh:mm:ss")
    } catch (e) {
      return dateStr
    }
  }
}
