import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root

  required property var feedData
  property bool isSelected: false
  property bool isAllEntries: false
  property bool isCategory: false
  property bool hovered: false

  signal clicked()

  Layout.fillWidth: true
  Layout.preferredHeight: row.implicitHeight + Style.marginS

  Rectangle {
    id: row
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.verticalCenter: parent.verticalCenter
    anchors.leftMargin: Style.marginS
    anchors.rightMargin: Style.marginS
    implicitHeight: content.implicitHeight + Style.marginS
    radius: Style.radiusM
    color: root.isSelected
      ? Qt.alpha(Theme.cPrimary, 0.15)
      : (area.containsMouse ? Qt.alpha(Theme.cPrimary, 0.06) : "transparent")
    border.width: root.isSelected ? Style.borderS : 0
    border.color: root.isSelected ? Theme.cPrimary : "transparent"

    RowLayout {
      id: content
      anchors.fill: parent
      anchors.leftMargin: isCategory ? Style.marginM : (isAllEntries ? Style.marginS : Style.marginL)
      anchors.rightMargin: Style.marginS
      spacing: Style.marginS

      CrawlIcon {
        icon: root.isAllEntries ? "inbox"
          : root.isCategory ? "folder"
          : (root.feedData.icon_url && root.feedData.icon_url.length > 0 ? "" : "rss")
        pointSize: Style.fontSizeS
        color: root.isSelected ? Theme.cPrimary : (root.isCategory ? Theme.cSecondary : Theme.cOnSurfaceVariant)
        visible: !root.isAllEntries || !root.isCategory || !root.feedData.icon_url || root.feedData.icon_url.length === 0
      }

      CrawlText {
        text: root.feedData.title || root.feedData.id || "Feed"
        pointSize: Style.fontSizeXS
        font.weight: root.isSelected ? Style.fontWeightBold : Style.fontWeightNormal
        color: root.isSelected ? Theme.cPrimary : Theme.cOnSurface
        elide: Text.ElideRight
        Layout.fillWidth: true
      }

      Rectangle {
        visible: (root.feedData.unread_count || 0) > 0
        color: Theme.cPrimary
        radius: height * 0.5
        implicitWidth: badge.implicitWidth + Style.marginS
        implicitHeight: badge.implicitHeight + Style.margin2XXS

        CrawlText {
          id: badge
          anchors.centerIn: parent
          text: root.feedData.unread_count > 99 ? "99+" : String(root.feedData.unread_count || 0)
          pointSize: Style.fontSizeXXS
          color: Theme.cOnPrimary
        }
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
}
