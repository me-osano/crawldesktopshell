import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginM

  // Properties to receive data from parent
  property var screen: null
  property var widgetData: null
  property var widgetMetadata: null

  signal settingsChanged(var settings)

  // Local state
  property string valueHideMode: "hidden" // Default to 'Hide When Empty'
  // Deprecated: hideWhenIdle now folded into hideMode = "idle"
  property bool valueHideWhenIdle: (widgetData && widgetData.hideWhenIdle !== undefined) ? widgetData.hideWhenIdle : (widgetMetadata && widgetMetadata.hideWhenIdle !== undefined ? widgetMetadata.hideWhenIdle : false)
  property bool valueShowAlbumArt: (widgetData && widgetData.showAlbumArt !== undefined) ? widgetData.showAlbumArt : (widgetMetadata && widgetMetadata.showAlbumArt !== undefined ? widgetMetadata.showAlbumArt : false)
  property bool valuePanelShowAlbumArt: (widgetData && widgetData.panelShowAlbumArt !== undefined) ? widgetData.panelShowAlbumArt : (widgetMetadata && widgetMetadata.panelShowAlbumArt !== undefined ? widgetMetadata.panelShowAlbumArt : true)
  property bool valueShowArtistFirst: (widgetData && widgetData.showArtistFirst !== undefined) ? widgetData.showArtistFirst : (widgetMetadata && widgetMetadata.showArtistFirst !== undefined ? widgetMetadata.showArtistFirst : true)
  property bool valueShowVisualizer: (widgetData && widgetData.showVisualizer !== undefined) ? widgetData.showVisualizer : (widgetMetadata && widgetMetadata.showVisualizer !== undefined ? widgetMetadata.showVisualizer : false)
  property string valueVisualizerType: (widgetData && widgetData.visualizerType) || (widgetMetadata && widgetMetadata.visualizerType) || "linear"
  property string valueScrollingMode: (widgetData && widgetData.scrollingMode) || (widgetMetadata && widgetMetadata.scrollingMode) || "hover"
  property int valueMaxWidth: (widgetData && widgetData.maxWidth !== undefined) ? widgetData.maxWidth : (widgetMetadata && widgetMetadata.maxWidth !== undefined ? widgetMetadata.maxWidth : 145)
  property bool valueUseFixedWidth: (widgetData && widgetData.useFixedWidth !== undefined) ? widgetData.useFixedWidth : (widgetMetadata && widgetMetadata.useFixedWidth !== undefined ? widgetMetadata.useFixedWidth : false)
  property bool valueShowProgressRing: (widgetData && widgetData.showProgressRing !== undefined) ? widgetData.showProgressRing : (widgetMetadata && widgetMetadata.showProgressRing !== undefined ? widgetMetadata.showProgressRing : true)
  property bool valueCompactMode: widgetData.compactMode !== undefined ? widgetData.compactMode : widgetMetadata.compactMode
  property string valueTextColor: (widgetData && widgetData.textColor !== undefined) ? widgetData.textColor : (widgetMetadata && widgetMetadata.textColor !== undefined ? widgetMetadata.textColor : "none")

  Component.onCompleted: {
    if (widgetData && widgetData.hideMode !== undefined) {
      valueHideMode = widgetData.hideMode;
    }
  }

  function saveSettings() {
    var settings = Object.assign({}, widgetData || {});
    settings.hideMode = valueHideMode;
    // No longer store hideWhenIdle separately; kept for backward compatibility only
    settings.showAlbumArt = valueShowAlbumArt;
    settings.panelShowAlbumArt = valuePanelShowAlbumArt;
    settings.showArtistFirst = valueShowArtistFirst;
    settings.showVisualizer = valueShowVisualizer;
    settings.visualizerType = valueVisualizerType;
    settings.scrollingMode = valueScrollingMode;
    settings.maxWidth = parseInt(widthInput.text) || widgetMetadata.maxWidth;
    settings.useFixedWidth = valueUseFixedWidth;
    settings.showProgressRing = valueShowProgressRing;
    settings.compactMode = valueCompactMode;
    settings.textColor = valueTextColor;
    settingsChanged(settings);
  }

  CrawlComboBox {
    Layout.fillWidth: true
    label: "Hiding mode"
    description: "Controls how the widget behaves when no media is playing."
    model: [
      {
        "key": "visible",
        "name": "Always visible"
      },
      {
        "key": "hidden",
        "name": "Hide when empty"
      },
      {
        "key": "transparent",
        "name": "Transparent when empty"
      },
      {
        "key": "idle",
        "name": "Hide when idle"
      }
    ]
    currentKey: root.valueHideMode
    onSelected: key => {
                  root.valueHideMode = key;
                  saveSettings();
                }
  }

  CrawlToggle {
    label: "Show album art"
    description: "Display the album artwork for the currently playing track."
    checked: valueShowAlbumArt
    onToggled: checked => {
                 valueShowAlbumArt = checked;
                 saveSettings();
               }
  }

  CrawlToggle {
    label: "Show artist first"
    description: "Display artist - title instead of title - artist."
    checked: valueShowArtistFirst
    onToggled: checked => {
                 valueShowArtistFirst = checked;
                 saveSettings();
               }
  }

  CrawlToggle {
    label: "Show visualizer"
    description: "Display an audio visualizer when music is playing."
    checked: valueShowVisualizer
    onToggled: checked => {
                 valueShowVisualizer = checked;
                 saveSettings();
               }
  }

  CrawlComboBox {
    visible: valueShowVisualizer
    label: "Visualizer type"
    description: "Choose the style of audio visualizer to display."
    model: [
      {
        "key": "linear",
        "name": "Linear"
      },
      {
        "key": "mirrored",
        "name": "Mirrored"
      },
      {
        "key": "wave",
        "name": "Wave"
      }
    ]
    currentKey: valueVisualizerType
    onSelected: key => {
                  valueVisualizerType = key;
                  saveSettings();
                }
    minimumWidth: 200
  }

  CrawlTextInput {
    id: widthInput
    Layout.fillWidth: true
    label: "Maximum width"
    description: "Sets the maximum horizontal size of the widget. The widget will shrink to fit shorter content."
    placeholderText: widgetMetadata.maxWidth
    text: valueMaxWidth
    onEditingFinished: saveSettings()
  }

  CrawlToggle {
    label: "Use fixed width"
    description: "When enabled, the widget will always use the maximum width instead of dynamically adjusting to content."
    checked: valueUseFixedWidth
    onToggled: checked => {
                 valueUseFixedWidth = checked;
                 saveSettings();
               }
  }

  CrawlToggle {
    label: "Show progress ring"
    description: "Display a circular progress indicator showing track progress."
    checked: valueShowProgressRing
    onToggled: checked => {
                 valueShowProgressRing = checked;
                 saveSettings();
               }
  }

  CrawlColorChoice {
    currentKey: valueTextColor
    onSelected: key => {
                  valueTextColor = key;
                  saveSettings();
                }
  }

  CrawlComboBox {
    label: "Scrolling mode"
    description: "Control when text scrolling is enabled for long track titles."
    model: [
      {
        "key": "always",
        "name": "Scroll always"
      },
      {
        "key": "hover",
        "name": "Scroll on hover"
      },
      {
        "key": "never",
        "name": "Never scroll"
      }
    ]
    currentKey: valueScrollingMode
    onSelected: key => {
                  valueScrollingMode = key;
                  saveSettings();
                }
    minimumWidth: 200
  }

  CrawlDivider {
    Layout.fillWidth: true
    Layout.topMargin: Style.marginS
  }

  CrawlLabel {
    label: "Media player panel"
    description: "Configure the appearance and behavior of the media player panel."
    labelColor: Theme.cPrimary
  }

  CrawlToggle {
    label: "Show album art"
    description: "Display the album artwork for the currently playing track."
    checked: valuePanelShowAlbumArt
    onToggled: checked => {
                 valuePanelShowAlbumArt = checked;
                 saveSettings();
               }
  }

  CrawlToggle {
    label: "Compact mode"
    description: "Enable a space-saving layout for the media player panel."
    checked: valueCompactMode
    onToggled: checked => {
                 valueCompactMode = checked;
                 saveSettings();
               }
  }
}
