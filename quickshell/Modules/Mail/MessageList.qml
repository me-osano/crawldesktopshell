import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  signal messageSelected(int uid)
  signal composeRequested()

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.marginS

    // ── Toolbar ──────────────────────────────────────────────────────────
    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginS

      CrawlIconButton {
        icon: "edit"
        tooltipText: "Compose new message"
        baseSize: Style.baseWidgetSize * 0.7
        onClicked: root.composeRequested()
      }

      Item { Layout.fillWidth: true }

      CrawlText {
        text: MailService.totalMessages > 0 ? MailService.totalMessages + " messages" : ""
        pointSize: Style.fontSizeXS
        color: Theme.cOnSurfaceVariant
        visible: text.length > 0
      }
    }

    // ── Message list or empty/loading state ──────────────────────────────
    CrawlScrollView {
      Layout.fillWidth: true
      Layout.fillHeight: true
      horizontalPolicy: ScrollBar.AlwaysOff
      verticalPolicy: ScrollBar.AsNeeded
      gradientColor: Theme.cSurface

      ColumnLayout {
        width: parent.width
        spacing: 0

        // Loading state
        Rectangle {
          visible: MailService.loadingMessages
          Layout.fillWidth: true
          Layout.preferredHeight: 100

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
              text: "Loading messages..."
              pointSize: Style.fontSizeS
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }
          }
        }

        // Search results header
        Rectangle {
          visible: MailService.searchResults !== null && MailService.searchResults.length > 0
          Layout.fillWidth: true
          Layout.preferredHeight: searchLabel.implicitHeight + Style.marginS
          color: Qt.alpha(Theme.cPrimary, 0.08)

          RowLayout {
            anchors.fill: parent
            anchors.margins: Style.marginS
            spacing: Style.marginS

            CrawlIcon {
              icon: "search"
              pointSize: Style.fontSizeXS
              color: Theme.cPrimary
            }

            CrawlText {
              id: searchLabel
              text: "Search results (" + MailService.searchResults.length + ")"
              pointSize: Style.fontSizeXS
              color: Theme.cPrimary
              Layout.fillWidth: true
            }

            CrawlIconButton {
              icon: "close"
              baseSize: Style.baseWidgetSize * 0.5
              onClicked: {
                MailService.clearSearch()
                MailService.listMessages(0, 50)
              }
            }
          }
        }

        // No messages state
        Rectangle {
          visible: !MailService.loadingMessages
            && !MailService.selectedFolder
          Layout.fillWidth: true
          Layout.preferredHeight: 100

          ColumnLayout {
            anchors.centerIn: parent
            spacing: Style.marginM

            CrawlIcon {
              icon: "folder-open"
              pointSize: 36
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlText {
              text: "Select a folder to view messages"
              pointSize: Style.fontSizeS
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }
          }
        }

        // Empty folder
        Rectangle {
          visible: !MailService.loadingMessages
            && MailService.selectedFolder
            && ((MailService.searchResults === null && (!MailService.messages || MailService.messages.length === 0))
             || (MailService.searchResults !== null && MailService.searchResults.length === 0))
          Layout.fillWidth: true
          Layout.preferredHeight: 100

          ColumnLayout {
            anchors.centerIn: parent
            spacing: Style.marginM

            CrawlIcon {
              icon: "inbox"
              pointSize: 36
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }

            CrawlText {
              text: MailService.searchResults !== null ? "No results found" : "No messages"
              pointSize: Style.fontSizeS
              color: Theme.cOnSurfaceVariant
              Layout.alignment: Qt.AlignHCenter
            }
          }
        }

        // Message list
        Repeater {
          model: {
            var src = MailService.searchResults !== null ? MailService.searchResults : MailService.messages
            return src || []
          }

          delegate: MessageItem {
            required property var modelData
            width: parent.width
            messageData: modelData
            selected: MailService.selectedMessageUids.indexOf(modelData.uid) >= 0

            onClicked: root.messageSelected(modelData.uid)
            onDoubleClicked: MailService.getMessage(MailService.selectedAccountId, modelData.uid, true)
          }
        }
      }
    }
  }
}
