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

  // Language
    CrawlComboBox {
      Layout.fillWidth: true
      label: "Application language"
      description: "Select the language used in the application's interface."
      defaultValue: Settings.getDefaultValue("general.language")
      model: [
        { "key": "", "name": "Automatic (en)" },
        { "key": "en", "name": "English" }
      ]
      currentKey: Settings.data.general.language
      settingsPath: "general.language"
      onSelected: key => {
        // Switch to English-only behavior. If user chooses Automatic (empty key), default to English.
        Qt.callLater(() => {
          Settings.data.general.language = key || "en";
        });
      }
    }

  CrawlDivider {
    Layout.fillWidth: true
  }

  // Location
  ColumnLayout {
    Layout.fillWidth: true
    spacing: Style.marginS

    CrawlTextInput {
      Layout.maximumWidth: root.width / 2

      label: "Search for a location"
      description: "e.g. Toronto, ON"
      text: Settings.data.location.name || Settings.defaultLocation
      placeholderText: "Enter the location name"
      onEditingFinished: {
        // Verify the location has really changed to avoid extra resets
        var newLocation = text.trim();
        // If empty, set to default location
        if (newLocation === "") {
          newLocation = Settings.defaultLocation;
          text = Settings.defaultLocation; // Update the input field to show the default
        }
        if (newLocation != Settings.data.location.name) {
          Settings.data.location.name = newLocation;
          LocationService.resetWeather();
        }
      }
    }

    CrawlText {
      visible: LocationService.coordinatesReady
      text: "{name} ({coordinates})"
      pointSize: Style.fontSizeS
      color: Theme.cOnSurfaceVariant
    }
  }

  ColumnLayout {
    spacing: Style.marginL
    Layout.fillWidth: true

    CrawlToggle {
      label: "Enable weather"
      description: "Show weather information throughout the interface and fetch weather data. When disabled, all weather elements will be hidden and no network requests will be made."
      checked: Settings.data.location.weatherEnabled
      onToggled: checked => Settings.data.location.weatherEnabled = checked
      defaultValue: Settings.getDefaultValue("location.weatherEnabled")
    }

    CrawlToggle {
      label: "Display temperature in Fahrenheit (°F)"
      description: "Display temperature in Fahrenheit instead of Celsius."
      checked: Settings.data.location.useFahrenheit
      onToggled: checked => Settings.data.location.useFahrenheit = checked
      enabled: Settings.data.location.weatherEnabled
    }

    CrawlToggle {
      label: "Display weather effects"
      description: "Show additional visual effects (like rain, snow, or lightning) on the weather card."
      checked: Settings.data.location.weatherShowEffects
      onToggled: checked => Settings.data.location.weatherShowEffects = checked
      enabled: Settings.data.location.weatherEnabled
    }

    CrawlToggle {
      label: "Hide city name"
      description: "Hide the city name from weather displays throughout the interface."
      checked: Settings.data.location.hideWeatherCityName
      onToggled: checked => Settings.data.location.hideWeatherCityName = checked
      enabled: Settings.data.location.weatherEnabled
    }

    CrawlToggle {
      label: "Hide timezone"
      description: "Hide the timezone abbreviation from weather displays throughout the interface."
      checked: Settings.data.location.hideWeatherTimezone
      onToggled: checked => Settings.data.location.hideWeatherTimezone = checked
      enabled: Settings.data.location.weatherEnabled
    }
  }
}
