pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

import qs.Common
import qs.Services

Singleton {
    id: root

    // Suppress transition animations until the first colors.json load completes
    property bool skipTransition: true
  
    // Flag indicating theme colors are currently transitioning (for widgets to disable their own animations)
    property bool isTransitioning: false
  
    // Timer to reset isTransitioning after animation completes
    Timer {
      id: transitionTimer
      interval: Style.animationSlowest + 50 // Small buffer after animation
      onTriggered: root.isTransitioning = false
    }
    
    readonly property var defaultColors: getColors(ThemeService.currentTheme, ThemeService.currentVariant)
    readonly property var themeMode: ThemeService.mode
    readonly property bool isDark: ThemeService.isDark
    readonly property string variant: ThemeService.currentVariant

    // ----Key Colors----
    property color primary: defaultColors.primary
    property color onPrimary: defaultColors.on_primary
    property color secondary: defaultColors.secondary
    property color onSecondary: defaultColors.on_secondary
    property color tertiary: defaultColors.tertiary
    property color onTertiary: defaultColors.on_tertiary
    // -----Utility Colors----
    property color error: defaultColors.error
    property color onError: defaultColors.on_error
    // ----Surface and Variant Colors-----
    property color surface: defaultColors.surface
    property color onSurface: defaultColors.on_surface
    property color surfaceVariant: defaultColors.surface_variant
    property color onSurfaceVariant: defaultColors.on_surface_variant
    property color outline: defaultColors.outline
    property color shadow: defaultColors.shadow
    property color hover: defaultColors.hover
    property color onHover: defaultColors.on_hover

    // --- Derived Colors: Variations based on primary color
    property color primaryHover: Qt.rgba(primary.r, primary.g, primary.b, 0.12)
    property color primaryHoverLight: Qt.rgba(primary.r, primary.g, primary.b, 0.08)
    property color primaryPressed: Qt.rgba(primary.r, primary.g, primary.b, 0.16)
    property color primarySelected: Qt.rgba(primary.r, primary.g, primary.b, 0.3)
    property color primaryBackground: Qt.rgba(primary.r, primary.g, primary.b, 0.04)
    // --- Secondary hover variation ----
    property color secondaryHover: Qt.rgba(secondary.r, secondary.g, secondary.b, 0.08)
    // ---- Surface Hover variations ----
    property color surfaceHover: Qt.rgba(surfaceVariant.r, surfaceVariant.g, surfaceVariant.b, 0.08)
    property color surfacePressed: Qt.rgba(surfaceVariant.r, surfaceVariant.g, surfaceVariant.b, 0.12)
    property color surfaceSelected: Qt.rgba(surfaceVariant.r, surfaceVariant.g, surfaceVariant.b, 0.15)
    property color surfaceLight: Qt.rgba(surfaceVariant.r, surfaceVariant.g, surfaceVariant.b, 0.1)
    property color surfaceVariantAlpha: Qt.rgba(surfaceVariant.r, surfaceVariant.g, surfaceVariant.b, 0.2)
    // ----- Outline Hover variations ----
    property color outlineButton: Qt.rgba(outline.r, outline.g, outline.b, 0.5)
    property color outlineLight: Qt.rgba(outline.r, outline.g, outline.b, 0.05)
    property color outlineMedium: Qt.rgba(outline.r, outline.g, outline.b, 0.08)
    property color outlineStrong: Qt.rgba(outline.r, outline.g, outline.b, 0.12)
    // ----- Error variations ----
    property color errorHover: Qt.rgba(error.r, error.g, error.b, 0.12)
    property color errorPressed: Qt.rgba(error.r, error.g, error.b, 0.16)
    // ---- Shadow colors
    property color shadowMedium: Qt.rgba(0, 0, 0, 0.08)
    property color shadowStrong: Qt.rgba(0, 0, 0, 0.3)
    
    readonly property var colorKeyModel: [
      {
        "key": "none",
        "name": "common.none"
      },
      {
        "key": "primary",
        "name": "common.primary"
      },
      {
        "key": "secondary",
        "name": "common.secondary"
      },
      {
        "key": "tertiary",
        "name": "common.tertiary"
      },
      {
        "key": "error",
        "name": "common.error"
      }
    ]
    
    // Utility functions
    // Adaptive opacity calculation: automatically makes light mode more transparent
    function adaptiveOpacity(baseOpacity) {
      return Settings.data.colorSchemes.darkMode ? baseOpacity : Math.pow(baseOpacity, 1.5);
    }
  
    function smartAlpha(baseColor, minAlpha = 0.4) {
      if (!Settings.data.ui.translucentWidgets)
        return baseColor;
  
      let alpha = Math.max(adaptiveOpacity(Settings.data.ui.panelBackgroundOpacity), minAlpha);
  
      // Combine with the base color's existing alpha
      let resultAlpha = Math.max(0, baseColor.a - (1.0 - alpha));
      return Qt.alpha(baseColor, resultAlpha);
    }
    
    // Default colors
    function getColors(theme, variant) {
        if (!theme) {
            return _defaultColors()
        }
        const v = theme[variant] || theme.dark || theme.light
        if (!v) {
            return _defaultColors()
        }
        return v.colors || _defaultColors()
    }

    function _defaultColors() {
        return {
            primary: "#fff59b",
            on_primary: "#0e0e43",
            secondary: "#a9aefe",
            on_secondary: "#0e0e43",
            tertiary: "#9BFECE",
            on_tertiary: "#0e0e43",
            error: "#FD466",
            on_error: "#0e0e43",
            surface: "#070722",
            on_surface: "#f3edf7",
            surface_variant: "#cccccc",
            on_surface_variant: "#11112d",
            outline: "#7c80b4",
            shadow: "#070722",
            hover: "#9BFECE",
            on_hover: "#0e0e43"
        }
    }

    // Color transition animations
    // --- Color transition animations ---
    Behavior on primary {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on onPrimary {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on secondary {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on onSecondary {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on tertiary {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on onTertiary {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on error {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on onError {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on surface {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on onSurface {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on surfaceVariant {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on onSurfaceVariant {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on outline {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on shadow {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on hover {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
    Behavior on onHover {
      enabled: !root.skipTransition
      ColorAnimation {
        duration: Style.animationSlowest
        easing.type: Easing.OutCubic
      }
    }
   
    // Helper to start transition and update a color
    function startTransition() {
      root.isTransitioning = true;
      transitionTimer.restart();
    }

    // --------------------------------
    // Default colors: CrawlDS (default) dark — must match Assets/ColorScheme/Crawlds
    QtObject {
      id: defaultColors
  
      readonly property color primary: "#fff59b"
      readonly property color onPrimary: "#0e0e43"
  
      readonly property color secondary: "#a9aefe"
      readonly property color onSecondary: "#0e0e43"
  
      readonly property color tertiary: "#9BFECE"
      readonly property color onTertiary: "#0e0e43"
  
      readonly property color error: "#FD4663"
      readonly property color onError: "#0e0e43"
  
      readonly property color surface: "#070722"
      readonly property color onSurface: "#f3edf7"
  
      readonly property color surfaceVariant: "#11112d"
      readonly property color onSurfaceVariant: "#7c80b4"
  
      readonly property color outline: "#21215F"
      readonly property color shadow: "#070722"
  
      readonly property color hover: "#9BFECE"
      readonly property color onHover: "#0e0e43"
    }

    // ----------------------------------------------------------------
    // FileView to load custom colors data from colors.json
    FileView {
      id: customColorsFile
      path: Settings.directoriesCreated ? (Settings.configDir + "current-theme.json") : undefined
      printErrors: false
      watchChanges: true
      onFileChanged: scheduleExternalColorReload()
      onAdapterUpdated: {
        Logger.d("Theme", "Writing colors to disk");
        writeAdapter();
      }
  
      onLoaded: {
        if (root.skipTransition) {
          Qt.callLater(function () {
            root.skipTransition = false;
          });
        }
      }
  
      // Trigger initial load when path changes from empty to actual path
      onPathChanged: {
        if (path !== undefined) {
          reload();
        }
      }
      onLoadFailed: function (error) {
        if (reloadColors) {
          reloadColors = false;
          return;
        }
  
        if (root.skipTransition) {
          Qt.callLater(function () {
            root.skipTransition = false;
          });
        }
  
        // Error code 2 = ENOENT (No such file or directory)
        if (error === 2 || error.toString().includes("No such file")) {
          // File doesn't exist, create it with default values
          writeAdapter();
        }
      }
      JsonAdapter {
        id: customColorsData
  
        property color primary: defaultColors.primary
        property color onPrimary: defaultColors.onPrimary
  
        property color secondary: defaultColors.secondary
        property color onSecondary: defaultColors.onSecondary
  
        property color tertiary: defaultColors.tertiary
        property color onTertiary: defaultColors.onTertiary
  
        property color error: defaultColors.error
        property color onError: defaultColors.onError
  
        property color surface: defaultColors.surface
        property color onSurface: defaultColors.onSurface
  
        property color surfaceVariant: defaultColors.surfaceVariant
        property color onSurfaceVariant: defaultColors.onSurfaceVariant
  
        property color outline: defaultColors.outline
        property color shadow: defaultColors.shadow
  
        property color hover: defaultColors.hover
        property color onHover: defaultColors.onHover
      }
    }
}