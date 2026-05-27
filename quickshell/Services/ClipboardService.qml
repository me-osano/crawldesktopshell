pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import qs.Common
import qs.Services

// Clipboard history service using Rust backend IPC
Singleton {
  id: root

  // Public API
  property bool active: true
  property bool loading: false
  property var items: [] // [{id, preview, mime, isImage, is_pinned, content?, timestamp_ms?}]

  // Expose decoded thumbnails by id and a revision to notify bindings
  property var imageDataById: ({})
  property var _imageDataInsertOrder: []
  readonly property int _imageDataMaxEntries: 20
  property int revision: 0

  // Local content cache - stores full text content by ID
  property var contentCache: ({})

  // Queue for content fetches
  property var _contentQueue: []
  property var _contentCurrentCb: null
  property var _contentCurrentId: ""

  signal listCompleted

  // ── Bootstrap ────────────────────────────────────────────

  Component.onCompleted: {
    if (root.active) {
      Qt.callLater(root.list);
    }
  }

  onActiveChanged: {
    if (root.active) {
      root.list();
    } else {
      loading = false;
      items = [];
    }
  }

  // Periodic refresh as fallback if events are missed
  Timer {
    interval: 5000
    repeat: true
    running: root.active
    onTriggered: list()
  }

  // Listen for clipboard events from the backend
  Connections {
    target: typeof CrawlService !== "undefined" ? CrawlService : null

    function onClipboardChanged(data) {
      if (!root.active) return;
      // Refresh list on any clipboard event
      Qt.callLater(root.list);
    }
  }

  // ── IPC Calls ────────────────────────────────────────────

  function list() {
    if (!root.active) return;
    if (typeof CrawlService === "undefined" || !CrawlService.connected) {
      return;
    }

    loading = true;
    CrawlService.clipboardList(function(result) {
      if (!result || result.error) {
        loading = false;
        return;
      }

      // Map Rust snake_case to JS camelCase
      const mapped = result.map(function(entry) {
        return {
          "id": entry.id,
          "preview": entry.preview,
          "isImage": entry.is_image,
          "mime": entry.mime,
          "is_pinned": entry.is_pinned,
          "content": entry.content || "",
          "timestamp_ms": entry.timestamp_ms,
          "size": entry.size
        };
      });

      // Cache content for text entries
      for (var i = 0; i < mapped.length; i++) {
        var item = mapped[i];
        if (!item.isImage && item.content && !root.contentCache[item.id]) {
          root.contentCache[item.id] = item.content;
        }
      }

      items = mapped;
      loading = false;
      root.listCompleted();
    });
  }

  // Get cached content for an ID
  function getContent(id) {
    if (root.contentCache[id]) {
      return root.contentCache[id];
    }
    return null;
  }

  // Async decode - checks cache first, then fetches via IPC
  function decode(id, cb) {
    var cached = root.contentCache[id];
    if (cached) {
      if (cb) cb(cached);
      return;
    }

    // Queue the request
    root._contentQueue.push({ "id": id, "cb": cb });
    if (root._contentCurrentCb === null) {
      root._startNextContentFetch();
    }
  }

  function _startNextContentFetch() {
    if (root._contentQueue.length === 0) return;

    var job = root._contentQueue.shift();
    root._contentCurrentCb = job.cb;
    root._contentCurrentId = job.id;

    CrawlService.clipboardGetContent(job.id, function(result) {
      if (result && !result.error) {
        var content = result.content || "";
        root.contentCache[job.id] = content;
        if (root._contentCurrentCb) {
          try { root._contentCurrentCb(content); } catch (e) {}
        }
      } else {
        if (root._contentCurrentCb) {
          try { root._contentCurrentCb(""); } catch (e) {}
        }
      }
      root._contentCurrentCb = null;
      root._contentCurrentId = "";
      Qt.callLater(root._startNextContentFetch);
    });
  }

  // Decode image to data URL
  function decodeToDataUrl(id, mime, cb) {
    // If cached, return immediately
    if (root.imageDataById[id]) {
      if (cb) cb(root.imageDataById[id]);
      return;
    }

    CrawlService.clipboardGetContent(id, function(result) {
      if (result && !result.error && result.data_base64) {
        var mimeType = mime || result.mime || "image/*";
        var url = "data:" + mimeType + ";base64," + result.data_base64;

        // Cache the data URL
        root.imageDataById[id] = url;
        root._imageDataInsertOrder.push(String(id));
        while (root._imageDataInsertOrder.length > root._imageDataMaxEntries) {
          var evicted = root._imageDataInsertOrder.shift();
          delete root.imageDataById[evicted];
        }
        root.revision += 1;

        if (cb) cb(url);
      } else {
        if (cb) cb("");
      }
    });
  }

  function getImageData(id) {
    if (id === undefined) return null;
    return root.imageDataById[id];
  }

  function copyToClipboard(id) {
    if (!root.active) return;
    CrawlService.clipboardCopy(id);
  }

  function pasteFromClipboard(id, mime) {
    if (!root.active) return;
    // Copy entry to clipboard first
    CrawlService.clipboardCopy(id, function(result) {
      if (result && !result.error) {
        // Send paste key combo
        var isImage = mime && mime.indexOf("image/") === 0;
        var keys = isImage
          ? ["wtype", "-M", "ctrl", "-k", "v"]
          : ["wtype", "-M", "ctrl", "-M", "shift", "v"];
        Quickshell.execDetached(keys);
      }
    });
  }

  function pasteText(text) {
    if (!text) return;
    CrawlService.clipboardPasteText(text, function(result) {
      if (result && !result.error) {
        Quickshell.execDetached(["wtype", "-M", "ctrl", "-M", "shift", "v"]);
      }
    });
  }

  function deleteById(id) {
    if (!root.active) return;
    // Remove from caches
    delete root.contentCache[id];
    delete root.imageDataById[id];
    var orderIdx = root._imageDataInsertOrder.indexOf(String(id));
    if (orderIdx !== -1)
      root._imageDataInsertOrder.splice(orderIdx, 1);

    CrawlService.clipboardDelete(id, function(result) {
      if (result && !result.error) {
        root.revision++;
        Qt.callLater(root.list);
      }
    });
  }

  function wipeAll() {
    if (!root.active) return;
    // Clear caches
    root.contentCache = {};
    root.imageDataById = {};
    root._imageDataInsertOrder = [];
    root._contentQueue = [];
    root._contentCurrentCb = null;
    root._contentCurrentId = "";

    CrawlService.clipboardWipe(function(result) {
      if (result && !result.error) {
        root.revision++;
        Qt.callLater(root.list);
      }
    });
  }

  function togglePin(id) {
    if (!root.active) return;
    CrawlService.clipboardPin(id, function(result) {
      if (result && !result.error) {
        root.revision++;
        Qt.callLater(root.list);
      }
    });
  }

  // Parse image metadata from preview string
  function parseImageMeta(preview) {
    var re = /\[\[\s*image\s+([\d\.]+\s*(?:KiB|MiB))\s*\]\]/i;
    var match = (preview || "").match(re);
    if (!match) return null;
    return { "size": match[1] };
  }
}
