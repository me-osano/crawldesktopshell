import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Common
import qs.Modules.MainScreen
import qs.Services
import qs.Widgets

SmartPanel {
  id: root

  preferredWidth: Math.round(480 * Style.uiScaleRatio)
  preferredHeight: Math.round(560 * Style.uiScaleRatio)

  property string searchText: ""
  property var selectedItem: null
  property int currentTab: 0
  property bool pinnedOnly: false
  property bool confirmingWipe: false

  function pinnedItems() {
    var items = ClipboardService.items || [];
    var result = [];
    for (var i = 0; i < items.length; i++) {
      if (items[i].is_pinned) result.push(items[i]);
    }
    return result;
  }

  function refresh() {
    if (ClipboardService.active && !ClipboardService.loading) {
      ClipboardService.list(160);
    }
  }

  function filteredItems() {
    var items = ClipboardService.items || [];
    if (root.pinnedOnly) {
      items = items.filter(function (item) { return item.is_pinned; });
    }
    if (!searchText) return items;
    var q = searchText.toLowerCase();
    return items.filter(function (item) {
      return !item.isImage && (item.preview || "").toLowerCase().indexOf(q) !== -1;
    });
  }

  onOpened: {
    searchText = "";
    selectedItem = null;
    refresh();
  }

  panelContent: Rectangle {
    id: panelContent
    color: "transparent"

    ColumnLayout {
      anchors.fill: parent
      anchors.margins: Style.marginL
      spacing: Style.marginM

      // ── Header ──────────────────────────────────────────────
      CrawlBox {
        Layout.fillWidth: true
        Layout.preferredHeight: headerRow.implicitHeight + Style.margin2M

        RowLayout {
          id: headerRow
          anchors.fill: parent
          anchors.margins: Style.marginM

          CrawlIcon {
            icon: "clipboard"
            pointSize: Style.fontSizeXXL
            color: Theme.cPrimary
          }

          CrawlLabel {
            label: "Clipboard"
            Layout.fillWidth: true
          }

          CrawlIconButton {
            icon: "trash"
            tooltipText: "Clear all"
            baseSize: Style.baseWidgetSize * 0.8
            enabled: ClipboardService.active && ClipboardService.items.length > 0
            onClicked: ClipboardService.wipeAll()
          }

          CrawlIconButton {
            icon: "close"
            tooltipText: "Close"
            baseSize: Style.baseWidgetSize * 0.8
            onClicked: root.close()
          }
        }
      }

      // ── Tab bar ──────────────────────────────────────────────
      CrawlTabBar {
        Layout.fillWidth: true
        tabHeight: Style.baseWidgetSize
        currentIndex: root.currentTab
        distributeEvenly: true

        CrawlTabButton {
          text: "History"
          tabIndex: 0
          checked: root.currentTab === 0
          onClicked: root.currentTab = 0
        }
        CrawlTabButton {
          text: "Settings"
          tabIndex: 1
          checked: root.currentTab === 1
          onClicked: root.currentTab = 1
        }
      }

      // ── History tab ─────────────────────────────────────────
      Item {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: root.currentTab === 0

        ColumnLayout {
          anchors.fill: parent
          spacing: Style.marginM

          // Search bar
          CrawlTextInput {
            Layout.fillWidth: true
            visible: ClipboardService.active
            placeholderText: "Search clipboard history..."
            inputIconName: "search"
            onTextChanged: root.searchText = text
          }

          // Disabled state
          CrawlBox {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !ClipboardService.active
            anchors.margins: 0

            ColumnLayout {
              anchors.centerIn: parent
              spacing: Style.marginL

              CrawlIcon {
                icon: "clipboard-off"
                pointSize: 48
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlText {
                text: "Clipboard unavailable"
                pointSize: Style.fontSizeL
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlText {
                text: "The clipboard backend is not running."
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
              }
            }
          }

          // Loading state
          CrawlBox {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: ClipboardService.loading
            anchors.margins: 0

            ColumnLayout {
              anchors.centerIn: parent
              spacing: Style.marginM

              BusyIndicator {
                running: true
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlText {
                text: "Loading clipboard history..."
                pointSize: Style.fontSizeM
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }
            }
          }

          // Empty state
          CrawlBox {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: ClipboardService.active && !ClipboardService.loading && ClipboardService.items.length === 0
            anchors.margins: 0

            ColumnLayout {
              anchors.centerIn: parent
              spacing: Style.marginL

              CrawlIcon {
                icon: "clipboard"
                pointSize: 48
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlText {
                text: "Clipboard is empty"
                pointSize: Style.fontSizeL
                color: Theme.cOnSurfaceVariant
                Layout.alignment: Qt.AlignHCenter
              }

              CrawlText {
                text: "Copy something to see it here."
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                Layout.alignment: Qt.AlignHCenter
              }
            }
          }

          // Item list + preview
          RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: ClipboardService.active && !ClipboardService.loading && ClipboardService.items.length > 0
            spacing: Style.marginS

            CrawlScrollView {
              Layout.fillWidth: true
              Layout.fillHeight: true
              Layout.minimumWidth: 200
              horizontalPolicy: ScrollBar.AlwaysOff
              verticalPolicy: ScrollBar.AsNeeded
              reserveScrollbarSpace: false
              gradientColor: Theme.cSurface

              ListView {
                id: itemList
                anchors.fill: parent
                clip: true
                spacing: Style.marginXS
                model: root.filteredItems()
                currentIndex: -1

                delegate: ItemDelegate {
                  id: itemDelegate
                  width: ListView.view.width
                  height: delegateRow.implicitHeight + Style.marginM

                  highlighted: ListView.isCurrentItem
                  palette.highlight: Theme.cPrimary
                  palette.highlightedText: Theme.cOnPrimary

                  onClicked: {
                    itemList.currentIndex = index;
                    root.selectedItem = modelData;
                  }

                  RowLayout {
                    id: delegateRow
                    anchors.fill: parent
                    anchors.margins: Style.marginS
                    spacing: Style.marginM

                    CrawlIcon {
                      icon: modelData.isImage ? "photo" : "clipboard"
                      pointSize: Style.fontSizeL
                      color: modelData.isImage ? Theme.cTertiary : Theme.cOnSurfaceVariant
                    }

                    ColumnLayout {
                      Layout.fillWidth: true
                      spacing: 2

                      CrawlText {
                        Layout.fillWidth: true
                        text: {
                          if (modelData.isImage) {
                            var meta = ClipboardService.parseImageMeta(modelData.preview);
                            return meta ? "Image " + meta.w + "\u00d7" + meta.h : "Image";
                          }
                          var preview = (modelData.preview || "").trim();
                          var firstLine = preview.split('\n')[0] || "Empty";
                          return firstLine.length > 60 ? firstLine.substring(0, 57) + "..." : firstLine;
                        }
                        elide: Text.ElideRight
                        pointSize: Style.fontSizeM
                        color: ListView.isCurrentItem ? Theme.cOnPrimary : Theme.cOnSurface
                        maximumLineCount: 1
                      }

                      CrawlText {
                        Layout.fillWidth: true
                        text: {
                          if (modelData.isImage) {
                            var meta = ClipboardService.parseImageMeta(modelData.preview);
                            return meta ? meta.fmt + " \u2022 " + meta.size : modelData.mime || "Image data";
                          }
                          var p = (modelData.preview || "").trim();
                          var lines = p.split('\n');
                          if (lines.length > 1) {
                            var desc = lines[1];
                            return desc.length > 80 ? desc.substring(0, 77) + "..." : desc;
                          }
                          if (p.length >= 100) return "Long text";
                          var c = p.length;
                          var w = p.split(/\s+/).length;
                          return c + " chars, " + w + " word" + (w !== 1 ? "s" : "");
                        }
                        elide: Text.ElideRight
                        pointSize: Style.fontSizeXS
                        color: ListView.isCurrentItem ? Theme.cOnPrimary : Theme.cOnSurfaceVariant
                        maximumLineCount: 1
                      }
                    }
                  }

                  CrawlPopupContextMenu {
                    id: itemContextMenu
                    model: [
                      { "label": "Copy", "action": "copy", "icon": "copy" },
                      { "label": "Paste", "action": "paste", "icon": "clipboard-check" },
                      { "label": "Delete", "action": "delete", "icon": "trash" }
                    ]
                    onTriggered: function (action) {
                      if (action === "copy") {
                        ClipboardService.copyToClipboard(modelData.id);
                      } else if (action === "paste") {
                        ClipboardService.pasteFromClipboard(modelData.id, modelData.mime);
                      } else if (action === "delete") {
                        ClipboardService.deleteById(modelData.id);
                      }
                    }
                  }

                  MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.RightButton
                    onClicked: {
                      itemList.currentIndex = index;
                      root.selectedItem = modelData;
                      PanelService.showContextMenu(itemContextMenu, itemDelegate, screen);
                    }
                  }
                }

                ScrollBar.vertical: ScrollBar {}
              }
            }

            ClipboardPreviewPanel {
              Layout.preferredWidth: 220
              Layout.fillHeight: true
              visible: root.selectedItem !== null
              currentItem: root.selectedItem
            }
          }
        }
      }

      // ── Settings tab ────────────────────────────────────────
      CrawlBox {
        Layout.fillWidth: true
        Layout.fillHeight: true
        visible: root.currentTab === 1

        ColumnLayout {
          anchors.fill: parent
          anchors.margins: Style.marginM
          spacing: Style.marginS

          // ── Header ────────────────────────────────────────────
          RowLayout {
            spacing: Style.marginM

            CrawlIcon {
              icon: "settings"
              pointSize: Style.fontSizeXL
              color: Theme.cPrimary
            }

            ColumnLayout {
              Layout.fillWidth: true
              spacing: Style.marginXXS

              CrawlText {
                text: "Clipboard Settings"
                pointSize: Style.fontSizeL
                color: Theme.cOnSurface
                font.weight: Font.DemiBold
              }

              CrawlText {
                text: "Backend: Rust daemon (poll interval 200ms)"
                pointSize: Style.fontSizeXS
                color: Theme.cOnSurfaceVariant
              }
            }
          }

          CrawlDivider { Layout.fillWidth: true }

          // ── Stats card ────────────────────────────────────────
          RowLayout {
            spacing: Style.marginS

            CrawlBox {
              Layout.fillWidth: true
              Layout.preferredHeight: 72

              ColumnLayout {
                anchors.centerIn: parent
                spacing: Style.marginXXS

                CrawlText {
                  text: String(ClipboardService.items ? ClipboardService.items.length : 0)
                  pointSize: Style.fontSizeXL
                  color: Theme.cPrimary
                  font.weight: Font.DemiBold
                }
                CrawlText {
                  text: "entries"
                  pointSize: Style.fontSizeXS
                  color: Theme.cOnSurfaceVariant
                }
              }
            }

            CrawlBox {
              Layout.fillWidth: true
              Layout.preferredHeight: 72

              ColumnLayout {
                anchors.centerIn: parent
                spacing: Style.marginXXS

                CrawlText {
                  text: {
                    if (!ClipboardService.items) return "0";
                    var c = 0;
                    for (var i = 0; i < ClipboardService.items.length; i++) {
                      if (ClipboardService.items[i].is_pinned) c++;
                    }
                    return String(c);
                  }
                  pointSize: Style.fontSizeXL
                  color: Theme.cSecondary
                  font.weight: Font.DemiBold
                }
                CrawlText {
                  text: "pinned"
                  pointSize: Style.fontSizeXS
                  color: Theme.cOnSurfaceVariant
                }
              }
            }

            CrawlBox {
              Layout.fillWidth: true
              Layout.preferredHeight: 72

              ColumnLayout {
                anchors.centerIn: parent
                spacing: Style.marginXXS

                Rectangle {
                  width: 10
                  height: 10
                  radius: 5
                  color: Theme.cTertiary
                  Layout.alignment: Qt.AlignHCenter
                }
                CrawlText {
                  text: "Active"
                  pointSize: Style.fontSizeXS
                  color: Theme.cOnSurfaceVariant
                  Layout.alignment: Qt.AlignHCenter
                }
              }
            }
          }

          CrawlDivider { Layout.fillWidth: true }

          // ── Filter section ────────────────────────────────────
          CrawlToggle {
            label: "Show pinned only"
            description: "Only show pinned clipboard entries in the list"
            checked: root.pinnedOnly
            onToggled: root.pinnedOnly = checked
          }

          // ── Pinned items list ─────────────────────────────────
          CrawlBox {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.pinnedItems().length > 0

            ColumnLayout {
              anchors.fill: parent
              anchors.margins: Style.marginS
              spacing: Style.marginXXS

              CrawlText {
                text: "Pinned Items (" + root.pinnedItems().length + ")"
                pointSize: Style.fontSizeS
                color: Theme.cOnSurfaceVariant
                font.weight: Font.DemiBold
                leftPadding: Style.marginXS
                bottomPadding: Style.marginXXS
              }

              CrawlScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                horizontalPolicy: ScrollBar.AlwaysOff
                verticalPolicy: ScrollBar.AsNeeded
                gradientColor: Theme.cSurface

                ColumnLayout {
                  width: parent.width
                  spacing: Style.marginXXS

                  Repeater {
                    model: root.pinnedItems()

                    Rectangle {
                      id: pinDelegate
                      Layout.fillWidth: true
                      height: 32
                      radius: Style.iRadiusS
                      color: mouseArea.containsMouse ? Theme.cHover : "transparent"

                      property var itemData: modelData

                      RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Style.marginXS
                        anchors.rightMargin: Style.marginXXS
                        spacing: Style.marginS

                        CrawlIcon {
                          icon: itemData.isImage ? "photo" : "file-text"
                          pointSize: Style.fontSizeS
                          color: itemData.isImage ? Theme.cTertiary : Theme.cOnSurfaceVariant
                        }

                        CrawlText {
                          Layout.fillWidth: true
                          text: {
                            if (itemData.isImage) return itemData.mime || "Image";
                            var p = (itemData.preview || "").trim();
                            return p.length > 40 ? p.substring(0, 37) + "..." : p;
                          }
                          elide: Text.ElideRight
                          pointSize: Style.fontSizeXS
                          color: Theme.cOnSurface
                          maximumLineCount: 1
                        }

                        CrawlIconButton {
                          icon: "pin-off"
                          tooltipText: "Unpin"
                          baseSize: 20
                          opacity: 0.7
                          onClicked: ClipboardService.togglePin(itemData.id)
                        }
                      }

                      MouseArea {
                        id: mouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                      }
                    }
                  }
                }
              }
            }
          }

          // ── Danger zone ───────────────────────────────────────
          Item { Layout.fillHeight: true }

          CrawlDivider { Layout.fillWidth: true }

          Rectangle {
            Layout.fillWidth: true
            radius: Style.iRadiusM
            color: root.confirmingWipe ? Qt.alpha(Theme.cError, 0.12) : "transparent"
            border.color: root.confirmingWipe ? Theme.cError : "transparent"
            border.width: root.confirmingWipe ? Style.borderS : 0
            implicitHeight: dangerRow.implicitHeight + Style.marginM * 2

            RowLayout {
              id: dangerRow
              anchors.left: parent.left
              anchors.right: parent.right
              anchors.verticalCenter: parent.verticalCenter
              anchors.margins: Style.marginM
              spacing: Style.marginM

              ColumnLayout {
                Layout.fillWidth: true
                spacing: Style.marginXXS

                CrawlText {
                  text: "Danger Zone"
                  pointSize: Style.fontSizeS
                  color: root.confirmingWipe ? Theme.cError : Theme.cOnSurfaceVariant
                  font.weight: Font.DemiBold
                }

                CrawlText {
                  text: root.confirmingWipe
                    ? "This action cannot be undone. All clipboard history will be deleted."
                    : "Permanently delete all clipboard history."
                  pointSize: Style.fontSizeXS
                  color: Theme.cOnSurfaceVariant
                  wrapMode: Text.WordWrap
                  Layout.fillWidth: true
                  Layout.maximumWidth: 220
                }
              }

              CrawlButton {
                text: root.confirmingWipe ? "Confirm clear" : "Clear all"
                icon: "trash"
                backgroundColor: root.confirmingWipe ? Theme.cError : Theme.cPrimary
                textColor: root.confirmingWipe ? Theme.cOnError : Theme.cOnPrimary
                enabled: ClipboardService.items && ClipboardService.items.length > 0
                onClicked: {
                  if (root.confirmingWipe) {
                    root.confirmingWipe = false;
                    ClipboardService.wipeAll();
                  } else {
                    root.confirmingWipe = true;
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
