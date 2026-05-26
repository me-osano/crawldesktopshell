import QtQuick
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root

  property string label: ""
  property string description: ""
  property string icon: ""
  property color labelColor: Theme.cOnSurface
  property color descriptionColor: Theme.cOnSurfaceVariant
  property color iconColor: Theme.cOnSurface
  property bool showIndicator: false
  property string indicatorTooltip: ""
  property real labelSize: Style.fontSizeL

  opacity: enabled ? 1.0 : 0.6
  spacing: Style.marginXXS
  visible: root.label != "" || root.description != ""

  Layout.fillWidth: true

  RowLayout {
    spacing: Style.marginXS
    Layout.fillWidth: true
    visible: root.label !== ""

    CrawlIcon {
      visible: root.icon !== ""
      icon: root.icon
      pointSize: Style.fontSizeXXL
      color: root.iconColor
      Layout.rightMargin: Style.marginS
    }

    CrawlText {
      id: labelText
      Layout.fillWidth: true
      text: root.label
      pointSize: root.labelSize
      font.weight: Style.fontWeightSemiBold
      color: labelColor
      wrapMode: Text.WordWrap

      // Settings indicator dot positioned right after the text content
      Loader {
        active: root.showIndicator
        x: labelText.contentWidth + Style.marginXS
        anchors.verticalCenter: parent.verticalCenter
        sourceComponent: CrawlSettingsIndicator {
          show: true
          tooltipText: root.indicatorTooltip || ""
        }
      }
    }
  }

  CrawlText {
    visible: root.description !== ""
    Layout.fillWidth: true
    text: root.description
    pointSize: Style.fontSizeS
    color: root.descriptionColor
    wrapMode: Text.WordWrap
    textFormat: Text.StyledText
  }
}
