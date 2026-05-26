import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  signal accountSelected(string accountId)
  signal folderSelected(string accountId, string folder)

  ColumnLayout {
    anchors.fill: parent
    spacing: 0

    // ── Accounts header ──────────────────────────────────────────────────
    CrawlBox {
      Layout.fillWidth: true
      Layout.preferredHeight: header.implicitHeight + Style.margin2M

      RowLayout {
        id: header
        anchors.fill: parent
        anchors.margins: Style.marginM
        spacing: Style.marginS

        CrawlIcon {
          icon: "mail"
          pointSize: Style.fontSizeM
          color: Theme.cPrimary
        }

        CrawlLabel {
          label: "Accounts"
          Layout.fillWidth: true
        }

        CrawlIconButton {
          icon: "plus"
          tooltipText: "Add account"
          baseSize: Style.baseWidgetSize * 0.7
          onClicked: _showAddDialog()
        }
      }
    }

    // ── Scrollable account list ──────────────────────────────────────────
    CrawlScrollView {
      Layout.fillWidth: true
      Layout.fillHeight: true
      horizontalPolicy: ScrollBar.AlwaysOff
      verticalPolicy: ScrollBar.AsNeeded
      gradientColor: Theme.cSurface

      ColumnLayout {
        width: parent.width
        spacing: 0

        Repeater {
          model: {
            var accs = MailService.accounts || []
            var arr = []
            for (var i = 0; i < (accs.length || 0); i++) arr.push(accs[i])
            return arr
          }

          delegate: Item {
            id: accountDelegate
            required property var modelData
            Layout.fillWidth: true
            Layout.preferredHeight: accountBox.implicitHeight + Style.marginS

            CrawlBox {
              id: accountBox
              anchors.left: parent.left
              anchors.right: parent.right
              anchors.verticalCenter: parent.verticalCenter
              anchors.leftMargin: Style.marginS
              anchors.rightMargin: Style.marginS
              implicitHeight: accountRow.implicitHeight + Style.marginM
              color: accountMouse.containsMouse
                ? Qt.alpha(Theme.cPrimary, 0.08)
                : (MailService.selectedAccountId === modelData.id ? Qt.alpha(Theme.cPrimary, 0.12) : "transparent")
              radius: Style.radiusM
              border.width: MailService.selectedAccountId === modelData.id ? Style.borderS : 0
              border.color: Theme.cPrimary

              RowLayout {
                id: accountRow
                anchors.fill: parent
                anchors.margins: Style.marginS
                spacing: Style.marginS

                CrawlIcon {
                  icon: "user-circle"
                  pointSize: Style.fontSizeL
                  color: Theme.cPrimary
                }

                ColumnLayout {
                  Layout.fillWidth: true
                  spacing: 1

                  CrawlText {
                    text: modelData.display_name || modelData.email || modelData.id
                    pointSize: Style.fontSizeS
                    font.weight: Style.fontWeightMedium
                    color: Theme.cOnSurface
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                  }

                  CrawlText {
                    text: modelData.email || ""
                    pointSize: Style.fontSizeXS
                    color: Theme.cOnSurfaceVariant
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                    visible: text.length > 0
                  }
                }

                Rectangle {
                  visible: (modelData.unread_count || 0) > 0
                  color: Theme.cError
                  radius: height * 0.5
                  implicitWidth: badgeText.implicitWidth + Style.marginS
                  implicitHeight: badgeText.implicitHeight + Style.margin2XXS

                  CrawlText {
                    id: badgeText
                    anchors.centerIn: parent
                    text: modelData.unread_count > 99 ? "99+" : String(modelData.unread_count || 0)
                    pointSize: Style.fontSizeXXS
                    color: Theme.cOnPrimary
                  }
                }
              }

              MouseArea {
                id: accountMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                  root.accountSelected(modelData.id)
                  _loadFolders(modelData.id)
                }
              }
            }

            // ── Folder list for this account ────────────────────────────
            FolderList {
              id: folderList
              anchors.left: parent.left
              anchors.right: parent.right
              anchors.top: accountBox.bottom
              anchors.topMargin: Style.marginXS
              visible: MailService.selectedAccountId === modelData.id
              accountId: modelData.id

              onFolderSelected: (accountId, folder) => root.folderSelected(accountId, folder)
            }
          }
        }

        // ── Empty state ────────────────────────────────────────────────
        CrawlBox {
          visible: !MailService.accounts || MailService.accounts.length === 0
          Layout.fillWidth: true
          Layout.preferredHeight: emptyCol.implicitHeight + Style.margin2L
          Layout.topMargin: Style.marginL

          ColumnLayout {
            id: emptyCol
            anchors.centerIn: parent
            spacing: Style.marginM

            CrawlIcon {
              icon: "mail-off"
              pointSize: 36
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlText {
              text: "No accounts"
              pointSize: Style.fontSizeM
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlButton {
              text: "Add account"
              icon: "plus"
              Layout.alignment: Qt.AlignHCenter
              onClicked: _showAddDialog()
            }
          }
        }
      }
    }
  }

  // ── Add account dialog (simple inline) ─────────────────────────────────
  function _showAddDialog() {
    _addDialog.visible = true
  }

  Rectangle {
    id: _addDialog
    visible: false
    anchors.fill: parent
    color: Qt.alpha(Theme.cSurface, 0.95)
    z: 10

    ColumnLayout {
      anchors.centerIn: parent
      width: parent.width * 0.85
      spacing: Style.marginM

      CrawlLabel { label: "Add Mail Account" }

      CrawlBox {
        Layout.fillWidth: true
        implicitHeight: formCol.implicitHeight + Style.margin2M

        ColumnLayout {
          id: formCol
          anchors.fill: parent
          anchors.margins: Style.marginM
          spacing: Style.marginS

          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginS

            ColumnLayout {
              Layout.fillWidth: true
              spacing: Style.marginXS

              CrawlText { text: "Display Name"; pointSize: Style.fontSizeXS; color: Theme.cOnSurfaceVariant }
              TextField {
                id: _displayName
                Layout.fillWidth: true
                placeholderText: "John Doe"
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

              CrawlText { text: "Email"; pointSize: Style.fontSizeXS; color: Theme.cOnSurfaceVariant }
              TextField {
                id: _email
                Layout.fillWidth: true
                placeholderText: "john@example.com"
                color: Theme.cOnSurface
                background: Rectangle {
                  color: Theme.cSurfaceVariant
                  radius: Style.radiusS
                  border.width: Style.borderS
                  border.color: Theme.cOutline
                }
              }
            }
          }

          CrawlText { text: "IMAP Server"; pointSize: Style.fontSizeXS; color: Theme.cOnSurfaceVariant }
          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginS

            TextField {
              id: _imapHost
              Layout.fillWidth: true
              placeholderText: "imap.example.com"
              color: Theme.cOnSurface
              background: Rectangle {
                color: Theme.cSurfaceVariant
                radius: Style.radiusS
                border.width: Style.borderS
                border.color: Theme.cOutline
              }
            }
            TextField {
              id: _imapPort
              width: 80
              placeholderText: "993"
              color: Theme.cOnSurface
              background: Rectangle {
                color: Theme.cSurfaceVariant
                radius: Style.radiusS
                border.width: Style.borderS
                border.color: Theme.cOutline
              }
            }
          }

          CrawlText { text: "SMTP Server"; pointSize: Style.fontSizeXS; color: Theme.cOnSurfaceVariant }
          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginS

            TextField {
              id: _smtpHost
              Layout.fillWidth: true
              placeholderText: "smtp.example.com"
              color: Theme.cOnSurface
              background: Rectangle {
                color: Theme.cSurfaceVariant
                radius: Style.radiusS
                border.width: Style.borderS
                border.color: Theme.cOutline
              }
            }
            TextField {
              id: _smtpPort
              width: 80
              placeholderText: "587"
              color: Theme.cOnSurface
              background: Rectangle {
                color: Theme.cSurfaceVariant
                radius: Style.radiusS
                border.width: Style.borderS
                border.color: Theme.cOutline
              }
            }
          }

          RowLayout {
            Layout.fillWidth: true
            spacing: Style.marginS

            ColumnLayout {
              Layout.fillWidth: true
              spacing: Style.marginXS

              CrawlText { text: "Username"; pointSize: Style.fontSizeXS; color: Theme.cOnSurfaceVariant }
              TextField {
                id: _username
                Layout.fillWidth: true
                placeholderText: "john@example.com"
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

              CrawlText { text: "Password"; pointSize: Style.fontSizeXS; color: Theme.cOnSurfaceVariant }
              TextField {
                id: _password
                Layout.fillWidth: true
                placeholderText: "••••••••"
                echoMode: TextInput.Password
                color: Theme.cOnSurface
                background: Rectangle {
                  color: Theme.cSurfaceVariant
                  radius: Style.radiusS
                  border.width: Style.borderS
                  border.color: Theme.cOutline
                }
              }
            }
          }
        }
      }

      RowLayout {
        Layout.fillWidth: true
        spacing: Style.marginM

        Item { Layout.fillWidth: true }

        CrawlButton {
          text: "Cancel"
          onClicked: _addDialog.visible = false
        }

        CrawlButton {
          text: "Add Account"
          icon: "plus"
          enabled: _email.text.length > 0 && _imapHost.text.length > 0 && _smtpHost.text.length > 0
          onClicked: {
            MailService.addAccount({
              display_name: _displayName.text,
              email: _email.text,
              imap_host: _imapHost.text,
              imap_port: parseInt(_imapPort.text) || 993,
              smtp_host: _smtpHost.text,
              smtp_port: parseInt(_smtpPort.text) || 587,
              username: _username.text || _email.text,
              password: _password.text
            }, function(success, error) {
              if (success) {
                _addDialog.visible = false
                _displayName.text = ""; _email.text = ""
                _imapHost.text = ""; _imapPort.text = ""
                _smtpHost.text = ""; _smtpPort.text = ""
                _username.text = ""; _password.text = ""
                MailService.listAccounts()
              } else {
                ToastService.showNotice("Mail", "Add account failed: " + (error?.message || "unknown"), "mail")
              }
            })
          }
        }
      }
    }
  }

  // ── Helper ──────────────────────────────────────────────────────────────
  function _loadFolders(accountId) {
    MailService.listFolders(accountId)
  }
}
