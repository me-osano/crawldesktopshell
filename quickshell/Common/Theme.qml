pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.Common

/*
NOTE: All color names are prefixed with 'c' (e.g., cPrimary) to prevent QML from
misinterpreting them as signals (e.g., the 'onPrimary' property name).
*/
Singleton {
  id: root

  property bool reloadColors: false

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

  // --- Key Colors: These are the main accent colors that define your app's style
  property color cPrimary: defaultColors.cPrimary
  property color cOnPrimary: defaultColors.cOnPrimary
  property color cSecondary: defaultColors.cSecondary
  property color cOnSecondary: defaultColors.cOnSecondary
  property color cTertiary: defaultColors.cTertiary
  property color cOnTertiary: defaultColors.cOnTertiary

  // --- Utility Colors: These colors serve specific, universal purposes like indicating errors
  property color cError: defaultColors.cError
  property color cOnError: defaultColors.cOnError

  // --- Surface and Variant Colors: These provide additional options for surfaces and their contents, creating visual hierarchy
  property color cSurface: defaultColors.cSurface
  property color cOnSurface: defaultColors.cOnSurface

  property color cSurfaceVariant: defaultColors.cSurfaceVariant
  property color cOnSurfaceVariant: defaultColors.cOnSurfaceVariant

  property color cOutline: defaultColors.cOutline
  property color cShadow: defaultColors.cShadow

  property color cHover: defaultColors.cHover
  property color cOnHover: defaultColors.cOnHover

  // --- Color transition animations ---
  Behavior on cPrimary {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cOnPrimary {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cSecondary {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cOnSecondary {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cTertiary {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cOnTertiary {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cError {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cOnError {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cSurface {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cOnSurface {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cSurfaceVariant {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cOnSurfaceVariant {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cOutline {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cShadow {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cHover {
    enabled: !root.skipTransition
    ColorAnimation {
      duration: Style.animationSlowest
      easing.type: Easing.OutCubic
    }
  }
  Behavior on cOnHover {
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

  // Update colors when customColorsData changes (imperative assignment enables Behavior animations)
  Connections {
    target: customColorsData
    function onCPrimaryChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cPrimary = customColorsData.cPrimary;
    }
    function onCOnPrimaryChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cOnPrimary = customColorsData.cOnPrimary;
    }
    function onCSecondaryChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cSecondary = customColorsData.cSecondary;
    }
    function onCOnSecondaryChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cOnSecondary = customColorsData.cOnSecondary;
    }
    function onCTertiaryChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cTertiary = customColorsData.cTertiary;
    }
    function onCOnTertiaryChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cOnTertiary = customColorsData.cOnTertiary;
    }
    function onCErrorChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cError = customColorsData.cError;
    }
    function onCOnErrorChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cOnError = customColorsData.cOnError;
    }
    function onCSurfaceChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cSurface = customColorsData.cSurface;
    }
    function onCOnSurfaceChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cOnSurface = customColorsData.cOnSurface;
    }
    function onCSurfaceVariantChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cSurfaceVariant = customColorsData.cSurfaceVariant;
    }
    function onCOnSurfaceVariantChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cOnSurfaceVariant = customColorsData.cOnSurfaceVariant;
    }
    function onCOutlineChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cOutline = customColorsData.cOutline;
    }
    function onCShadowChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cShadow = customColorsData.cShadow;
    }
    function onCHoverChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cHover = customColorsData.cHover;
    }
    function onCOnHoverChanged() {
      if (!root.skipTransition) {
        startTransition();
      }
      root.cOnHover = customColorsData.cOnHover;
    }
  }

  function resolveColorKey(key) {
    switch (key) {
    case "primary":
      return root.cPrimary;
    case "secondary":
      return root.cSecondary;
    case "tertiary":
      return root.cTertiary;
    case "error":
      return root.cError;
    default:
      return root.cOnSurface;
    }
  }

  function resolveOnColorKey(key) {
    switch (key) {
    case "primary":
      return root.cOnPrimary;
    case "secondary":
      return root.cOnSecondary;
    case "tertiary":
      return root.cOnTertiary;
    case "error":
      return root.cOnError;
    default:
      return root.cSurface;
    }
  }

  function resolveColorKeyOptional(key) {
    switch (key) {
    case "primary":
      return root.cPrimary;
    case "secondary":
      return root.cSecondary;
    case "tertiary":
      return root.cTertiary;
    case "error":
      return root.cError;
    default:
      return "transparent";
    }
  }
  
  function smartAlpha(baseColor, minAlpha = 0.4) {
    if (!Settings.data.general.translucentWidgets)
      return baseColor;

    let alpha = Math.max(adaptiveOpacity(Settings.data.general.panelBackgroundOpacity), minAlpha);

    // Combine with the base color's existing alpha
    let resultAlpha = Math.max(0, baseColor.a - (1.0 - alpha));
    return Qt.alpha(baseColor, resultAlpha);
  }

  readonly property var colorKeyModel: [
    {
      "key": "none",
      "name": "None"
    },
    {
      "key": "primary",
      "name": "Primary"
    },
    {
      "key": "secondary",
      "name": "Secondary"
    },
    {
      "key": "tertiary",
      "name": "Tertiary"
    },
    {
      "key": "error",
      "name": "Error"
    }
  ]

  // --------------------------------
  // Default colors: CrawlDS (default) dark — must match Assets/ColorScheme/Crawlds
  QtObject {
    id: defaultColors

    readonly property color cPrimary: "#fff59b"
    readonly property color cOnPrimary: "#0e0e43"

    readonly property color cSecondary: "#a9aefe"
    readonly property color cOnSecondary: "#0e0e43"

    readonly property color cTertiary: "#9BFECE"
    readonly property color cOnTertiary: "#0e0e43"

    readonly property color cError: "#FD4663"
    readonly property color cOnError: "#0e0e43"

    readonly property color cSurface: "#070722"
    readonly property color cOnSurface: "#f3edf7"

    readonly property color cSurfaceVariant: "#11112d"
    readonly property color cOnSurfaceVariant: "#7c80b4"

    readonly property color cOutline: "#21215F"
    readonly property color cShadow: "#070722"

    readonly property color cHover: "#9BFECE"
    readonly property color cOnHover: "#0e0e43"
  }

  // ----------------------------------------------------------------
  // FileView to load custom colors data from colors.json
  FileView {
    id: customColorsFile
    path: Settings.directoriesCreated ? (Settings.configDir + "colors.json") : undefined
    printErrors: false
    watchChanges: true
    onFileChanged: {
      Logger.d("Color", "Reloading colors from disk");
      reloadColors = true;
      reload();
    }
    onAdapterUpdated: {
      Logger.d("Color", "Writing colors to disk");
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

      property color cPrimary: defaultColors.cPrimary
      property color cOnPrimary: defaultColors.cOnPrimary

      property color cSecondary: defaultColors.cSecondary
      property color cOnSecondary: defaultColors.cOnSecondary

      property color cTertiary: defaultColors.cTertiary
      property color cOnTertiary: defaultColors.cOnTertiary

      property color cError: defaultColors.cError
      property color cOnError: defaultColors.cOnError

      property color cSurface: defaultColors.cSurface
      property color cOnSurface: defaultColors.cOnSurface

      property color cSurfaceVariant: defaultColors.cSurfaceVariant
      property color cOnSurfaceVariant: defaultColors.cOnSurfaceVariant

      property color cOutline: defaultColors.cOutline
      property color cShadow: defaultColors.cShadow

      property color cHover: defaultColors.cHover
      property color cOnHover: defaultColors.cOnHover
    }
  }
}
