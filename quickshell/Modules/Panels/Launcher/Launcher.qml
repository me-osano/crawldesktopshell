import QtQuick
import QtQuick.Controls
import Quickshell

import qs.Common
import qs.Modules.MainScreen
import qs.Services
import qs.Widgets

SmartPanel {
  id: root

  // Disable when overlay mode is enabled (LauncherOverlayWindow handles it)
  enabled: !Settings.data.appLauncher.overviewLayer
  visible: !Settings.data.appLauncher.overviewLayer

  // Reference to core (set after panelContent loads)
  property var launcherCoreRef: null

  // Expose core launcher for external access (e.g., IPC)
  readonly property string searchText: launcherCoreRef ? launcherCoreRef.searchText : ""
  readonly property int selectedIndex: launcherCoreRef ? launcherCoreRef.selectedIndex : 0
  readonly property var results: launcherCoreRef ? launcherCoreRef.results : []
  readonly property var activeProvider: launcherCoreRef ? launcherCoreRef.activeProvider : null
  readonly property var currentProvider: launcherCoreRef ? launcherCoreRef.currentProvider : null
  readonly property bool isGridView: launcherCoreRef ? launcherCoreRef.isGridView : false
  readonly property int gridColumns: launcherCoreRef ? launcherCoreRef.gridColumns : 5

  function setSearchText(text) {
    if (launcherCoreRef)
      launcherCoreRef.setSearchText(text);
  }

  // Panel sizing
  readonly property int listPanelWidth: Math.round(500 * Style.uiScaleRatio)
  readonly property int totalBaseWidth: listPanelWidth + Style.margin2L

  preferredWidth: totalBaseWidth
  preferredHeight: Math.round(600 * Style.uiScaleRatio)
  preferredWidthRatio: 0.25
  preferredHeightRatio: 0.5

  // Positioning
  readonly property string screenBarPosition: Settings.getBarPositionForScreen(screen?.name)
  readonly property string panelPosition: {
    if (Settings.data.appLauncher.position === "follow_bar") {
      if (screenBarPosition === "left" || screenBarPosition === "right") {
        return `center_${screenBarPosition}`;
      } else {
        return `${screenBarPosition}_center`;
      }
    } else {
      return Settings.data.appLauncher.position;
    }
  }
  panelAnchorHorizontalCenter: panelPosition === "center" || panelPosition.endsWith("_center")
  panelAnchorVerticalCenter: panelPosition === "center"
  panelAnchorLeft: panelPosition !== "center" && panelPosition.endsWith("_left")
  panelAnchorRight: panelPosition !== "center" && panelPosition.endsWith("_right")
  panelAnchorBottom: panelPosition.startsWith("bottom_")
  panelAnchorTop: panelPosition.startsWith("top_")

  panelContent: Rectangle {
    id: ui
    color: "transparent"
    opacity: launcherCore.resultsReady ? 1.0 : 0.0

    Component.onCompleted: root.launcherCoreRef = launcherCore

    Behavior on opacity {
      NumberAnimation {
        duration: Style.animationFast
        easing.type: Easing.OutCirc
      }
    }

    // Core launcher (state, providers, UI)
    CrawlBox {
      anchors.fill: parent
      anchors.margins: Style.marginL

      LauncherCore {
        id: launcherCore
        anchors.fill: parent
        screen: root.screen
        isOpen: root.isPanelOpen
        onRequestClose: root.close()
        // Defer so the signal emission completes before SmartPanel
        // sets isPanelOpen=false and the contentLoader destroys us.
        onRequestCloseImmediately: Qt.callLater(root.closeImmediately)
      }
    }

  }
}
