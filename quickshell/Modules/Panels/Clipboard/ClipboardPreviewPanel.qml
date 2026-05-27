import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import qs.Common
import qs.Services
import qs.Widgets

Item {
  id: previewPanel

  property var currentItem: null
  property string fullContent: ""
  property string imageDataUrl: ""
  property bool loadingFullContent: false

  function loadContent() {
    if (!currentItem || !currentItem.id) return;
    var itemId = String(currentItem.id);

    if (currentItem.isImage) {
      imageDataUrl = ClipboardService.getImageData(itemId) || "";
      loadingFullContent = !imageDataUrl;
      if (!imageDataUrl && currentItem.mime) {
        ClipboardService.decodeToDataUrl(itemId, currentItem.mime, null);
      }
    } else {
      var cached = ClipboardService.getContent(itemId);
      if (cached) {
        fullContent = cached;
        loadingFullContent = false;
      } else {
        fullContent = currentItem.preview || "";
        loadingFullContent = true;
        var requestedId = itemId;
        ClipboardService.decode(requestedId, function (content) {
          if (previewPanel.currentItem && String(previewPanel.currentItem.id) === requestedId) {
            var trimmed = content ? content.trim() : "";
            if (trimmed !== "") previewPanel.fullContent = trimmed;
            previewPanel.loadingFullContent = false;
          }
        });
      }
    }
  }

  onCurrentItemChanged: {
    fullContent = "";
    imageDataUrl = "";
    loadingFullContent = false;
    if (currentItem) loadContent();
  }

  readonly property int _rev: ClipboardService.revision
  on_RevChanged: {
    if (currentItem && currentItem.id && !currentItem.isImage && loadingFullContent) {
      var cached = ClipboardService.getContent(String(currentItem.id));
      if (cached) {
        fullContent = cached;
        loadingFullContent = false;
      }
    }
  }

  Timer {
    id: imageUpdateTimer
    interval: 200
    running: currentItem && currentItem.isImage && imageDataUrl === ""
    repeat: currentItem && currentItem.isImage && imageDataUrl === ""

    onTriggered: {
      if (currentItem && currentItem.id) {
        var newData = ClipboardService.getImageData(String(currentItem.id)) || "";
        if (newData !== imageDataUrl) {
          imageDataUrl = newData;
          if (newData) loadingFullContent = false;
        }
      }
    }
  }

  Rectangle {
    anchors.fill: parent
    radius: Style.radiusM
    color: Theme.cSurfaceContainerHigh

    BusyIndicator {
      anchors.centerIn: parent
      running: loadingFullContent
      visible: loadingFullContent
      width: Style.baseWidgetSize
      height: width
    }

    // Image preview
    ColumnLayout {
      anchors.fill: parent
      anchors.margins: Style.marginS
      spacing: Style.marginS
      visible: currentItem && currentItem.isImage && !loadingFullContent && imageDataUrl !== ""

      CrawlImageRounded {
        id: previewImage
        Layout.fillWidth: true
        Layout.fillHeight: true
        radius: Style.marginS
        imagePath: imageDataUrl
        imageFillMode: Image.PreserveAspectFit
      }

      CrawlDivider {
        Layout.fillWidth: true
        Layout.bottomMargin: Style.marginS
      }

      CrawlText {
        Layout.fillWidth: true
        text: {
          var meta = ClipboardService.parseImageMeta(currentItem?.preview);
          if (meta) return meta.fmt + " \u2022 " + meta.w + "\u00d7" + meta.h + " \u2022 " + meta.size;
          var fmt = (currentItem?.mime || "image").split("/")[1]?.toUpperCase() || "Image";
          return fmt + " \u2022 " + previewImage.implicitWidth + "\u00d7" + previewImage.implicitHeight;
        }
        pointSize: Style.fontSizeS
        color: Theme.cOnSurfaceVariant
      }
    }

    // Text preview
    ColumnLayout {
      anchors.fill: parent
      anchors.margins: Style.marginS
      spacing: Style.marginS
      visible: currentItem && !currentItem.isImage && !loadingFullContent

      CrawlScrollView {
        id: clipboardScrollView
        Layout.fillWidth: true
        Layout.fillHeight: true
        horizontalPolicy: ScrollBar.AlwaysOff
        verticalPolicy: ScrollBar.AsNeeded

        CrawlText {
          text: fullContent
          width: clipboardScrollView.availableWidth
          wrapMode: Text.Wrap
          textFormat: Text.PlainText
          font.pointSize: Style.fontSizeS
          font.family: Settings.data.ui.fontFixed
          color: Theme.cOnSurface
        }
      }

      CrawlDivider {
        Layout.fillWidth: true
        Layout.bottomMargin: Style.marginS
      }

      CrawlText {
        Layout.fillWidth: true
        visible: fullContent.length > 0
        text: {
          var c = fullContent.length;
          var w = fullContent.split(/\s+/).filter(function (x) { return x.length > 0; }).length;
          var l = fullContent.split('\n').length;
          return c + " chars, " + w + " words, " + l + " lines";
        }
        pointSize: Style.fontSizeS
        color: Theme.cOnSurfaceVariant
      }
    }

    // No selection state
    CrawlText {
      anchors.centerIn: parent
      visible: !currentItem
      text: "Select an item to preview"
      pointSize: Style.fontSizeS
      color: Theme.cOnSurfaceVariant
    }
  }
}
