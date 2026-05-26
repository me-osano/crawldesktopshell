import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Widgets

Item {
  id: root

  signal search(string query)
  signal cleared()

  property alias text: searchInput.text
  property string placeholder: "Search mail..."

  RowLayout {
    anchors.fill: parent
    spacing: Style.marginS

    CrawlBox {
      Layout.fillWidth: true
      Layout.fillHeight: true
      implicitHeight: searchRow.implicitHeight + Style.marginS

      RowLayout {
        id: searchRow
        anchors.fill: parent
        anchors.margins: Style.marginXS
        spacing: Style.marginS

        CrawlIcon {
          icon: "search"
          pointSize: Style.fontSizeS
          color: Theme.cOnSurfaceVariant
        }

        TextField {
          id: searchInput
          Layout.fillWidth: true
          placeholderText: root.placeholder
          color: Theme.cOnSurface
          background: null
          leftPadding: 0
          topPadding: 2
          bottomPadding: 2

          onAccepted: root.search(text)
          onTextChanged: {
            if (text.length === 0) root.cleared()
          }
        }

        CrawlIconButton {
          icon: "close"
          baseSize: Style.baseWidgetSize * 0.5
          visible: searchInput.text.length > 0
          onClicked: {
            searchInput.text = ""
            root.cleared()
          }
        }
      }
    }

    CrawlIconButton {
      icon: "search"
      tooltipText: "Search"
      baseSize: Style.baseWidgetSize * 0.7
      onClicked: root.search(searchInput.text)
    }
  }
}
