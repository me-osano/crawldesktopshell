import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Services
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true

  property var timeOptions
  property var schemeColorsCache: ({})
  property int cacheVersion: 0
  property var screen

  signal openDownloadPopup

  function extractSchemeName(schemePath) {
    var pathParts = schemePath.split("/");
    var filename = pathParts[pathParts.length - 1];
    var schemeName = filename.replace(".json", "");

    if (schemeName === "Crawlds") {
      schemeName = "Crawlds (default)";
    } else if (schemeName === "Crawlds-legacy") {
      schemeName = "CrawlDS (legacy)";
    } else if (schemeName === "Tokyo-Night") {
      schemeName = "Tokyo Night";
    } else if (schemeName === "Rosepine") {
      schemeName = "Rose Pine";
    }

    return schemeName;
  }

  function getSchemeColor(schemeName, colorKey) {
    var _ = cacheVersion;

    if (schemeColorsCache[schemeName]) {
      var entry = schemeColorsCache[schemeName];
      var variant = entry;

      if (entry.dark || entry.light) {
        variant = Settings.data.colorSchemes.darkMode ? (entry.dark || entry.light) : (entry.light || entry.dark);
      }

      if (variant && variant[colorKey]) {
        return variant[colorKey];
      }
    }

    if (colorKey === "cSurface")
      return Theme.cSurfaceVariant;
    if (colorKey === "cPrimary")
      return Theme.cPrimary;
    if (colorKey === "cSecondary")
      return Theme.cSecondary;
    if (colorKey === "cTertiary")
      return Theme.cTertiary;
    if (colorKey === "cError")
      return Theme.cError;
    return Theme.cOnSurfaceVariant;
  }

  function schemeLoaded(schemeName, jsonData) {
    var value = jsonData || {};
    schemeColorsCache[schemeName] = value;
    cacheVersion++;
  }

  Connections {
    target: ThemeService
    function onSchemesChanged() {
      root.schemeColorsCache = {};
      root.cacheVersion++;
    }
  }

  Item {
    id: fileLoaders
    visible: false

    Repeater {
      model: ThemeService.schemes
      delegate: Item {
        FileView {
          path: modelData
          blockLoading: false
          onLoaded: {
            var schemeName = root.extractSchemeName(path);

            try {
              var jsonData = JSON.parse(text());
              root.schemeLoaded(schemeName, jsonData);
            } catch (e) {
              Logger.w("ColorSchemeTab", "Failed to parse JSON for scheme:", schemeName, e);
              root.schemeLoaded(schemeName, null);
            }
          }
        }
      }
    }
  }

  CrawlToggle {
    label: "Dark Mode"
    description: "Switches to a darker theme for easier viewing at night."
    checked: Settings.data.colorSchemes.darkMode
    onToggled: checked => {
                 Settings.data.colorSchemes.darkMode = checked;
                 root.cacheVersion++;
               }
  }

  CrawlComboBox {
    label: "Dark Mode schedule"
    description: "Enables automatic switching between Light and Dark Mode."

    model: [
      {
        "name": "Off",
        "key": "off"
      },
      {
        "name": "Manual",
        "key": "manual"
      },
      {
        "name": "Location",
        "key": "location"
      }
    ]

    currentKey: Settings.data.colorSchemes.schedulingMode

    onSelected: key => {
                  Settings.data.colorSchemes.schedulingMode = key;
                  AppThemeService.generate();
                }
  }

  ColumnLayout {
    spacing: Style.marginS
    visible: Settings.data.colorSchemes.schedulingMode === "manual"

    CrawlLabel {
      label: "Manual scheduling"
      description: "Set custom times for sunrise and sunset."
    }

    RowLayout {
      Layout.fillWidth: false
      spacing: Style.marginS

      CrawlText {
        text: "Sunrise time"
        pointSize: Style.fontSizeM
        color: Theme.cOnSurfaceVariant
      }

      CrawlComboBox {
        model: root.timeOptions
        currentKey: Settings.data.colorSchemes.manualSunrise
        placeholder: "Select start time"
        onSelected: key => Settings.data.colorSchemes.manualSunrise = key
        minimumWidth: 120
      }

      Item {
        Layout.preferredWidth: 20
      }

      CrawlText {
        text: "Sunset time"
        pointSize: Style.fontSizeM
        color: Theme.cOnSurfaceVariant
      }

      CrawlComboBox {
        model: root.timeOptions
        currentKey: Settings.data.colorSchemes.manualSunset
        placeholder: "Select stop time"
        onSelected: key => Settings.data.colorSchemes.manualSunset = key
        minimumWidth: 120
      }
    }
  }

  CrawlDivider {
    Layout.fillWidth: true
  }

  CrawlToggle {
    label: "Use wallpaper colors"
    description: "Generate color schemes from your wallpaper. Automatically extracts colors to create a cohesive theme."
    checked: Settings.data.colorSchemes.useWallpaperColors
    onToggled: checked => {
                 Settings.data.colorSchemes.useWallpaperColors = checked;
                 if (checked) {
                   AppThemeService.generate();
                 } else {
                   ToastService.showNotice("Wallpaper colors", "Wallpaper colors disabled", "settings-color-scheme");
                   if (Settings.data.colorSchemes.predefinedScheme) {
                     ThemeService.applyScheme(Settings.data.colorSchemes.predefinedScheme);
                   }
                 }
               }
  }

  CrawlComboBox {
    Layout.fillWidth: true
    label: "Color generation source"
    description: "Select which monitor to use for extracting wallpaper colors."
    enabled: Settings.data.colorSchemes.useWallpaperColors
    model: {
      var m = [];
      if (Quickshell.screens) {
        for (var i = 0; i < Quickshell.screens.length; i++) {
          var screen = Quickshell.screens[i];
          var name = screen.name;
          var displayName = name + " (" + screen.width + "x" + screen.height + ")";
          m.push({
                   "key": name,
                   "name": displayName
                 });
        }
      }
      return m;
    }
    currentKey: Settings.data.colorSchemes.monitorForColors || (screen ? screen.name : "")
    onSelected: key => {
                  Settings.data.colorSchemes.monitorForColors = key;
                  AppThemeService.generate();
                }
    defaultValue: ""
  }

  CrawlComboBox {
    Layout.fillWidth: true
    label: "Palette generation method"
    description: "Choose your favorite palette generation method."
    enabled: Settings.data.colorSchemes.useWallpaperColors
    model: TemplateProcessor.schemeTypes
    currentKey: Settings.data.colorSchemes.generationMethod
    onSelected: key => {
                  Settings.data.colorSchemes.generationMethod = key;
                  AppThemeService.generate();
                }
  }

  CrawlBox {
    visible: Settings.data.colorSchemes.useWallpaperColors
    Layout.fillWidth: true
    implicitHeight: descriptionColumn.implicitHeight + Style.margin2L
    color: Theme.cSurface

    Column {
      id: descriptionColumn
      anchors.left: parent.left
      anchors.right: parent.right
      anchors.top: parent.top
      anchors.margins: Style.marginL
      spacing: Style.marginM

      CrawlText {
        width: parent.width
        wrapMode: Text.WordWrap
        text: "Color scheme will be generated from the current wallpaper"
        pointSize: Style.fontSizeS
        color: Theme.cOnSurfaceVariant
      }

      Row {
        id: colorPreviewRow
        spacing: Style.marginS

        property int diameter: 16 * Style.uiScaleRatio

        Repeater {
          model: [Theme.cPrimary, Theme.cSecondary, Theme.cTertiary, Theme.cError]

          Rectangle {
            width: colorPreviewRow.diameter
            height: colorPreviewRow.diameter
            radius: width * 0.5
            color: modelData
          }
        }
      }
    }
  }

  CrawlDivider {
    Layout.fillWidth: true
  }

  ColumnLayout {
    spacing: Style.marginM
    Layout.fillWidth: true
    enabled: !Settings.data.colorSchemes.useWallpaperColors

    CrawlHeader {
      label: "Predefined Themes"
      description: "Choose from a collection of predefined color schemes."
      Layout.fillWidth: true
    }

    GridLayout {
      columns: 2
      rowSpacing: Style.marginM
      columnSpacing: Style.marginM
      Layout.fillWidth: true

      Repeater {
        model: ThemeService.schemes

        Rectangle {
          id: schemeItem

          property string schemePath: modelData
          property string schemeName: root.extractSchemeName(modelData)

          opacity: enabled ? 1.0 : 0.6
          Layout.fillWidth: true
          Layout.alignment: Qt.AlignHCenter
          height: 50 * Style.uiScaleRatio
          radius: Style.radiusS
          color: root.getSchemeColor(schemeName, "cSurface")
          border.width: Style.borderL
          border.color: {
            if ((Settings.data.colorSchemes.predefinedScheme === schemeName) && schemeItem.enabled) {
              return Theme.cSecondary;
            }
            if (itemMouseArea.containsMouse) {
              return Theme.cHover;
            }
            return Theme.cOutline;
          }

          RowLayout {
            id: scheme
            anchors.fill: parent
            anchors.margins: Style.marginL
            spacing: Style.marginS

            CrawlText {
              text: schemeItem.schemeName
              pointSize: Style.fontSizeS
              color: Theme.cOnSurface
              Layout.fillWidth: true
              elide: Text.ElideRight
              verticalAlignment: Text.AlignVCenter
              wrapMode: Text.WordWrap
              maximumLineCount: 1
            }

            property int diameter: 16 * Style.uiScaleRatio

            Rectangle {
              width: scheme.diameter
              height: scheme.diameter
              radius: scheme.diameter * 0.5
              color: root.getSchemeColor(schemeItem.schemeName, "cPrimary")
            }

            Rectangle {
              width: scheme.diameter
              height: scheme.diameter
              radius: scheme.diameter * 0.5
              color: root.getSchemeColor(schemeItem.schemeName, "cSecondary")
            }

            Rectangle {
              width: scheme.diameter
              height: scheme.diameter
              radius: scheme.diameter * 0.5
              color: root.getSchemeColor(schemeItem.schemeName, "cTertiary")
            }

            Rectangle {
              width: scheme.diameter
              height: scheme.diameter
              radius: scheme.diameter * 0.5
              color: root.getSchemeColor(schemeItem.schemeName, "cError")
            }
          }

          MouseArea {
            id: itemMouseArea
            anchors.fill: parent
            enabled: schemeItem.enabled
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: {
              Settings.data.colorSchemes.useWallpaperColors = false;
              Logger.i("ColorSchemeTab", "Disabled wallpaper colors");

              Settings.data.colorSchemes.predefinedScheme = schemeItem.schemeName;
              ThemeService.applyScheme(Settings.data.colorSchemes.predefinedScheme);
            }
          }

          Rectangle {
            visible: (Settings.data.colorSchemes.predefinedScheme === schemeItem.schemeName) && schemeItem.enabled
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.rightMargin: 0
            anchors.topMargin: -3
            width: 20
            height: 20
            radius: Math.min(Style.radiusL, width / 2)
            color: Theme.cSecondary
            border.width: Style.borderS
            border.color: Theme.cOnSecondary

            CrawlIcon {
              icon: "check"
              pointSize: Style.fontSizeXS
              color: Theme.cOnSecondary
              anchors.centerIn: parent
            }
          }

          Behavior on border.color {
            ColorAnimation {
              duration: Style.animationNormal
            }
          }
        }
      }
    }

    CrawlButton {
      text: "Download more"
      icon: "download"
      onClicked: root.openDownloadPopup()
      Layout.alignment: Qt.AlignRight
      Layout.topMargin: Style.marginS
    }
  }
}
