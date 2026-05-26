import QtQuick
import QtQuick.Layouts
import qs.Common
import qs.Services

RowLayout {
  id: root

  property string label: "Select color"
  property string description: "Apply theme colors for emphasis."
  property string tooltip: ""
  property string currentKey: ""
  property var defaultValue: undefined
  property var noneColor: undefined      // color declared as var so we can nullify
  property var noneOnColor: undefined    // color declared as var so we can nullify

  readonly property bool isValueChanged: (defaultValue !== undefined) && (currentKey !== defaultValue)
  readonly property string indicatorTooltip: {
    "Default: " + defaultValue === "" ? "(empty)" : String(defaultValue);
  }

  readonly property int diameter: Style.baseWidgetSize * 0.9 * Style.uiScaleRatio

  signal selected(string key)

  CrawlLabel {
    label: root.label
    description: root.description
    showIndicator: root.isValueChanged
    indicatorTooltip: root.indicatorTooltip
  }

  RowLayout {
    id: colourRow

    opacity: enabled ? 1.0 : 0.6
    Layout.minimumWidth: root.diameter * Color.colorKeyModel.length

    Repeater {
      model: Color.colorKeyModel

      Rectangle {
        id: colorCircle

        property bool isSelected: root.currentKey === modelData.key
        property bool isHovered: circleMouseArea.containsMouse

        Layout.alignment: Qt.AlignHCenter
        implicitWidth: root.diameter
        implicitHeight: root.diameter
        radius: root.diameter * 0.5
        color: (modelData.key === "none" && root.noneColor !== undefined) ? root.noneColor : Theme.resolveColorKey(modelData.key)
        border.color: (isSelected || isHovered) ? Theme.cOnSurface : Theme.cOutline
        border.width: Style.borderM

        MouseArea {
          id: circleMouseArea

          anchors.fill: parent
          hoverEnabled: true
          cursorShape: Qt.PointingHandCursor
          onEntered: TooltipService.show(parent, modelData.name)
          onExited: TooltipService.hide()
          onClicked: {
            root.currentKey = modelData.key;
            root.selected(modelData.key);
          }
        }

        CrawlIcon {
          anchors.centerIn: parent
          icon: "check"
          pointSize: Math.max(Style.fontSizeXS, colorCircle.width * 0.4)
          color: (modelData.key === "none" && root.noneOnColor !== undefined) ? root.noneOnColor : Theme.resolveOnColorKey(modelData.key)
          font.weight: Style.fontWeightBold
          visible: colorCircle.isSelected
        }

        Behavior on border.color {
          ColorAnimation {
            duration: Style.animationFast
          }
        }
      }
    }
  }
}
