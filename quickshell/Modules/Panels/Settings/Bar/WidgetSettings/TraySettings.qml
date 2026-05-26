import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root

  // Properties to receive data from parent
  property var screen: null
  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  // Local state
  property var localBlacklist: widgetData.blacklist || []
  property bool valueColorizeIcons: widgetData.colorizeIcons !== undefined ? widgetData.colorizeIcons : widgetMetadata.colorizeIcons
  property string valueChevronColor: widgetData.chevronColor !== undefined ? widgetData.chevronColor : widgetMetadata.chevronColor
  property bool valueDrawerEnabled: widgetData.drawerEnabled !== undefined ? widgetData.drawerEnabled : widgetMetadata.drawerEnabled
  property bool valueHidePassive: widgetData.hidePassive !== undefined ? widgetData.hidePassive : widgetMetadata.hidePassive

  ListModel {
    id: blacklistModel
  }

  function populateBlacklist() {
    for (var i = 0; i < localBlacklist.length; i++) {
      blacklistModel.append({
                              "rule": localBlacklist[i]
                            });
    }
  }

  Component.onCompleted: {
    Qt.callLater(populateBlacklist);
  }

  spacing: Style.marginM

  CrawlToggle {
    Layout.fillWidth: true
    label: "Enable drawer"
    description: "When enabled, unpinned tray items are shown in a drawer panel.<br>When disabled, all tray items are shown inline."
    checked: root.valueDrawerEnabled
    onToggled: checked => {
                 root.valueDrawerEnabled = checked;
                 saveSettings();
               }
  }

  CrawlColorChoice {
    label: "Chevron color"
    description: "Apply theme colors to the drawer chevron icon."
    currentKey: root.valueChevronColor
    onSelected: key => {
                  root.valueChevronColor = key;
                  saveSettings();
                }
    visible: root.valueDrawerEnabled
  }

  CrawlToggle {
    Layout.fillWidth: true
    label: "Colorize icons"
    description: "Apply theme colors to tray icons."
    checked: root.valueColorizeIcons
    onToggled: checked => {
                 root.valueColorizeIcons = checked;
                 saveSettings();
               }
  }

  CrawlToggle {
    Layout.fillWidth: true
    label: "Hide passive items"
    description: "When enabled, tray items with 'Passive' status will be hidden."
    checked: root.valueHidePassive
    onToggled: checked => {
                 root.valueHidePassive = checked;
                 saveSettings();
               }
  }

  ColumnLayout {
    Layout.fillWidth: true
    spacing: Style.marginS

    CrawlLabel {
      label: "Blacklist"
      description: "Add tray exclusion rules, supports wildcards (*)."
    }

    RowLayout {
      Layout.fillWidth: true
      spacing: Style.marginS

      CrawlTextInputButton {
        id: newRuleInput
        Layout.fillWidth: true
        placeholderText: "e.g. nm-applet, Fcitx*"
        buttonIcon: "add"
        onButtonClicked: {
          if (newRuleInput.text.length > 0) {
            var newRule = newRuleInput.text.trim();
            var exists = false;
            for (var i = 0; i < blacklistModel.count; i++) {
              if (blacklistModel.get(i).rule === newRule) {
                exists = true;
                break;
              }
            }
            if (!exists) {
              blacklistModel.append({
                                      "rule": newRule
                                    });
              newRuleInput.text = "";
              saveSettings();
            }
          }
        }
      }
    }
  }

  // List of current blacklist items
  CrawlListView {
    Layout.fillWidth: true
    Layout.preferredHeight: 150
    Layout.topMargin: Style.marginL // Increased top margin
    gradientColor: Theme.cSurface

    model: blacklistModel
    delegate: Item {
      width: ListView.width
      height: 40

      Rectangle {
        id: itemBackground
        anchors.fill: parent
        anchors.margins: Style.marginXS
        color: "transparent" // Make background transparent
        border.color: Theme.cOutline
        border.width: Style.borderS
        radius: Style.radiusS
        visible: model.rule !== undefined && model.rule !== "" // Only visible if rule exists
      }

      Row {
        anchors.fill: parent
        anchors.leftMargin: Style.marginS
        anchors.rightMargin: Style.marginS
        spacing: Style.marginS

        CrawlText {
          anchors.verticalCenter: parent.verticalCenter
          text: model.rule
          elide: Text.ElideRight
        }

        CrawlIconButton {
          anchors.verticalCenter: parent.verticalCenter
          icon: "close"
          baseSize: 12 * Style.uiScaleRatio
          colorBg: Theme.cSurfaceVariant
          colorFg: Theme.cOnSurfaceVariant
          colorBgHover: Theme.cError
          colorFgHover: Theme.cOnError
          onClicked: {
            blacklistModel.remove(index);
            saveSettings();
          }
        }
      }
    }
  }

  // This function will be called by the dialog to get the new settings
  function saveSettings() {
    var newBlacklist = [];
    for (var i = 0; i < blacklistModel.count; i++) {
      newBlacklist.push(blacklistModel.get(i).rule);
    }

    // Return the updated settings for this widget instance
    var settings = Object.assign({}, widgetData || {});
    settings.blacklist = newBlacklist;
    settings.colorizeIcons = root.valueColorizeIcons;
    settings.chevronColor = root.valueChevronColor;
    settings.drawerEnabled = root.valueDrawerEnabled;
    settings.hidePassive = root.valueHidePassive;
    settingsChanged(settings);
  }
}
