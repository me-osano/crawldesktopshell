import QtQuick
import QtQuick.Layouts
import qs.Common

ColumnLayout {
  id: root

  property string label: ""
  property string description: ""
  property bool expanded: false
  property real contentSpacing: Style.marginM
  property bool _userInteracted: false

  signal toggled(bool expanded)

  Layout.fillWidth: true
  spacing: 0

  // Default property to accept children
  default property alias content: contentLayout.children

  // Header with clickable area
  Rectangle {
    id: headerContainer
    Layout.fillWidth: true
    Layout.preferredHeight: headerContent.implicitHeight + Style.margin2M
    color: root.expanded ? Theme.cSecondary : Theme.cPrimary
    radius: Style.iRadiusM
    border.color: root.expanded ? Theme.cOnSecondary : Theme.cOutline
    border.width: Style.borderS

    // Smooth color transitions
    Behavior on color {
      enabled: root._userInteracted
      ColorAnimation {
        duration: Style.animationNormal
        easing.type: Easing.OutCubic
      }
    }

    Behavior on border.color {
      enabled: root._userInteracted
      ColorAnimation {
        duration: Style.animationNormal
        easing.type: Easing.OutCubic
      }
    }

    MouseArea {
      id: headerArea
      anchors.fill: parent
      cursorShape: Qt.PointingHandCursor
      hoverEnabled: true

      onClicked: {
        root._userInteracted = true;
        root.expanded = !root.expanded;
        root.toggled(root.expanded);
      }

      // Hover effect overlay
      Rectangle {
        anchors.fill: parent
        color: headerArea.containsMouse ? Theme.cOnSurface : "transparent"
        opacity: headerArea.containsMouse ? 0.08 : 0
        radius: headerContainer.radius // Reference the container's radius directly

        Behavior on opacity {
          NumberAnimation {
            duration: Style.animationFast
          }
        }
      }
    }

    RowLayout {
      id: headerContent
      anchors.fill: parent
      anchors.margins: Style.marginM
      spacing: Style.marginM

      // Expand/collapse icon with rotation animation
      CrawlIcon {
        id: chevronIcon
        icon: "chevron-right"
        pointSize: Style.fontSizeL
        color: root.expanded ? Theme.cOnSecondary : Theme.cOnPrimary
        Layout.alignment: Qt.AlignVCenter

        rotation: root.expanded ? 90 : 0
        Behavior on rotation {
          enabled: root._userInteracted
          NumberAnimation {
            duration: Style.animationNormal
            easing.type: Easing.OutCubic
          }
        }

        Behavior on color {
          enabled: root._userInteracted
          ColorAnimation {
            duration: Style.animationNormal
          }
        }
      }

      // Header text content - properly contained
      RowLayout {
        Layout.fillWidth: true
        Layout.alignment: Qt.AlignVCenter
        spacing: Style.marginL

        CrawlText {
          text: root.label
          pointSize: Style.fontSizeL
          font.weight: Style.fontWeightSemiBold
          color: root.expanded ? Theme.cOnSecondary : Theme.cOnPrimary
          wrapMode: Text.WordWrap

          Behavior on color {
            enabled: root._userInteracted
            ColorAnimation {
              duration: Style.animationNormal
            }
          }
        }

        CrawlText {
          text: root.description
          pointSize: Style.fontSizeS
          font.weight: Style.fontWeightRegular
          color: root.expanded ? Theme.cOnSecondary : Theme.cOnPrimary
          Layout.fillWidth: true
          wrapMode: Text.WordWrap
          visible: root.description !== ""
          opacity: 0.87

          Behavior on color {
            enabled: root._userInteracted
            ColorAnimation {
              duration: Style.animationNormal
            }
          }
        }
      }
    }
  }

  // Collapsible content with Material 3 styling
  Rectangle {
    id: contentContainer
    Layout.fillWidth: true
    Layout.topMargin: Style.marginS

    visible: root.expanded
    color: Theme.cSurface
    radius: Style.iRadiusL
    border.color: Theme.cOutline
    border.width: Style.borderS

    // Dynamic height based on content
    Layout.preferredHeight: expanded ? contentLayout.implicitHeight + Style.margin2L : 0

    // Smooth height animation
    Behavior on Layout.preferredHeight {
      enabled: root._userInteracted
      NumberAnimation {
        duration: Style.animationNormal
        easing.type: Easing.OutCubic
      }
    }

    // Content layout
    ColumnLayout {
      id: contentLayout
      anchors.fill: parent
      anchors.margins: Style.marginL
      spacing: root.contentSpacing
    }

    // Fade in animation for content
    opacity: root.expanded ? 1.0 : 0.0
    Behavior on opacity {
      enabled: root._userInteracted
      NumberAnimation {
        duration: Style.animationNormal
        easing.type: Easing.OutCubic
      }
    }
  }
}
