import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  // Preferred player
  CrawlTextInput {
    label: "Primary player"
    description: "Enter a keyword to identify your main player."
    placeholderText: "e.g. spotify, vlc, mpv"
    text: Settings.data.audio.preferredPlayer
    defaultValue: Settings.getDefaultValue("audio.preferredPlayer")
    onTextChanged: {
      Settings.data.audio.preferredPlayer = text;
      MediaService.updateCurrentPlayer();
    }
  }

  // Blacklist editor
  ColumnLayout {
    spacing: Style.marginS
    Layout.fillWidth: true

    CrawlTextInputButton {
      id: blacklistInput
      label: "Excluded player"
      description: "Add keywords for players you want the system to ignore. Each keyword should be on a new line."
      placeholderText: "type substring and press +"
      buttonIcon: "add"
      Layout.fillWidth: true
      onButtonClicked: {
        const val = (blacklistInput.text || "").trim();
        if (val !== "") {
          const arr = (Settings.data.audio.mprisBlacklist || []);
          if (!arr.find(x => String(x).toLowerCase() === val.toLowerCase())) {
            Settings.data.audio.mprisBlacklist = [...arr, val];
            blacklistInput.text = "";
            MediaService.updateCurrentPlayer();
          }
        }
      }
    }

    // Current blacklist entries
    Flow {
      Layout.fillWidth: true
      Layout.leftMargin: Style.marginS
      spacing: Style.marginS

      Repeater {
        model: Settings.data.audio.mprisBlacklist
        delegate: Rectangle {
          required property string modelData
          property real pad: Style.marginS
          color: Qt.alpha(Theme.cOnSurface, 0.125)
          border.color: Qt.alpha(Theme.cOnSurface, Style.opacityLight)
          border.width: Style.borderS

          RowLayout {
            id: chipRow
            spacing: Style.marginXS
            anchors.fill: parent
            anchors.margins: pad

            CrawlText {
              text: modelData
              color: Theme.cOnSurface
              pointSize: Style.fontSizeS
              Layout.alignment: Qt.AlignVCenter
              Layout.leftMargin: Style.marginS
            }

            CrawlIconButton {
              icon: "close"
              baseSize: Style.baseWidgetSize * 0.8
              Layout.alignment: Qt.AlignVCenter
              Layout.rightMargin: Style.marginXS
              onClicked: {
                const arr = (Settings.data.audio.mprisBlacklist || []);
                const idx = arr.findIndex(x => String(x) === modelData);
                if (idx >= 0) {
                  arr.splice(idx, 1);
                  Settings.data.audio.mprisBlacklist = arr;
                  MediaService.updateCurrentPlayer();
                }
              }
            }
          }

          implicitWidth: chipRow.implicitWidth + pad * 2
          implicitHeight: Math.max(chipRow.implicitHeight + pad * 2, Style.baseWidgetSize * 0.8)
          radius: Style.radiusM
        }
      }
    }
  }
}
