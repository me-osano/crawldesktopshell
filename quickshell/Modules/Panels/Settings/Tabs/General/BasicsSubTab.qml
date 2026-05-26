import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import "../../../../../Common/Helpers/QtObj2JS.js" as QtObj2JS
import qs.Common
import qs.Services
import qs.Widgets

ColumnLayout {
  id: root

  // Profile section
  RowLayout {
    Layout.fillWidth: true
    spacing: Style.marginL

    // Avatar preview
    CrawlImageRounded {
      Layout.preferredWidth: 128 * Style.uiScaleRatio
      Layout.preferredHeight: width
      radius: width / 2
      imagePath: Settings.preprocessPath(Settings.data.general.avatarImage)
      fallbackIcon: "person"
      borderColor: Theme.cPrimary
      borderWidth: Style.borderM
      Layout.alignment: Qt.AlignTop
    }

    ColumnLayout {
      CrawlText {
        text: HostService.displayName
        pointSize: Style.fontSizeM
        color: Theme.cPrimary
      }

      CrawlTextInputButton {
        label: "Profile picture"
        description: "Your profile picture that appears throughout the interface."
        text: Settings.data.general.avatarImage
        placeholderText: '~/.face' // don't translate path
        buttonIcon: "photo"
        buttonTooltip: "Profile picture"
        onInputEditingFinished: Settings.data.general.avatarImage = text
        onButtonClicked: {
          avatarPicker.openFilePicker();
        }
      }
    }
  }

  CrawlFilePicker {
    id: avatarPicker
    title: "Select avatar image"
    selectionMode: "files"
    initialPath: Settings.preprocessPath(Settings.data.general.avatarImage).substr(0, Settings.preprocessPath(Settings.data.general.avatarImage).lastIndexOf("/")) || Quickshell.env("HOME")
    nameFilters: ImageCacheService.basicImageFilters
    onAccepted: paths => {
                  if (paths.length > 0) {
                    Settings.data.general.avatarImage = paths[0];
                  }
                }
  }

  CrawlDivider {
    Layout.fillWidth: true
    Layout.topMargin: Style.marginM
    Layout.bottomMargin: Style.marginM
  }

  // Fonts
  ColumnLayout {
    spacing: Style.marginL
    Layout.fillWidth: true

    // Font configuration section
    ColumnLayout {
      spacing: Style.marginL
      Layout.fillWidth: true

      CrawlSearchableComboBox {
        label: "Default font"
        description: "Main font used throughout the interface."
        model: FontService.availableFonts
        currentKey: Settings.data.ui.fontDefault
        placeholder: "Select default font..."
        searchPlaceholder: "Search font..."
        popupHeight: 420
        defaultValue: Settings.getDefaultValue("ui.fontDefault")
        settingsPath: "ui.fontDefault"
        onSelected: key => Settings.data.ui.fontDefault = key
      }

      CrawlSearchableComboBox {
        label: "Monospaced font"
        description: "Monospaced font used for numbers and stats display."
        model: FontService.monospaceFonts
        currentKey: Settings.data.ui.fontFixed
        placeholder: "Select monospace font..."
        searchPlaceholder: "Search monospace font..."
        popupHeight: 320
        defaultValue: Settings.getDefaultValue("ui.fontFixed")
        settingsPath: "ui.fontFixed"
        onSelected: key => Settings.data.ui.fontFixed = key
      }

      CrawlValueSlider {
        Layout.fillWidth: true
        label: "Default font size"
        description: "Increase or decrease the size of the standard text."
        from: 0.75
        to: 1.25
        stepSize: 0.01
        showReset: true
        value: Settings.data.ui.fontDefaultScale
        defaultValue: Settings.getDefaultValue("ui.fontDefaultScale")
        onMoved: value => Settings.data.ui.fontDefaultScale = value
        text: Math.floor(Settings.data.ui.fontDefaultScale * 100) + "%"
      }

      CrawlValueSlider {
        Layout.fillWidth: true
        label: "Monospaced font size"
        description: "Increase or decrease the size of the monospaced text."
        from: 0.75
        to: 1.25
        stepSize: 0.01
        showReset: true
        value: Settings.data.ui.fontFixedScale
        defaultValue: Settings.getDefaultValue("ui.fontFixedScale")
        onMoved: value => Settings.data.ui.fontFixedScale = value
        text: Math.floor(Settings.data.ui.fontFixedScale * 100) + "%"
      }
    }
  }

  CrawlDivider {
    Layout.fillWidth: true
    Layout.topMargin: Style.marginM
    Layout.bottomMargin: Style.marginM
  }

  CrawlToggle {
    Layout.fillWidth: true
    label: "Reverse scrolling"
    description: "Reverse the interpreted scroll direction"
    checked: Settings.data.general.reverseScroll
    defaultValue: Settings.getDefaultValue("general.reverseScroll")
    onToggled: checked => Settings.data.general.reverseScroll = checked
  }

  CrawlDivider {
    Layout.fillWidth: true
    Layout.topMargin: Style.marginM
    Layout.bottomMargin: Style.marginM
  }

  RowLayout {
    spacing: Style.marginL
    Layout.fillWidth: true

    CrawlButton {
      icon: "external-link"
      text: "Documentation"
      outlined: true
      Layout.fillWidth: true
      onClicked: {
        Qt.openUrlExternally("https://docs.crawlds.dev");
      }
    }

    CrawlButton {
      icon: "json"
      text: "Copy settings"
      outlined: true
      Layout.fillWidth: true
      onClicked: {
        var plainData = QtObj2JS.qtObjectToPlainObject(Settings.data);
        var json = JSON.stringify(plainData, null, 2);
        Quickshell.execDetached(["wl-copy", json]);
        ToastService.showNotice("Settings copied to clipboard");
      }
    }
  }
}
