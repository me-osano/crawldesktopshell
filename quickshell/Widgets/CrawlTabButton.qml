import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services
import qs.Widgets

Rectangle {
  id: root

  // Public properties
  property string text: ""
  property string icon: ""
  property var tooltipText
  property bool checked: false
  property int tabIndex: 0
  property real pointSize: Style.fontSizeM
  property bool isFirst: false
  property bool isLast: false

  // Internal state
  property bool isHovered: false

  signal clicked

  // Sizing
  Layout.fillHeight: true
  implicitWidth: contentLayout.implicitWidth + Style.margin2M

  topLeftRadius: isFirst ? Style.iRadiusM : Style.iRadiusXXXS
  bottomLeftRadius: isFirst ? Style.iRadiusM : Style.iRadiusXXXS
  topRightRadius: isLast ? Style.iRadiusM : Style.iRadiusXXXS
  bottomRightRadius: isLast ? Style.iRadiusM : Style.iRadiusXXXS

  color: root.isHovered ? Theme.cHover : (root.checked ? Theme.cPrimary : Theme.smartAlpha(Theme.cSurface))
  border.color: root.checked ? Theme.cPrimary : Theme.cOutline
  border.width: Style.borderS

  Behavior on color {
    enabled: !Theme.isTransitioning
    ColorAnimation {
      duration: Style.animationFast
      easing.type: Easing.OutCubic
    }
  }

  // Content
  RowLayout {
    id: contentLayout
    anchors.centerIn: parent
    width: Math.min(implicitWidth, parent.width - Style.margin2S)
    spacing: (root.icon !== "" && root.text !== "") ? Style.marginXS : 0

    CrawlIcon {
      visible: root.icon !== ""
      Layout.alignment: Qt.AlignVCenter
      icon: root.icon
      pointSize: root.pointSize * 1.2
      color: root.isHovered ? Theme.cOnHover : (root.checked ? Theme.cOnPrimary : Theme.cOnSurface)

      Behavior on color {
        enabled: !Theme.isTransitioning
        ColorAnimation {
          duration: Style.animationFast
          easing.type: Easing.OutCubic
        }
      }
    }

    CrawlText {
      id: tabText
      visible: root.text !== ""
      Layout.alignment: Qt.AlignVCenter
      text: root.text
      pointSize: root.pointSize
      font.weight: Style.fontWeightSemiBold
      color: root.isHovered ? Theme.cOnHover : (root.checked ? Theme.cOnPrimary : Theme.cOnSurface)
      horizontalAlignment: Text.AlignHCenter
      verticalAlignment: Text.AlignVCenter

      Behavior on color {
        enabled: !Theme.isTransitioning
        ColorAnimation {
          duration: Style.animationFast
          easing.type: Easing.OutCubic
        }
      }
    }
  }

  // Tooltip
  Timer {
    id: tooltipTimer
    interval: 500
    onTriggered: {
      if (root.isHovered && root.tooltipText && (!Array.isArray(root.tooltipText) || root.tooltipText.length > 0)) {
        TooltipService.show(root, root.tooltipText);
      }
    }
  }

  MouseArea {
    anchors.fill: parent
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onEntered: {
      root.isHovered = true;
      if (root.tooltipText && (!Array.isArray(root.tooltipText) || root.tooltipText.length > 0)) {
        tooltipTimer.start();
      }
    }
    onExited: {
      root.isHovered = false;
      tooltipTimer.stop();
      if (root.tooltipText && (!Array.isArray(root.tooltipText) || root.tooltipText.length > 0)) {
        TooltipService.hide();
      }
    }
    onClicked: {
      root.clicked();
      // Update parent CrawlTabBar's currentIndex
      if (root.parent && root.parent.parent && root.parent.parent.currentIndex !== undefined) {
        root.parent.parent.currentIndex = root.tabIndex;
      }
    }
  }
}
