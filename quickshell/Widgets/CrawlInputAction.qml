import QtQuick
import QtQuick.Layouts
import qs.Common
import qs.Widgets

// Input and button row
RowLayout {
  id: root

  // Public properties
  property string label: ""
  property string description: ""
  property string placeholderText: ""
  property string text: ""
  property string actionButtonText: "Test"
  property string actionButtonIcon: "media-play"
  property bool actionButtonEnabled: text !== ""

  // Signals
  signal editingFinished
  signal actionClicked

  // Internal properties
  spacing: Style.marginM

  CrawlTextInput {
    id: textInput
    label: root.label
    description: root.description
    placeholderText: root.placeholderText
    text: root.text
    onTextChanged: root.text = text
    onEditingFinished: root.editingFinished()
    Layout.fillWidth: true
  }

  CrawlButton {
    Layout.fillWidth: false
    Layout.alignment: Qt.AlignBottom

    text: root.actionButtonText
    icon: root.actionButtonIcon
    backgroundColor: Theme.cSecondary
    textColor: Theme.cOnSecondary
    hoverColor: Theme.cHover
    enabled: root.actionButtonEnabled

    onClicked: {
      root.actionClicked();
    }
  }
}
