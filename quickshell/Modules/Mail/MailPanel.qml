import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import qs.Common
import qs.Modules.MainScreen
import qs.Services
import qs.Widgets

SmartPanel {
  id: root

  preferredWidth: Math.round(900 * Style.uiScaleRatio)
  preferredHeight: Math.round(600 * Style.uiScaleRatio)

  readonly property string panelMode: Settings.data.ui.mailPanelMode
  readonly property bool attachToBar: panelMode === "attached"
  readonly property bool isWindowMode: panelMode === "window"

  forceAttachToBar: attachToBar
  panelAnchorHorizontalCenter: !attachToBar
  panelAnchorVerticalCenter: !attachToBar

  // ── State ──────────────────────────────────────────────────────────────
  property string activeView: "list" // "list" | "message" | "compose"
  property string replyToMessageId: ""
  property string replyToSubject: ""
  property var replyToAddresses: ([])

  // ── Window mode ───────────────────────────────────────────────────────
  function toggle(buttonItem, buttonName) {
    if (isWindowMode) {
      MailService.toggleWindow()
      return
    }
    if (isPanelOpen) {
      close()
    } else if (!attachToBar) {
      open(null, null)
    } else {
      open(buttonItem, buttonName)
    }
  }

  // ── Panel lifecycle ────────────────────────────────────────────────────
  onOpened: {
    MailService.listAccounts()
  }

  panelContent: Rectangle {
    color: "transparent"
    property bool allowAttach: root.attachToBar

    RowLayout {
      anchors {
        fill: parent
        margins: Style.marginM
      }
      spacing: 0

      // ── Left pane: Accounts + Folders ──────────────────────────────────
      AccountList {
        id: accountList
        Layout.preferredWidth: Math.round(200 * Style.uiScaleRatio)
        Layout.fillHeight: true

        onAccountSelected: accountId => MailService.selectAccount(accountId)
        onFolderSelected: (accountId, folder) => {
          MailService.selectFolder(accountId, folder)
          root.activeView = "list"
        }
      }

      // ── Divider ────────────────────────────────────────────────────────
      Rectangle {
        width: 1
        Layout.fillHeight: true
        color: Theme.cOutline
        opacity: 0.3
      }

      // ── Center + Right panes ───────────────────────────────────────────
      Item {
        Layout.fillWidth: true
        Layout.fillHeight: true

        // ── Three-state view stack ──────────────────────────────────────
        StackLayout {
          anchors.fill: parent
          currentIndex: root.activeView === "list" ? 0
            : root.activeView === "message" ? 1
            : 2

          // ── List view: SearchBar + MessageList ─────────────────────────
          ColumnLayout {
            spacing: 0

            // Header
            CrawlBox {
              Layout.fillWidth: true
              Layout.preferredHeight: headerRow.implicitHeight + Style.margin2M

              RowLayout {
                id: headerRow
                anchors.fill: parent
                anchors.margins: Style.marginM
                spacing: Style.marginM

                CrawlIcon {
                  icon: "mail"
                  pointSize: Style.fontSizeL
                  color: Theme.cPrimary
                }

                CrawlLabel {
                  label: MailService.selectedFolder || "Mail"
                  Layout.fillWidth: true
                }

                CrawlIconButton {
                  icon: "refresh"
                  tooltipText: "Sync"
                  baseSize: Style.baseWidgetSize * 0.8
                  enabled: !MailService.syncing
                  onClicked: MailService.syncNow(MailService.selectedAccountId)
                }

                CrawlIconButton {
                  icon: "close"
                  tooltipText: "Close"
                  baseSize: Style.baseWidgetSize * 0.8
                  onClicked: root.close()
                }
              }
            }

            // Search bar
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
              onCleared: {
                MailService.clearSearch()
                MailService.listMessages(0, 50)
              }
            }

            // Syncing indicator
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

            // Message list
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

          // ── Message view ──────────────────────────────────────────────
          MessageView {
            id: messageView

            onBackRequested: root.activeView = "list"
            onReplyRequested: (msgId, fromAddr, subject) => {
              root.replyToMessageId = msgId
              root.replyToSubject = "Re: " + subject
              root.replyToAddresses = [fromAddr]
              root.activeView = "compose"
            }
            onForwardRequested: (msgId, subject) => {
              root.replyToMessageId = msgId
              root.replyToSubject = "Fwd: " + subject
              root.replyToAddresses = []
              root.activeView = "compose"
            }
          }

          // ── Compose view ──────────────────────────────────────────────
          ComposeMessage {
            replyToId: root.replyToMessageId
            replySubject: root.replyToSubject
            replyToAddresses: root.replyToAddresses

            onSendRequested: params => {
              MailService.sendMessage(params, function(queueId, error) {
                if (!error) {
                  ToastService.showNotice("Mail", "Message queued for sending", "mail")
                  root.activeView = "list"
                } else {
                  ToastService.showNotice("Mail", "Failed to send: " + (error.message || "unknown"), "mail")
                }
              })
            }
            onDiscardRequested: root.activeView = "list"
          }
        }
      }
    }
  }
}
