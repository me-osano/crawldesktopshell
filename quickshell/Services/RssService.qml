pragma Singleton
import QtQuick
import Quickshell
import qs.Common

Singleton {
  id: root

  // ── State ──────────────────────────────────────────────────────────────────
  property var feeds: []
  property var entries: []
  property var currentEntry: null
  property var categories: []
  property int totalEntries: 0
  property bool loadingFeeds: false
  property bool loadingEntries: false
  property bool refreshing: false
  property string lastError: ""
  property string selectedFeedId: ""
  property string selectedCategory: ""
  property bool onlyUnread: false
  property bool onlyStarred: false
  property bool rssEnabled: true

  // ── Signals for RssEvent dispatch ────────────────────────────────────────
  signal feedAdded(string feedId, string title, string category)
  signal feedRemoved(string feedId)
  signal entryUpdated(string feedId, string entryId)
  signal newEntries(string feedId, int count)
  signal syncStarted(string feedId)
  signal syncComplete(string feedId)

  // ── Connection via CrawlService (IPC router) ────────────────────────────
  Component.onCompleted: {
    CrawlService.rssChanged.connect(function(data) {
      root._handleRssEvent(data)
    })
    Logger.i("RssService", "RSS service ready via CrawlService")
  }

  function _handleRssEvent(data) {
    if (!data) return
    var eventType = data.event || data.type
    switch (eventType) {
      case "feed_added":
        root.feedAdded(data.feed_id || "", data.title || "", data.category || "")
        root.listFeeds()
        break
      case "feed_removed":
        root.feedRemoved(data.feed_id || "")
        root.listFeeds()
        break
      case "entry_updated":
        root.entryUpdated(data.feed_id || "", data.entry_id || "")
        break
      case "new_entries":
        root.newEntries(data.feed_id || "", data.count || 0)
        break
      case "sync_started":
        root.syncStarted(data.feed_id || "")
        root.refreshing = true
        break
      case "sync_complete":
        root.syncComplete(data.feed_id || "")
        root.refreshing = false
        root.listFeeds()
        break
      case "sync_error":
        root.refreshing = false
        root.lastError = data.error || "Sync error"
        break
      case "state_changed":
        root.rssEnabled = data.enabled !== false
        break
    }
  }

  function _send(method, params, callback) {
    CrawlService.sendRequest(method, params || {}, function(result) {
      if (result && result.error) {
        root.lastError = result.error
        if (callback) callback(null, { error: { message: result.error } })
      } else {
        root.lastError = ""
        var parsed = result || {}
        if (callback) callback(parsed, null)
      }
    })
  }

  function _parseResponse(response, callback) {
    if (!response) { if (callback) callback(null, null); return }
    if (callback) callback(response.type, response.data)
  }

  // ── Feeds ───────────────────────────────────────────────────────────────
  function listFeeds(callback) {
    root.loadingFeeds = true
    root._send("RssListFeeds", {}, function(result, error) {
      root.loadingFeeds = false
      if (error) { if (callback) callback([], error); return }
      root._parseResponse(result, function(type, data) {
        if (type === "feed_list" && data && data.feeds) {
          root.feeds = data.feeds
          if (callback) callback(data.feeds, null)
        } else {
          if (callback) callback([], { message: "unexpected response" })
        }
      })
    })
  }

  function addFeed(url, category, callback) {
    root._send("RssAddFeed", { url: url, category: category || "" }, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseResponse(result, function(type, data) {
        if (type === "ok") {
          root.listFeeds()
          if (callback) callback(true, null)
        } else {
          if (callback) callback(null, { message: "add feed failed" })
        }
      })
    })
  }

  function removeFeed(feedId, callback) {
    root._send("RssRemoveFeed", { feed_id: feedId }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (root.selectedFeedId === feedId) root.selectedFeedId = ""
      root.listFeeds()
      root.entries = []
      root.totalEntries = 0
      if (callback) callback(true, null)
    })
  }

  function updateFeed(feedId, category, callback) {
    root._send("RssUpdateFeed", { feed_id: feedId, category: category }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      root.listFeeds()
      if (callback) callback(true, null)
    })
  }

  // ── Entries ─────────────────────────────────────────────────────────────
  function listEntries(params, callback) {
    if (!params || typeof params === "function") { if (typeof params === "function") callback = params; params = {} }
    root.loadingEntries = true
    var payload = {
      feed_id: params.feed_id || root.selectedFeedId || null,
      category: params.category || root.selectedCategory || null,
      offset: params.offset || 0,
      limit: params.limit || 50,
      only_unread: params.only_unread !== undefined ? params.only_unread : root.onlyUnread,
      only_starred: params.only_starred !== undefined ? params.only_starred : root.onlyStarred,
      sort: params.sort || "newest_first"
    }
    root._send("RssListEntries", payload, function(result, error) {
      root.loadingEntries = false
      if (error) { if (callback) callback([], error); return }
      root._parseResponse(result, function(type, data) {
        if (type === "entry_list" && data) {
          root.entries = data.entries || []
          root.totalEntries = data.total || 0
          if (callback) callback(data.entries, null)
        } else {
          if (callback) callback([], { message: "unexpected response" })
        }
      })
    })
  }

  function getEntry(entryId, callback) {
    root._send("RssGetEntry", { entry_id: entryId }, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseResponse(result, function(type, data) {
        if (type === "entry" && data && data.entry) {
          root.currentEntry = data.entry
          if (callback) callback(data.entry, null)
        } else {
          if (callback) callback(null, { message: "unexpected response" })
        }
      })
    })
  }

  function setEntryRead(entryId, isRead, callback) {
    root._send("RssSetEntryRead", { entry_id: entryId, is_read: isRead }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }

  function setEntryStarred(entryId, isStarred, callback) {
    root._send("RssSetEntryStarred", { entry_id: entryId, is_starred: isStarred }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }

  function markAllRead(feedId, callback) {
    root._send("RssMarkAllRead", { feed_id: feedId }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }

  // ── Refresh ─────────────────────────────────────────────────────────────
  function refreshFeed(feedId, callback) {
    root.refreshing = true
    root._send("RssRefreshFeed", { feed_id: feedId }, function(result, error) {
      root.refreshing = false
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }

  function refreshAll(callback) {
    root.refreshing = true
    root._send("RssRefreshAll", {}, function(result, error) {
      root.refreshing = false
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }

  // ── Categories ──────────────────────────────────────────────────────────
  function listCategories(callback) {
    root._send("RssListCategories", {}, function(result, error) {
      if (error) { if (callback) callback([], error); return }
      root._parseResponse(result, function(type, data) {
        if (type === "categories" && data && data.categories) {
          root.categories = data.categories
          if (callback) callback(data.categories, null)
        } else {
          if (callback) callback([], { message: "unexpected response" })
        }
      })
    })
  }

  // ── OPML ────────────────────────────────────────────────────────────────
  function importOpml(path, callback) {
    root._send("RssImportOpml", { path: path }, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseResponse(result, function(type, data) {
        if (type === "import_result" && data) {
          root.listFeeds()
          if (callback) callback(data, null)
        } else {
          if (callback) callback(null, { message: "import failed" })
        }
      })
    })
  }

  function exportOpml(callback) {
    root._send("RssExportOpml", {}, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseResponse(result, function(type, data) {
        if (type === "export_data" && data && data.opml) {
          if (callback) callback(data.opml, null)
        } else {
          if (callback) callback(null, { message: "export failed" })
        }
      })
    })
  }

  // ── Local state helpers ─────────────────────────────────────────────────
  function selectFeed(feedId) {
    root.selectedFeedId = feedId
    root.selectedCategory = ""
    root.entries = []
    root.currentEntry = null
    root.listEntries()
  }

  function selectCategory(category) {
    root.selectedCategory = category
    root.selectedFeedId = ""
    root.entries = []
    root.currentEntry = null
    root.listEntries()
  }

  function selectEntry(entryId) {
    root.getEntry(entryId, function(entry, error) {
      if (entry && !entry.is_read) {
        root.setEntryRead(entryId, true)
      }
    })
  }

  function toggleStarred(entryId, currentlyStarred) {
    root.setEntryStarred(entryId, !currentlyStarred)
  }

  function openInBrowser(url) {
    if (url && url.length > 0) {
      Quickshell.execDetached(["xdg-open", url])
    }
  }

  function setRssEnabled(enabled) {
    CrawlService.sendRequest("RssSetEnabled", { enabled: enabled })
  }

  function refresh() {
    root.listFeeds()
    if (root.selectedFeedId) {
      root.listEntries({ feed_id: root.selectedFeedId })
    } else if (root.selectedCategory) {
      root.listEntries({ category: root.selectedCategory })
    }
  }

  function getFeedById(feedId) {
    var list = root.feeds || []
    for (var i = 0; i < list.length; i++) {
      if (list[i].id === feedId) return list[i]
    }
    return null
  }

  function totalUnreadCount() {
    var list = root.feeds || []
    var sum = 0
    for (var i = 0; i < list.length; i++) sum += (list[i].unread_count || 0)
    return sum
  }

  // ── Window management ──────────────────────────────────────────────────
  property bool isWindowOpen: false
  property var rssWindow: null

  signal windowOpened
  signal windowClosed

  function openWindow() {
    if (root.rssWindow) {
      root.rssWindow.visible = true
      root.isWindowOpen = true
      root.windowOpened()
    }
  }

  function closeWindow() {
    if (root.rssWindow) {
      root.rssWindow.visible = false
      root.isWindowOpen = false
      root.windowClosed()
    }
  }

  function toggleWindow() {
    if (root.isWindowOpen) root.closeWindow()
    else root.openWindow()
  }
}
