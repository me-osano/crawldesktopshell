import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

ColumnLayout {
  id: root
  spacing: Style.marginL
  Layout.fillWidth: true
  enabled: Settings.data.notifications.enabled

  signal openUnifiedPicker
  signal openLowPicker
  signal openNormalPicker
  signal openCriticalPicker

  // QtMultimedia unavailable message
  CrawlBox {
    Layout.fillWidth: true
    visible: !SoundService.multimediaAvailable
    implicitHeight: unavailableContent.implicitHeight + Style.margin2L

    RowLayout {
      id: unavailableContent
      anchors.fill: parent
      anchors.margins: Style.marginL
      spacing: Style.marginM

      CrawlIcon {
        icon: "warning"
        color: Theme.cOnSurfaceVariant
        pointSize: Style.fontSizeXL
        Layout.alignment: Qt.AlignVCenter
      }

      CrawlLabel {
        Layout.fillWidth: true
        label: "Notification sounds unavailable"
        description: "Install Qt6 Multimedia to enable notification sounds."
      }
    }
  }

  CrawlToggle {
    enabled: SoundService.multimediaAvailable
    label: "Enable notification sounds"
    description: "Enable sound effects for incoming notifications."
    checked: Settings.data.notifications?.sounds?.enabled ?? false
    onToggled: checked => Settings.data.notifications.sounds.enabled = checked
    defaultValue: Settings.getDefaultValue("notifications.sounds.enabled")
  }

  // Sound Volume
  CrawlValueSlider {
    enabled: SoundService.multimediaAvailable && (Settings.data.notifications?.sounds?.enabled ?? false)
    Layout.fillWidth: true
    label: "Sound volume"
    description: "Adjust the volume level for notification sounds."
    from: 0
    to: 1
    stepSize: 0.01
    showReset: true
    value: Settings.data.notifications?.sounds?.volume ?? 0.5
    onMoved: value => Settings.data.notifications.sounds.volume = value
    text: Math.round((Settings.data.notifications?.sounds?.volume ?? 0.5) * 100) + "%"
    defaultValue: Settings.getDefaultValue("notifications.sounds.volume")
  }

  // Separate Sounds Toggle
  CrawlToggle {
    enabled: SoundService.multimediaAvailable && (Settings.data.notifications?.sounds?.enabled ?? false)
    Layout.fillWidth: true
    label: "Use different sounds per priority"
    description: "Use different sound files for low, normal, and critical priority notifications."
    checked: Settings.data.notifications?.sounds?.separateSounds ?? false
    onToggled: checked => Settings.data.notifications.sounds.separateSounds = checked
    defaultValue: Settings.getDefaultValue("notifications.sounds.separateSounds")
  }

  // Unified Sound File (shown when separateSounds is false)
  ColumnLayout {
    enabled: SoundService.multimediaAvailable && (Settings.data.notifications?.sounds?.enabled ?? false)
    visible: !(Settings.data.notifications?.sounds?.separateSounds ?? false)
    spacing: Style.marginXXS
    Layout.fillWidth: true

    CrawlLabel {
      label: "Notification sound"
      description: "Path to the sound file played for notifications."
    }

    CrawlTextInputButton {
      enabled: parent.enabled
      Layout.fillWidth: true
      placeholderText: "Enter path to sound file"
      text: Settings.data.notifications?.sounds?.normalSoundFile ?? ""
      buttonIcon: "folder-open"
      buttonTooltip: "Select sound file"
      onInputEditingFinished: {
        const soundPath = text;
        Settings.data.notifications.sounds.normalSoundFile = soundPath;
        Settings.data.notifications.sounds.lowSoundFile = soundPath;
        Settings.data.notifications.sounds.criticalSoundFile = soundPath;
      }
      onButtonClicked: root.openUnifiedPicker()
    }
  }

  // Separate Sound Files (shown when separateSounds is true)
  ColumnLayout {
    visible: SoundService.multimediaAvailable && (Settings.data.notifications?.sounds?.enabled ?? false) && (Settings.data.notifications?.sounds?.separateSounds ?? false)
    spacing: Style.marginXXS
    Layout.fillWidth: true

    // Low Urgency Sound File
    ColumnLayout {
      spacing: Style.marginXXS
      Layout.fillWidth: true

      CrawlLabel {
        label: "Low urgency sound"
        description: "Path to the sound file played for low priority notifications."
      }

      CrawlTextInputButton {
        enabled: parent.enabled
        Layout.fillWidth: true
        placeholderText: "Enter path to sound file"
        text: Settings.data.notifications?.sounds?.lowSoundFile ?? ""
        buttonIcon: "folder-open"
        buttonTooltip: "Select sound file"
        onInputEditingFinished: Settings.data.notifications.sounds.lowSoundFile = text
        onButtonClicked: root.openLowPicker()
      }
    }

    // Normal Urgency Sound File
    ColumnLayout {
      spacing: Style.marginXXS
      Layout.fillWidth: true

      CrawlLabel {
        label: "Normal urgency sound"
        description: "Path to the sound file played for normal priority notifications."
      }

      CrawlTextInputButton {
        enabled: parent.enabled
        Layout.fillWidth: true
        placeholderText: "Enter path to sound file"
        text: Settings.data.notifications?.sounds?.normalSoundFile ?? ""
        buttonIcon: "folder-open"
        buttonTooltip: "Select sound file"
        onInputEditingFinished: Settings.data.notifications.sounds.normalSoundFile = text
        onButtonClicked: root.openNormalPicker()
      }
    }

    // Critical Urgency Sound File
    ColumnLayout {
      spacing: Style.marginXXS
      Layout.fillWidth: true

      CrawlLabel {
        label: "Critical urgency sound"
        description: "Path to the sound file played for critical priority notifications."
      }

      CrawlTextInputButton {
        enabled: parent.enabled
        Layout.fillWidth: true
        placeholderText: "Enter path to sound file"
        text: Settings.data.notifications?.sounds?.criticalSoundFile ?? ""
        buttonIcon: "folder-open"
        buttonTooltip: "Select sound file"
        onInputEditingFinished: Settings.data.notifications.sounds.criticalSoundFile = text
        onButtonClicked: root.openCriticalPicker()
      }
    }
  }

  // Excluded Apps List
  ColumnLayout {
    enabled: SoundService.multimediaAvailable && (Settings.data.notifications?.sounds?.enabled ?? false)
    spacing: Style.marginXXS
    Layout.fillWidth: true

    CrawlLabel {
      label: "Excluded applications"
      description: "Skip playing the configured notification sound for specific applications that have their own built-in sounds."
    }

    CrawlTextInput {
      enabled: parent.enabled
      Layout.fillWidth: true
      placeholderText: "discord,firefox,chrome,chromium,edge"
      text: Settings.data.notifications?.sounds?.excludedApps ?? ""
      onEditingFinished: Settings.data.notifications.sounds.excludedApps = text
    }
  }
}
