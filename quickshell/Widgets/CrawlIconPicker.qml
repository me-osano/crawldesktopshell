import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import qs.Common
import qs.Widgets

Popup {
  id: root
  modal: true

  property string selectedIcon: ""
  property string initialIcon: ""

  signal iconSelected(string iconName)

  width: Math.round(900 * Style.uiScaleRatio)
  height: Math.round(700 * Style.uiScaleRatio)
  anchors.centerIn: Overlay.overlay
  padding: Style.marginXL

  property string query: ""
  property var allIcons: Object.keys(Icons.icons)
  property var filteredIcons: {
    if (query === "")
      return allIcons;
    var q = query.toLowerCase();
    return allIcons.filter(name => name.toLowerCase().includes(q));
  }
  readonly property int columns: 6
  readonly property int cellW: Math.floor(grid.width / columns)
  readonly property int cellH: Math.round(cellW * 0.7 + 36)

  onOpened: {
    selectedIcon = initialIcon;
    query = initialIcon;
    searchInput.forceActiveFocus();
  }

  background: Rectangle {
    color: Theme.cSurface
    radius: Style.iRadiusL
    border.color: Theme.cPrimary
    border.width: Style.borderM
  }

  ColumnLayout {
    anchors.fill: parent
    spacing: Style.marginM

    // Title row
    RowLayout {
      Layout.fillWidth: true
      CrawlText {
        text: "Icon picker"
        pointSize: Style.fontSizeL
        font.weight: Style.fontWeightBold
        color: Theme.cPrimary
        Layout.fillWidth: true
      }
      CrawlIconButton {
        icon: "close"
        tooltipText: "Close"
        onClicked: root.close()
      }
    }

    CrawlDivider {
      Layout.fillWidth: true
    }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginS
      CrawlTextInput {
        id: searchInput
        Layout.fillWidth: true
        label: "Search"
        placeholderText: "e.g. crawlds, niri, battery, cloud"
        text: root.query
        onTextChanged: root.query = text.trim().toLowerCase()
      }
    }

    // Icon grid
    CrawlGridView {
      id: grid
      Layout.fillWidth: true
      Layout.fillHeight: true
      Layout.margins: Style.marginM
      cellWidth: root.cellW
      cellHeight: root.cellH
      model: root.filteredIcons
      reserveScrollbarSpace: false
      gradientColor: Theme.cSurface

      delegate: Rectangle {
        width: grid.cellWidth
        height: grid.cellHeight
        radius: Style.iRadiusS

        color: (root.selectedIcon === modelData) ? Qt.alpha(Theme.cPrimary, 0.15) : "transparent"
        border.color: (root.selectedIcon === modelData) ? Theme.cPrimary : "transparent"
        border.width: (root.selectedIcon === modelData) ? Style.borderS : 0

        MouseArea {
          anchors.fill: parent
          onClicked: root.selectedIcon = modelData
          onDoubleClicked: {
            root.selectedIcon = modelData;
            root.iconSelected(root.selectedIcon);
            root.close();
          }
        }

        ColumnLayout {
          anchors.fill: parent
          anchors.margins: Style.marginS
          spacing: Style.marginS
          Item {
            Layout.fillHeight: true
            Layout.preferredHeight: 4
          }
          CrawlIcon {
            Layout.alignment: Qt.AlignHCenter
            icon: modelData
            pointSize: 42
          }
          CrawlText {
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
            Layout.topMargin: Style.marginXS
            elide: Text.ElideRight
            wrapMode: Text.NoWrap
            maximumLineCount: 1
            horizontalAlignment: Text.AlignHCenter
            color: Theme.cOnSurfaceVariant
            pointSize: Style.fontSizeXS
            text: modelData
          }
          Item {
            Layout.fillHeight: true
          }
        }
      }
    }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginM
      Item {
        Layout.fillWidth: true
      }
      CrawlButton {
        text: "Cancel"
        outlined: true
        onClicked: root.close()
      }
      CrawlButton {
        text: "Apply"
        icon: "check"
        enabled: root.selectedIcon !== ""
        onClicked: {
          root.iconSelected(root.selectedIcon);
          root.close();
        }
      }
    }
  }
}
