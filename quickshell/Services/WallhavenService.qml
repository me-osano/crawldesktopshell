pragma Singleton

import QtQuick
import Quickshell
import qs.Common

Singleton {
  id: root

  // State
  property bool fetching: false
  property bool initialSearchScheduled: false
  property var currentResults: []
  property var currentMeta: ({})
  property string lastError: ""
  property string currentQuery: ""
  property int currentPage: 1
  property int lastPage: 1

  // Search parameters
  property string categories: "111" // general,anime,people (all enabled by default)
  property string purity: "100" // sfw
  property string sorting: "relevance" // date_added, relevance, random, views, favorites, toplist
  property string order: "desc" // desc, asc
  property string topRange: "1M" // 1d, 3d, 1w, 1M, 3M, 6M, 1y
  property string seed: "" // For random sorting
  property string minResolution: "" // e.g., "1920x1080"
  property string resolutions: "" // e.g., "1920x1080,1920x1200"
  property string ratios: "" // e.g., "16x9,16x10"
  property string colors: "" // Color hex codes
  // API key status (UI only — daemon manages the key internally)
  readonly property string envApiKey: Quickshell.env("CRAWLDS_WALLHAVEN_API_KEY") || ""
  readonly property string apiKey: envApiKey !== "" ? envApiKey : (Settings.data.wallpaper.wallhavenApiKey || "")
  readonly property bool apiKeyManagedByEnv: envApiKey !== ""
  property bool whEnabled: true

  // Signals
  signal searchCompleted(var results, var meta)
  signal searchFailed(string error)
  signal wallpaperDownloaded(string wallpaperId, string localPath)

  // ── Download callback registry ──────────────────────────────────────────
  property var _downloadCallbacks: ({})

  // ── Connection via CrawlService (IPC router) ────────────────────────────
  Component.onCompleted: {
    CrawlService.wallhavenChanged.connect(function(data) {
      root._handleWallhavenEvent(data)
    })
    Logger.i("Wallhaven", "Wallhaven service ready via CrawlService")
  }

  function _handleWallhavenEvent(data) {
    if (!data) return
    var eventType = data.event || data.type
    if (eventType === "download_complete") {
      Logger.i("Wallhaven", "Download complete:", data.wallpaper_id, data.local_path)
      root.wallpaperDownloaded(data.wallpaper_id || "", data.local_path || "")
      var cb = root._downloadCallbacks[data.wallpaper_id]
      if (cb) {
        delete root._downloadCallbacks[data.wallpaper_id]
        cb(true, data.local_path || "")
      }
    } else if (eventType === "download_failed") {
      Logger.e("Wallhaven", "Download failed:", data.wallpaper_id, data.error)
      var cb = root._downloadCallbacks[data.wallpaper_id]
      if (cb) {
        delete root._downloadCallbacks[data.wallpaper_id]
        cb(false, "")
      }
    } else if (eventType === "download_progress") {
      // Progress events — could expose a signal in the future
    } else if (eventType === "state_changed") {
      root.whEnabled = data.enabled !== false
    }
  }

  // -------------------------------------------------
  function search(query, page) {
    if (fetching) {
      return;
    }

    if (initialSearchScheduled) {
      initialSearchScheduled = false;
    }

    fetching = true;
    lastError = "";
    currentQuery = query || "";
    currentPage = page || 1;

    var safePurity = (purity === "000") ? "100" : purity;

    var payload = {
      query: currentQuery || null,
      categories: categories,
      purity: safePurity,
      sorting: sorting,
      order: order,
      page: currentPage
    };

    if (sorting === "toplist") {
      payload.top_range = topRange;
    }

    if (sorting === "random" && seed) {
      payload.seed = seed;
    }

    if (minResolution) {
      payload.atleast = minResolution;
    }

    if (resolutions) {
      payload.resolutions = resolutions;
    }

    if (ratios) {
      payload.ratios = ratios;
    }

    if (colors) {
      payload.colors = colors;
    }

    CrawlService.sendRequest("WallhavenSearch", payload, function(result) {
      root.fetching = false
      if (result && result.error) {
        var errorMsg = result.error
        root.lastError = errorMsg
        Logger.e("Wallhaven", "Search failed:", errorMsg)
        root.searchFailed(errorMsg)
        return
      }

      try {
        if (result && result.type === "search_results" && result.data) {
          var results = result.data.results || []
          var meta = result.data.meta || {}
          root.currentResults = results
          root.currentMeta = meta
          root.lastPage = meta.last_page || 1

          if (meta.seed) {
            root.seed = meta.seed
          }

          Logger.d("Wallhaven", "Search completed:", results.length, "results, page", root.currentPage, "of", root.lastPage)
          root.searchCompleted(results, meta)
        } else {
          var errorMsg = "Unexpected search response"
          root.lastError = errorMsg
          Logger.e("Wallhaven", errorMsg)
          root.searchFailed(errorMsg)
        }
      } catch (e) {
        var errorMsg = "Failed to process search response: " + e.toString()
        root.lastError = errorMsg
        Logger.e("Wallhaven", errorMsg)
        root.searchFailed(errorMsg)
      }
    })
  }

  // -------------------------------------------------
  function getWallpaperUrl(wallpaper) {
    if (wallpaper.path) {
      return wallpaper.path;
    }
    if (wallpaper.id) {
      var idPrefix = wallpaper.id.substring(0, 2);
      return "https://w.wallhaven.cc/full/" + idPrefix + "/wallhaven-" + wallpaper.id + ".jpg";
    }
    return "";
  }

  // -------------------------------------------------
  function getThumbnailUrl(wallpaper, size) {
    if (wallpaper.thumbs && wallpaper.thumbs[size]) {
      return wallpaper.thumbs[size];
    }
    if (wallpaper.id) {
      var idPrefix = wallpaper.id.substring(0, 2);
      var sizeMap = {
        "small": "small",
        "large": "lg",
        "original": "orig"
      };
      var sizePath = sizeMap[size] || "lg";
      return "https://th.wallhaven.cc/" + sizePath + "/" + idPrefix + "/" + wallpaper.id + ".jpg";
    }
    return "";
  }

  // -------------------------------------------------
  function downloadWallpaper(wallpaper, callback) {
    var url = getWallpaperUrl(wallpaper);
    if (!url) {
      Logger.e("Wallhaven", "No URL available for wallpaper", wallpaper.id);
      if (callback) callback(false, "");
      return;
    }

    var wallpaperId = wallpaper.id;

    var wallpaperDir = Settings.preprocessPath(Settings.data.wallpaper.directory);
    if (!wallpaperDir || wallpaperDir === "") {
      wallpaperDir = Settings.defaultWallpapersDirectory;
    }

    if (!wallpaperDir.endsWith("/")) {
      wallpaperDir += "/";
    }

    Logger.d("Wallhaven", "Downloading wallpaper", wallpaperId, "from", url);

    if (callback) {
      root._downloadCallbacks[wallpaperId] = callback;
    }

    CrawlService.sendRequest("WallhavenDownload", {
      wallpaper_id: wallpaperId,
      url: url,
      dest_dir: wallpaperDir
    }, function(result) {
      if (result && result.error) {
        Logger.e("Wallhaven", "Download request failed:", result.error);
        delete root._downloadCallbacks[wallpaperId];
        if (callback) callback(false, "");
      }
    })
  }

  // -------------------------------------------------
  function setWhEnabled(enabled) {
    CrawlService.sendRequest("WallhavenSetEnabled", { enabled: enabled })
  }

  // -------------------------------------------------
  function reset() {
    currentResults = [];
    currentMeta = {};
    currentQuery = "";
    currentPage = 1;
    lastPage = 1;
    seed = "";
    lastError = "";
  }

  // -------------------------------------------------
  function nextPage() {
    if (currentPage < lastPage && !fetching) {
      search(currentQuery, currentPage + 1);
    }
  }

  // -------------------------------------------------
  function previousPage() {
    if (currentPage > 1 && !fetching) {
      search(currentQuery, currentPage - 1);
    }
  }
}
