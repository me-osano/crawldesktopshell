import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import qs.Common
import qs.Modules.Mail
import qs.Services
import qs.Widgets

FloatingWindow {
  id: root

  title: "Mail"
  minimumSize: Qt.size(Math.round(900 * Style.uiScaleRatio), Math.round(600 * Style.uiScaleRatio))
  implicitWidth: Math.round(900 * Style.uiScaleRatio)
  implicitHeight: Math.round(600 * Style.uiScaleRatio)
  color: "transparent"

  visible: false

  property string activeView: "list"
  property string replyToMessageId: ""
  property string replyToSubject: ""
  property var replyToAddresses: ([])

  Component.onCompleted: {
    MailService.mailWindow = root
    MailService.listAccounts()
  }

  Shortcut {
    sequence: "Escape"
    onActivated: MailService.closeWindow()
  }

  onVisibleChanged: {
    if (visible) {
      MailService.isWindowOpen = true
    } else {
      MailService.isWindowOpen = false
    }
  }

  Rectangle {
    anchors.fill: parent
    color: Theme.cSurface
    radius: Style.radiusL
    clip: true

    RowLayout {
      anchors {
        fill: parent
        margins: Style.marginM
      }
      spacing: 0

      AccountList {
        Layout.preferredWidth: Math.round(200 * Style.uiScaleRatio)
        Layout.fillHeight: true

        onAccountSelected: accountId => MailService.selectAccount(accountId)
        onFolderSelected: (accountId, folder) => {
          MailService.selectFolder(accountId, folder)
          root.activeView = "list"
        }
      }

      Rectangle {
        width: 1
        Layout.fillHeight: true
        color: Theme.cOutline
        opacity: 0.3
      }

      Item {
        Layout.fillWidth: true
        Layout.fillHeight: true

        StackLayout {
          anchors.fill: parent
          currentIndex: root.activeView === "list" ? 0 : root.activeView === "message" ? 1 : 2

          ColumnLayout {
            spacing: 0
            CrawlBox {
              Layout.fillWidth: true
              Layout.preferredHeight: headerRow.implicitHeight + Style.margin2M
              RowLayout {
                id: headerRow
                anchors.fill: parent
                anchors.margins: Style.marginM
                spacing: Style.marginM
                CrawlIcon { icon: "mail"; pointSize: Style.fontSizeL; color: Theme.cPrimary }
                CrawlLabel { label: MailService.selectedFolder || "Mail"; Layout.fillWidth: true }
                CrawlIconButton { icon: "refresh"; tooltipText: "Sync"; baseSize: Style.baseWidgetSize * 0.8; enabled: !MailService.syncing; onClicked: MailService.syncNow(MailService.selectedAccountId) }
                CrawlIconButton { icon: "close"; tooltipText: "Close"; baseSize: Style.baseWidgetSize * 0.8; onClicked: MailService.closeWindow() }
              }
            }
            SearchBar {
              Layout.fillWidth: true
              Layout.preferredHeight: Style.baseWidgetSize
              Layout.leftMargin: Style.marginM
              Layout.rightMargin: Style.marginM
              Layout.topMargin: Style.marginS
              onSearch: query => {
                if (query.trim().length > 0) {
                  MailService.searchMessages(query)
                } else {
                  MailService.clearSearch()
                  MailService.listMessages(0, 50)
                }
              }
              onCleared: { MailService.clearSearch(); MailService.listMessages(0, 50) }
            }
            Rectangle {
              visible: MailService.syncing
              Layout.fillWidth: true
              Layout.preferredHeight: 3
              color: Theme.cPrimary
              SequentialAnimation on opacity {
                loops: Animation.Infinite
                PropertyAnimation { from: 1; to: 0.3; duration: 800 }
                PropertyAnimation { from: 0.3; to: 1; duration: 800 }
              }
            }
            MessageList {
              Layout.fillWidth: true
              Layout.fillHeight: true
              Layout.leftMargin: Style.marginS
              Layout.rightMargin: Style.marginS
              Layout.bottomMargin: Style.marginS
              onMessageSelected: uid => {
                MailService.selectMessage(uid)
                root.activeView = "message"
              }
              onComposeRequested: {
                root.replyToMessageId = ""
                root.replyToSubject = ""
                root.replyToAddresses = []
                root.activeView = "compose"
              }
            }
          }

          MessageView {
            onBackRequested: root.activeView = "list"
            onReplyRequested: (msgId, fromAddr, subject) => { root.replyToMessageId = msgId; root.replyToSubject = "Re: " + subject; root.replyToAddresses = [fromAddr]; root.activeView = "compose" }
            onForwardRequested: (msgId, subject) => { root.replyToMessageId = msgId; root.replyToSubject = "Fwd: " + subject; root.replyToAddresses = []; root.activeView = "compose" }
          }

          ComposeMessage {
            replyToId: root.replyToMessageId; replySubject: root.replyToSubject; replyToAddresses: root.replyToAddresses
            onSendRequested: params => { MailService.sendMessage(params, function(qi, err) { if (!err) { ToastService.showNotice("Mail", "Message queued for sending", "mail"); root.activeView = "list" } else { ToastService.showNotice("Mail", "Failed to send: " + (err.message || "unknown"), "mail") } }) }
            onDiscardRequested: root.activeView = "list"
          }
        }
      }
    }
  }
}
