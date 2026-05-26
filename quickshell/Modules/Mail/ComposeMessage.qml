import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  property string replyToId: ""
  property string replySubject: ""
  property var replyToAddresses: ([])

  signal sendRequested(var params)
  signal discardRequested()

  function _buildSendParams() {
    return {
      account_id: MailService.selectedAccountId,
      from: _from.text,
      to: _to.text.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s.length > 0 }),
      cc: _cc.text.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s.length > 0 }),
      bcc: _bcc.text.split(",").map(function(s) { return s.trim() }).filter(function(s) { return s.length > 0 }),
      subject: _subject.text,
      body_text: _body.text,
      body_html: null,
      attachments: [],
      in_reply_to: root.replyToId.length > 0 ? root.replyToId : null
    }
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: 0

    // ── Header ───────────────────────────────────────────────────────────
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
          tooltipText: "Discard"
          baseSize: Style.baseWidgetSize * 0.7
          onClicked: _confirmDiscard()
        }

        CrawlIcon {
          icon: "edit"
          pointSize: Style.fontSizeL
          color: Theme.cPrimary
        }

        CrawlLabel {
          label: root.replyToId.length > 0 ? "Reply" : "New Message"
          Layout.fillWidth: true
        }

        CrawlIconButton {
          icon: "send"
          tooltipText: "Send"
          baseSize: Style.baseWidgetSize * 0.8
          enabled: _to.text.length > 0 && _subject.text.length > 0
          onClicked: root.sendRequested(root._buildSendParams())
        }

        CrawlIconButton {
          icon: "close"
          tooltipText: "Discard"
          baseSize: Style.baseWidgetSize * 0.7
          onClicked: _confirmDiscard()
        }
      }
    }

    // ── Compose form ─────────────────────────────────────────────────────
    CrawlScrollView {
      Layout.fillWidth: true
      Layout.fillHeight: true
      horizontalPolicy: ScrollBar.AlwaysOff
      verticalPolicy: ScrollBar.AsNeeded
      gradientColor: Theme.cSurface

      ColumnLayout {
        width: parent.width - Style.marginM - Style.marginM
        x: Style.marginM
        spacing: Style.marginS

        Item { height: Style.marginS }

        // From
        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginS

          CrawlText {
            text: "From:"
            pointSize: Style.fontSizeS
            color: Theme.cOnSurfaceVariant
            font.weight: Style.fontWeightBold
            width: 60
          }

          TextField {
            id: _from
            Layout.fillWidth: true
            placeholderText: "your@email.com"
            color: Theme.cOnSurface
            background: Rectangle {
              color: Theme.cSurfaceVariant
              radius: Style.radiusS
              border.width: Style.borderS
              border.color: Theme.cOutline
            }
          }
        }

        // To
        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginS

          CrawlText {
            text: "To:"
            pointSize: Style.fontSizeS
            color: Theme.cOnSurfaceVariant
            font.weight: Style.fontWeightBold
            width: 60
          }

          TextField {
            id: _to
            Layout.fillWidth: true
            placeholderText: "recipient@example.com"
            color: Theme.cOnSurface
            background: Rectangle {
              color: Theme.cSurfaceVariant
              radius: Style.radiusS
              border.width: Style.borderS
              border.color: Theme.cOutline
            }
            text: root.replyToAddresses.join(", ")
          }
        }

        // CC
        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginS

          CrawlText {
            text: "Cc:"
            pointSize: Style.fontSizeS
            color: Theme.cOnSurfaceVariant
            font.weight: Style.fontWeightBold
            width: 60
          }

          TextField {
            id: _cc
            Layout.fillWidth: true
            placeholderText: "cc@example.com (optional)"
            color: Theme.cOnSurface
            background: Rectangle {
              color: Theme.cSurfaceVariant
              radius: Style.radiusS
              border.width: Style.borderS
              border.color: Theme.cOutline
            }
          }
        }

        // BCC
        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginS
          visible: _showBcc.checked

          CrawlText {
            text: "Bcc:"
            pointSize: Style.fontSizeS
            color: Theme.cOnSurfaceVariant
            font.weight: Style.fontWeightBold
            width: 60
          }

          TextField {
            id: _bcc
            Layout.fillWidth: true
            placeholderText: "bcc@example.com (optional)"
            color: Theme.cOnSurface
            background: Rectangle {
              color: Theme.cSurfaceVariant
              radius: Style.radiusS
              border.width: Style.borderS
              border.color: Theme.cOutline
            }
          }
        }

        // Subject
        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginS

          CrawlText {
            text: "Subject:"
            pointSize: Style.fontSizeS
            color: Theme.cOnSurfaceVariant
            font.weight: Style.fontWeightBold
            width: 60
          }

          TextField {
            id: _subject
            Layout.fillWidth: true
            placeholderText: "Subject"
            color: Theme.cOnSurface
            background: Rectangle {
              color: Theme.cSurfaceVariant
              radius: Style.radiusS
              border.width: Style.borderS
              border.color: Theme.cOutline
            }
            text: root.replySubject
          }
        }

        // Options row
        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginS

          CheckBox {
            id: _showBcc
            text: "Bcc"
            checked: false
            contentItem: CrawlText {
              text: _showBcc.text
              pointSize: Style.fontSizeXS
              color: Theme.cOnSurfaceVariant
              verticalAlignment: Text.AlignVCenter
            }
            indicator: Rectangle {
              width: 16
              height: 16
              radius: 3
              color: _showBcc.checked ? Theme.cPrimary : Theme.cSurfaceVariant
              border.width: Style.borderS
              border.color: Theme.cOutline

              CrawlIcon {
                anchors.centerIn: parent
                icon: "check"
                pointSize: 8
                color: Theme.cOnPrimary
                visible: _showBcc.checked
              }
            }
          }
        }

        Rectangle {
          Layout.fillWidth: true
          height: 1
          color: Theme.cOutline
          opacity: 0.2
        }

        // Body
        Rectangle {
          id: bodyContainer
          Layout.fillWidth: true
          Layout.fillHeight: true
          Layout.bottomMargin: Style.marginM
          color: Theme.cSurfaceVariant
          radius: Style.radiusS
          border.width: Style.borderS
          border.color: Theme.cOutline

          ScrollView {
            anchors.fill: parent
            anchors.margins: Style.marginS
            clip: true

            TextArea {
              id: _body
              width: parent.width
              placeholderText: "Write your message..."
              color: Theme.cOnSurface
              background: null
              wrapMode: TextArea.Wrap
              topPadding: 4
              leftPadding: 4
            }
          }
        }

        // Bottom toolbar
        RowLayout {
          Layout.fillWidth: true
          spacing: Style.marginS
          Layout.bottomMargin: Style.marginS

          CrawlText {
            text: "Attachments not yet supported"
            pointSize: Style.fontSizeXXS
            color: Theme.cOnSurfaceVariant
            visible: false
          }

          Item { Layout.fillWidth: true }

          CrawlButton {
            text: "Discard"
            onClicked: _confirmDiscard()
          }

          CrawlButton {
            text: "Send"
            icon: "send"
            enabled: _to.text.length > 0 && _subject.text.length > 0
            onClicked: root.sendRequested(root._buildSendParams())
          }
        }
      }
    }
  }

  function _confirmDiscard() {
    if (_body.text.length > 0 || _subject.text.length > 0) {
      _discardDialog.visible = true
    } else {
      root.discardRequested()
    }
  }

  Rectangle {
    id: _discardDialog
    visible: false
    anchors.fill: parent
    color: Qt.alpha(Theme.cSurface, 0.9)
    z: 10

    ColumnLayout {
      anchors.centerIn: parent
      spacing: Style.marginL

      CrawlText {
        text: "Discard this message?"
        pointSize: Style.fontSizeL
        color: Theme.cOnSurface
      }

      RowLayout {
        spacing: Style.marginM
        Layout.alignment: Qt.AlignHCenter

        CrawlButton {
          text: "Keep editing"
          onClicked: _discardDialog.visible = false
        }

        CrawlButton {
          text: "Discard"
          onClicked: {
            _discardDialog.visible = false
            root.discardRequested()
          }
        }
      }
    }
  }
}
