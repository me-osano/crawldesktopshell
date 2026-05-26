import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: root
  anchors.fill: parent

  required property var lockControl
  required property var batteryIndicator
  required property var keyboardLayout
  required property TextInput passwordInput

  // Whether to enable lock screen animations (smooth cursor blink).
  // Defaults to false to reduce GPU usage.  Set Settings.data.general.lockScreenAnimations = true to restore.
  readonly property bool animationsEnabled: Settings.data.general.lockScreenAnimations || false

  Component.onCompleted: {
    if (Settings.data.general.autoStartAuth) {
      doUnlock();
    }
  }

  function doUnlock() {
    if (lockControl) {
      lockControl.tryUnlock();
    }
  }

  // Timer properties
  readonly property int timerDuration: Settings.data.general.lockScreenCountdownDuration
  property string pendingAction: ""
  property bool timerActive: false
  property int timeRemaining: 0
  readonly property bool weatherReady: Settings.data.location.weatherEnabled && (LocationService.data.weather !== null)

  // Timer management functions
  function startTimer(action) {
    // Check if global countdown is disabled
    if (!Settings.data.general.enableLockScreenCountdown) {
      executeAction(action);
      return;
    }

    if (timerActive && pendingAction === action) {
      // Second click - execute immediately
      executeAction(action);
      return;
    }

    pendingAction = action;
    timeRemaining = timerDuration;
    timerActive = true;
    countdownTimer.start();
  }

  function cancelTimer() {
    timerActive = false;
    pendingAction = "";
    timeRemaining = 0;
    countdownTimer.stop();
  }

  function executeAction(action) {
    // Stop timer but don't reset other properties yet
    countdownTimer.stop();

    // Execute the action
    switch (action) {
    case "logout":
      CompositorService.logout();
      break;
    case "suspend":
      CompositorService.suspend();
      break;
    case "hibernate":
      CompositorService.hibernate();
      break;
    case "reboot":
      CompositorService.reboot();
      break;
    case "shutdown":
      CompositorService.shutdown();
      break;
    }

    // Reset timer state
    cancelTimer();
  }

  // Countdown timer
  Timer {
    id: countdownTimer
    interval: 100
    repeat: true
    onTriggered: {
      timeRemaining -= interval;
      if (timeRemaining <= 0) {
        executeAction(pendingAction);
      }
    }
  }

  // Compact status indicators container (compact mode only)
  Rectangle {
    width: {
      var hasBattery = batteryIndicator.isReady;
      var hasKeyboard = keyboardLayout.currentLayout !== "Unknown";
      var hasCaps = LockKeysService.capsLockOn;
      var hasCapsSlot = hasBattery || hasKeyboard || hasCaps;

      var visibleCount = 0;
      if (hasBattery)
        visibleCount++;
      if (hasKeyboard)
        visibleCount++;
      if (hasCapsSlot)
        visibleCount++;

      if (visibleCount >= 3) {
        return 280;
      } else if (visibleCount === 2) {
        return 200;
      } else if (visibleCount === 1) {
        return 120;
      } else {
        return 0;
      }
    }
    height: 40
    anchors.horizontalCenter: parent.horizontalCenter
    anchors.bottom: parent.bottom
    anchors.bottomMargin: 96 + (Settings.data.general.compactLockScreen ? 116 : 220)
    topLeftRadius: Style.radiusL
    topRightRadius: Style.radiusL
    color: Theme.cSurface
    visible: Settings.data.general.compactLockScreen && (batteryIndicator.isReady || keyboardLayout.currentLayout !== "Unknown" || LockKeysService.capsLockOn)

    RowLayout {
      id: compactStatusRow
      anchors.centerIn: parent
      spacing: Style.marginL

      // Battery indicator
      RowLayout {
        spacing: Style.marginS
        visible: batteryIndicator.isReady

        CrawlIcon {
          icon: batteryIndicator.icon
          pointSize: Style.fontSizeM
          color: batteryIndicator.charging ? Theme.cPrimary : Theme.cOnSurfaceVariant
        }

        CrawlText {
          text: Math.round(batteryIndicator.percent) + "%"
          color: Theme.cOnSurfaceVariant
          pointSize: Style.fontSizeM
        }
      }

      // Keyboard layout indicator
      RowLayout {
        spacing: 6
        visible: keyboardLayout.currentLayout !== "Unknown"

        CrawlIcon {
          icon: "keyboard"
          pointSize: Style.fontSizeM
          color: Theme.cOnSurfaceVariant
        }

        CrawlText {
          text: keyboardLayout.currentLayout
          color: Theme.cOnSurfaceVariant
          pointSize: Style.fontSizeM
          elide: Text.ElideRight
        }
      }

      // Caps Lock indicator
      RowLayout {
        spacing: 6
        visible: batteryIndicator.isReady || keyboardLayout.currentLayout !== "Unknown" || LockKeysService.capsLockOn

        CrawlIcon {
          icon: "letter-c"
          pointSize: Style.fontSizeM
          color: LockKeysService.capsLockOn ? Theme.cPrimary : Qt.alpha(Theme.cOnSurfaceVariant, 0.5)
        }

        CrawlText {
          text: "Caps Lock"
          color: LockKeysService.capsLockOn ? Theme.cOnSurfaceVariant : Qt.alpha(Theme.cOnSurfaceVariant, 0.65)
          pointSize: Style.fontSizeM
          elide: Text.ElideRight
        }
      }
    }
  }

  // Bottom container with weather, password input and controls
  Rectangle {
    id: bottomContainer

    // Support for removing the session/power buttons at the bottom.
    readonly property int deltaY: Settings.data.general.showSessionButtonsOnLockScreen ? 0 : (Settings.data.general.compactLockScreen ? 36 : 48) + 14

    height: {
      let calcHeight = Settings.data.general.compactLockScreen ? 120 : 220;
      if (!Settings.data.general.showSessionButtonsOnLockScreen) {
        calcHeight -= bottomContainer.deltaY;
      }
      return calcHeight;
    }
    anchors.horizontalCenter: parent.horizontalCenter
    anchors.bottom: parent.bottom
    anchors.bottomMargin: 100 + bottomContainer.deltaY
    radius: Style.radiusL
    color: Theme.cSurface

    width: Settings.data.general.showHibernateOnLockScreen ? 860 : 810

    ColumnLayout {
      anchors.fill: parent
      anchors.margins: 14
      spacing: Style.marginL

      // Top info row
      RowLayout {
        Layout.fillWidth: true
        Layout.preferredHeight: 65
        spacing: Style.marginXL
        visible: !Settings.data.general.compactLockScreen

        // Media widget with visualizer
        Item {
          Layout.preferredWidth: Style.marginM
          visible: MediaService.currentPlayer && MediaService.canPlay
        }

        Rectangle {
          Layout.preferredWidth: 220
          // Expand to take remaining space when weather is hidden
          Layout.fillWidth: !(Settings.data.location.weatherEnabled && LocationService.data.weather !== null)
          Layout.preferredHeight: 50
          radius: Style.radiusL
          color: "transparent"
          clip: true
          visible: MediaService.currentPlayer && MediaService.canPlay

          Loader {
            anchors.fill: parent
            anchors.margins: 4
            active: Settings.data.audio.visualizerType === "linear"
            z: 0
            sourceComponent: CrawlLinearSpectrum {
              anchors.fill: parent
              values: CavaService.values
              fillColor: Theme.cPrimary
              opacity: 0.4
            }
          }

          Loader {
            anchors.fill: parent
            anchors.margins: 4
            active: Settings.data.audio.visualizerType === "mirrored"
            z: 0
            sourceComponent: CrawlMirroredSpectrum {
              anchors.fill: parent
              values: CavaService.values
              fillColor: Theme.cPrimary
              opacity: 0.4
            }
          }

          Loader {
            anchors.fill: parent
            anchors.margins: 4
            active: Settings.data.audio.visualizerType === "wave"
            z: 0
            sourceComponent: CrawlWaveSpectrum {
              anchors.fill: parent
              values: CavaService.values
              fillColor: Theme.cPrimary
              opacity: 0.4
            }
          }

          RowLayout {
            anchors.fill: parent
            anchors.margins: 8
            spacing: Style.marginM
            z: 1

            Rectangle {
              Layout.preferredWidth: 34
              Layout.preferredHeight: 34
              radius: Math.min(Style.radiusL, width / 2)
              color: "transparent"
              clip: true

              CrawlImageRounded {
                anchors.fill: parent
                anchors.margins: 2
                radius: Math.min(Style.radiusL, width / 2)
                imagePath: MediaService.trackArtUrl
                fallbackIcon: "disc"
                fallbackIconSize: Style.fontSizeM
                borderColor: Theme.cOutline
                borderWidth: Style.borderS
              }
            }

            ColumnLayout {
              Layout.fillWidth: true
              spacing: Style.marginXXS

              CrawlText {
                text: MediaService.trackTitle || "No media"
                pointSize: Style.fontSizeM
                color: Theme.cOnSurface
                Layout.fillWidth: true
                elide: Text.ElideRight
              }

              CrawlText {
                text: MediaService.trackArtist || ""
                pointSize: Style.fontSizeM
                color: Theme.cOnSurfaceVariant
                Layout.fillWidth: true
                elide: Text.ElideRight
              }
            }
          }
        }

        Rectangle {
          Layout.preferredWidth: 1
          Layout.fillHeight: true
          Layout.rightMargin: 4
          color: Qt.alpha(Theme.cOutline, 0.3)
          visible: MediaService.currentPlayer && MediaService.canPlay
        }

        Item {
          Layout.preferredWidth: Style.marginM
          visible: !(MediaService.currentPlayer && MediaService.canPlay)
        }

        // Current weather
        RowLayout {
          visible: Settings.data.location.weatherEnabled && LocationService.data.weather !== null
          Layout.preferredWidth: 180
          spacing: Style.marginM

          CrawlIcon {
            Layout.alignment: Qt.AlignVCenter
            icon: weatherReady ? LocationService.weatherSymbolFromCode(LocationService.data.weather.current_weather.weathercode, LocationService.data.weather.current_weather.is_day) : "weather-cloud-off"
            pointSize: Style.fontSizeXXXL
            color: Theme.cPrimary
          }

          ColumnLayout {
            Layout.fillWidth: true
            spacing: Style.marginXXS

            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginL

              CrawlText {
                text: {
                  var temp = LocationService.data.weather.current_weather.temperature;
                  var suffix = "C";
                  if (Settings.data.location.useFahrenheit) {
                    temp = LocationService.celsiusToFahrenheit(temp);
                    suffix = "F";
                  }
                  temp = Math.round(temp);
                  return temp + "°" + suffix;
                }
                pointSize: Style.fontSizeXL
                font.weight: Style.fontWeightBold
                color: Theme.cOnSurface
              }

              CrawlText {
                text: {
                  var wind = LocationService.data.weather.current_weather.windspeed;
                  var unit = "km/h";
                  if (Settings.data.location.useFahrenheit) {
                    wind = wind * 0.621371; // Convert km/h to mph
                    unit = "mph";
                  }
                  wind = Math.round(wind);
                  return wind + " " + unit;
                }
                pointSize: Style.fontSizeM
                color: Theme.cOnSurfaceVariant
              }
            }

            RowLayout {
              Layout.fillWidth: true
              spacing: Style.marginM

              CrawlText {
                text: Settings.data.location.name.split(",")[0]
                pointSize: Style.fontSizeM
                color: Theme.cOnSurfaceVariant
                visible: !Settings.data.location.hideWeatherCityName
              }

              CrawlText {
                text: (LocationService.data.weather.current && LocationService.data.weather.current.relativehumidity_2m) ? LocationService.data.weather.current.relativehumidity_2m + "% humidity" : ""
                pointSize: Style.fontSizeM
                color: Theme.cOnSurfaceVariant
              }
            }
          }
        }

        // Forecast
        RowLayout {
          visible: Settings.data.location.weatherEnabled && LocationService.data.weather !== null
          Layout.preferredWidth: 260
          Layout.rightMargin: 8
          spacing: Style.marginXS

          Repeater {
            model: MediaService.currentPlayer && MediaService.canPlay ? 2 : 4
            delegate: ColumnLayout {
              Layout.fillWidth: true
              spacing: Style.marginXXS + 1

              CrawlText {
                text: {
                  var weatherDate = new Date(LocationService.data.weather.daily.time[index].replace(/-/g, "/"));
                  return Qt.locale("en").toString(weatherDate, "ddd");
                }
                pointSize: Style.fontSizeM
                color: Theme.cOnSurfaceVariant
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
              }

              CrawlIcon {
                Layout.alignment: Qt.AlignHCenter
                icon: LocationService.weatherSymbolFromCode(LocationService.data.weather.daily.weathercode[index])
                pointSize: Style.fontSizeXL
                color: Theme.cOnSurfaceVariant
              }

              CrawlText {
                text: {
                  var max = LocationService.data.weather.daily.temperature_2m_max[index];
                  var min = LocationService.data.weather.daily.temperature_2m_min[index];
                  if (Settings.data.location.useFahrenheit) {
                    max = LocationService.celsiusToFahrenheit(max);
                    min = LocationService.celsiusToFahrenheit(min);
                  }
                  max = Math.round(max);
                  min = Math.round(min);
                  return max + "°/" + min + "°";
                }
                pointSize: Style.fontSizeM
                font.weight: Style.fontWeightMedium
                color: Theme.cOnSurfaceVariant
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
              }
            }
          }
        }

        Item {
          Layout.fillWidth: batteryIndicator.isReady
        }

        // Battery and Keyboard Layout (full mode only)
        ColumnLayout {
          Layout.alignment: (batteryIndicator.isReady) ? (Qt.AlignRight | Qt.AlignVCenter) : Qt.AlignVCenter
          spacing: Style.marginM
          visible: batteryIndicator.isReady || keyboardLayout.currentLayout !== "Unknown" || LockKeysService.capsLockOn

          // Battery
          RowLayout {
            spacing: Style.marginXS
            visible: batteryIndicator.isReady

            CrawlIcon {
              icon: batteryIndicator.icon
              pointSize: Style.fontSizeM
              color: batteryIndicator.charging ? Theme.cPrimary : Theme.cOnSurfaceVariant
            }

            CrawlText {
              text: Math.round(batteryIndicator.percent) + "%"
              color: Theme.cOnSurfaceVariant
              pointSize: Style.fontSizeM
            }
          }

          // Keyboard Layout
          RowLayout {
            spacing: Style.marginXS
            visible: keyboardLayout.currentLayout !== "Unknown"

            CrawlIcon {
              icon: "keyboard"
              pointSize: Style.fontSizeM
              color: Theme.cOnSurfaceVariant
            }

            CrawlText {
              text: keyboardLayout.currentLayout
              color: Theme.cOnSurfaceVariant
              pointSize: Style.fontSizeM
              elide: Text.ElideRight
            }
          }

          // Caps Lock
          RowLayout {
            spacing: Style.marginXS
            visible: batteryIndicator.isReady || keyboardLayout.currentLayout !== "Unknown" || LockKeysService.capsLockOn

            CrawlIcon {
              icon: "letter-c"
              pointSize: Style.fontSizeM
              color: LockKeysService.capsLockOn ? Theme.cPrimary : Qt.alpha(Theme.cOnSurfaceVariant, 0.5)
            }

            CrawlText {
              text: "Caps Lock"
              color: LockKeysService.capsLockOn ? Theme.cOnSurfaceVariant : Qt.alpha(Theme.cOnSurfaceVariant, 0.65)
              pointSize: Style.fontSizeM
              elide: Text.ElideRight
            }
          }
        }

        Item {
          Layout.preferredWidth: Style.marginM
        }
      }

      // Password input
      RowLayout {
        Layout.fillWidth: true
        spacing: 0

        Item {
          Layout.preferredWidth: Style.marginM
        }

        Rectangle {
          id: passwordInputContainer
          Layout.fillWidth: true
          Layout.preferredHeight: 48
          radius: Style.iRadiusL
          color: Theme.cSurface
          border.color: passwordInput.activeFocus ? Theme.cPrimary : Qt.alpha(Theme.cOutline, 0.3)
          border.width: passwordInput.activeFocus ? 2 : 1

          property bool passwordVisible: false

          Row {
            anchors.left: parent.left
            anchors.leftMargin: 18
            anchors.verticalCenter: parent.verticalCenter
            spacing: Style.marginL

            CrawlIcon {
              icon: "lock"
              pointSize: Style.fontSizeL
              color: passwordInput.activeFocus ? Theme.cPrimary : Theme.cOnSurfaceVariant
              anchors.verticalCenter: parent.verticalCenter
            }

            Row {
              spacing: 0

              Rectangle {
                width: 2
                height: 20
                color: Theme.cPrimary
                visible: passwordInput.activeFocus && passwordInput.text.length === 0
                anchors.verticalCenter: parent.verticalCenter

                // Smooth fade animation (when animations enabled)
                SequentialAnimation on opacity {
                  loops: Animation.Infinite
                  running: root.animationsEnabled && passwordInput.activeFocus && passwordInput.text.length === 0
                  NumberAnimation {
                    to: 0
                    duration: 530
                  }
                  NumberAnimation {
                    to: 1
                    duration: 530
                  }
                }

                // Simple toggle (when animations disabled) — no per-frame repaints
                Timer {
                  interval: 530
                  running: !root.animationsEnabled && passwordInput.activeFocus && passwordInput.text.length === 0
                  repeat: true
                  onTriggered: parent.opacity = parent.opacity > 0.5 ? 0 : 1
                }
              }

              // Password display - show dots or actual text based on passwordVisible
              Item {
                width: Math.min(passwordDisplayContent.width, 550)
                height: 20
                visible: passwordInput.text.length > 0 && !parent.parent.parent.passwordVisible
                anchors.verticalCenter: parent.verticalCenter
                clip: true

                Row {
                  id: passwordDisplayContent
                  spacing: Style.marginXXXS
                  anchors.verticalCenter: parent.verticalCenter

                  Repeater {
                    id: iconRepeater
                    model: ScriptModel {
                      values: Array(passwordInput.text.length)
                    }

                    property list<string> passwordChars: ["circle-filled", "pentagon-filled", "michelin-star-filled", "square-rounded-filled", "guitar-pick-filled", "blob-filled", "triangle-filled"]

                    CrawlIcon {
                      id: icon

                      required property int index
                      // This will be called with index = -1 when the TextInput is deleted
                      // So we make sur index is positive to avoid warning on array accesses
                      property bool drawCustomChar: index >= 0 && Settings.data.general.passwordChars

                      icon: drawCustomChar ? iconRepeater.passwordChars[index % iconRepeater.passwordChars.length] : "circle-filled"
                      pointSize: Style.fontSizeL
                      color: Theme.cPrimary
                      opacity: 1.0
                      scale: animationsEnabled ? 0.5 : 1
                      ParallelAnimation {
                        id: iconAnim
                        NumberAnimation {
                          target: icon
                          properties: "scale"
                          to: 1
                          duration: Style.animationFast
                          easing.type: Easing.BezierSpline
                          easing.bezierCurve: Easing.OutInBounce
                        }
                      }
                      Component.onCompleted: {
                        if (animationsEnabled) {
                          iconAnim.start();
                        }
                      }
                    }
                  }
                }
              }

              CrawlText {
                text: passwordInput.text
                color: Theme.cPrimary
                pointSize: Style.fontSizeM
                visible: passwordInput.text.length > 0 && parent.parent.parent.passwordVisible
                anchors.verticalCenter: parent.verticalCenter
                elide: Text.ElideRight
                width: Math.min(implicitWidth, 550)
              }

              Rectangle {
                width: 2
                height: 20
                color: Theme.cPrimary
                visible: passwordInput.activeFocus && passwordInput.text.length > 0
                anchors.verticalCenter: parent.verticalCenter

                // Smooth fade animation (when animations enabled)
                SequentialAnimation on opacity {
                  loops: Animation.Infinite
                  running: root.animationsEnabled && passwordInput.activeFocus && passwordInput.text.length > 0
                  NumberAnimation {
                    to: 0
                    duration: 530
                  }
                  NumberAnimation {
                    to: 1
                    duration: 530
                  }
                }

                // Simple toggle (when animations disabled) — no per-frame repaints
                Timer {
                  interval: 530
                  running: !root.animationsEnabled && passwordInput.activeFocus && passwordInput.text.length > 0
                  repeat: true
                  onTriggered: parent.opacity = parent.opacity > 0.5 ? 0 : 1
                }
              }
            }
          }

          // Eye button to toggle password visibility
          Rectangle {
            anchors.right: submitButton.left
            anchors.rightMargin: 4
            anchors.verticalCenter: parent.verticalCenter
            width: 36
            height: 36
            radius: Math.min(Style.iRadiusL, width / 2)
            color: eyeButtonArea.containsMouse ? Theme.cPrimary : "transparent"
            visible: passwordInput.text.length > 0
            enabled: !lockContext || !lockContext.unlockInProgress

            CrawlIcon {
              anchors.centerIn: parent
              icon: parent.parent.passwordVisible ? "eye-off" : "eye"
              pointSize: Style.fontSizeM
              color: eyeButtonArea.containsMouse ? Theme.cOnPrimary : Theme.cOnSurfaceVariant

              Behavior on color {
                ColorAnimation {
                  duration: Style.animationFast
                  easing.type: Easing.OutCubic
                }
              }
            }

            MouseArea {
              id: eyeButtonArea
              anchors.fill: parent
              hoverEnabled: true
              cursorShape: Qt.PointingHandCursor
              onClicked: parent.parent.passwordVisible = !parent.parent.passwordVisible
            }

            Behavior on color {
              ColorAnimation {
                duration: Style.animationFast
                easing.type: Easing.OutCubic
              }
            }
          }

          // Submit button
          Rectangle {
            id: submitButton
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            width: 36
            height: 36
            radius: Math.min(Style.iRadiusL, width / 2)
            color: submitButtonArea.containsMouse ? Theme.cPrimary : "transparent"
            border.color: Theme.cPrimary
            border.width: Style.borderS
            enabled: !lockContext || !lockContext.unlockInProgress

            CrawlIcon {
              anchors.centerIn: parent
              icon: "arrow-forward"
              pointSize: Style.fontSizeM
              color: submitButtonArea.containsMouse ? Theme.cOnPrimary : Theme.cPrimary

              Behavior on color {
                ColorAnimation {
                  duration: Style.animationFast
                  easing.type: Easing.OutCubic
                }
              }
            }

            MouseArea {
              id: submitButtonArea
              anchors.fill: parent
              hoverEnabled: true
              cursorShape: Qt.PointingHandCursor
              onClicked: root.doUnlock()
            }

            Behavior on color {
              ColorAnimation {
                duration: Style.animationFast
                easing.type: Easing.OutCubic
              }
            }
          }

          Behavior on border.color {
            ColorAnimation {
              duration: Style.animationFast
              easing.type: Easing.OutCubic
            }
          }
        }

        Item {
          Layout.preferredWidth: Style.marginM
        }
      }

      // Session control buttons
      RowLayout {
        id: sessionButtonRow
        Layout.fillWidth: true
        Layout.preferredHeight: Settings.data.general.compactLockScreen ? 36 : 48
        Layout.alignment: Qt.AlignHCenter
        spacing: Style.marginM
        visible: Settings.data.general.showSessionButtonsOnLockScreen

        readonly property int buttonCount: Settings.data.general.showHibernateOnLockScreen ? 5 : 4
        readonly property real availableWidth: bottomContainer.width - 48
        readonly property real buttonWidth: (availableWidth - (buttonCount - 1) * spacing) / buttonCount
        readonly property real buttonHeight: sessionButtonRow.height

        Item {
          Layout.preferredWidth: sessionButtonRow.buttonWidth
          Layout.preferredHeight: sessionButtonRow.buttonHeight

          CrawlButton {
            anchors.fill: parent
            icon: "logout"
            text: "Logout"
            outlined: true
            backgroundColor: Theme.cOnSurfaceVariant
            textColor: Theme.cOnPrimary
            fontSize: Settings.data.general.compactLockScreen ? Style.fontSizeS : Style.fontSizeM
            iconSize: Settings.data.general.compactLockScreen ? Style.fontSizeM : Style.fontSizeL
            horizontalAlignment: Qt.AlignHCenter
            buttonRadius: Style.radiusL
            onClicked: startTimer("logout")
          }
        }

        Item {
          Layout.preferredWidth: sessionButtonRow.buttonWidth
          Layout.preferredHeight: sessionButtonRow.buttonHeight

          CrawlButton {
            anchors.fill: parent
            icon: "suspend"
            text: "Suspend"
            outlined: true
            backgroundColor: Theme.cOnSurfaceVariant
            textColor: Theme.cOnPrimary
            fontSize: Settings.data.general.compactLockScreen ? Style.fontSizeS : Style.fontSizeM
            iconSize: Settings.data.general.compactLockScreen ? Style.fontSizeM : Style.fontSizeL
            horizontalAlignment: Qt.AlignHCenter
            buttonRadius: Style.radiusL
            onClicked: startTimer("suspend")
          }
        }

        Item {
          Layout.preferredWidth: sessionButtonRow.buttonWidth
          Layout.preferredHeight: sessionButtonRow.buttonHeight
          visible: Settings.data.general.showHibernateOnLockScreen

          CrawlButton {
            anchors.fill: parent
            icon: "hibernate"
            text: "Hibernate"
            outlined: true
            backgroundColor: Theme.cOnSurfaceVariant
            textColor: Theme.cOnPrimary
            fontSize: Settings.data.general.compactLockScreen ? Style.fontSizeS : Style.fontSizeM
            iconSize: Settings.data.general.compactLockScreen ? Style.fontSizeM : Style.fontSizeL
            horizontalAlignment: Qt.AlignHCenter
            buttonRadius: Style.radiusL
            onClicked: startTimer("hibernate")
          }
        }

        Item {
          Layout.preferredWidth: sessionButtonRow.buttonWidth
          Layout.preferredHeight: sessionButtonRow.buttonHeight

          CrawlButton {
            anchors.fill: parent
            icon: "reboot"
            text: "Reboot"
            outlined: true
            backgroundColor: Theme.cOnSurfaceVariant
            textColor: Theme.cOnPrimary
            fontSize: Settings.data.general.compactLockScreen ? Style.fontSizeS : Style.fontSizeM
            iconSize: Settings.data.general.compactLockScreen ? Style.fontSizeM : Style.fontSizeL
            horizontalAlignment: Qt.AlignHCenter
            buttonRadius: Style.radiusL
            onClicked: startTimer("reboot")
          }
        }

        Item {
          Layout.preferredWidth: sessionButtonRow.buttonWidth
          Layout.preferredHeight: sessionButtonRow.buttonHeight

          CrawlButton {
            anchors.fill: parent
            icon: "shutdown"
            text: "Shutdown"
            outlined: true
            backgroundColor: Theme.cError
            textColor: Theme.cOnError
            fontSize: Settings.data.general.compactLockScreen ? Style.fontSizeS : Style.fontSizeM
            iconSize: Settings.data.general.compactLockScreen ? Style.fontSizeM : Style.fontSizeL
            horizontalAlignment: Qt.AlignHCenter
            buttonRadius: Style.radiusL
            onClicked: startTimer("shutdown")
          }
        }
      }
    }
  }
}
