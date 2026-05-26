import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

CrawlCollapsible {
  id: root

  // Public API
  property var tags: []  // Array of tag strings
  property string selectedTag: ""
  property alias label: root.label
  property alias description: root.description
  property alias expanded: root.expanded

  // Formatting function for tag display (optional override)
  property var formatTag: function (tag) {
    if (tag === "")
      return "All";
    // Default: capitalize first letter
    return tag.charAt(0).toUpperCase() + tag.slice(1);
  }

  Layout.fillWidth: true
  contentSpacing: Style.marginXS

  Flow {
    Layout.fillWidth: true
    spacing: Style.marginXS
    flow: Flow.LeftToRight

    Repeater {
      model: [""].concat(root.tags)

      delegate: CrawlButton {
        text: root.formatTag(modelData)
        backgroundColor: root.selectedTag === modelData ? Theme.cPrimary : Theme.cSurfaceVariant
        textColor: root.selectedTag === modelData ? Theme.cOnPrimary : Theme.cOnSurfaceVariant
        onClicked: root.selectedTag = modelData
        fontSize: Style.fontSizeS
        iconSize: Style.fontSizeS
        fontWeight: Style.fontWeightSemiBold
        buttonRadius: Style.iRadiusM
      }
    }
  }
}
