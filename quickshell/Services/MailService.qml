pragma Singleton
import QtQuick
import Quickshell
import qs.Common
import qs.Services
Singleton {
  id: root
  // ── State ──────────────────────────────────────────────────────────────────
  property var accounts: ({})
  property var folders: ({})
  property var messages: ({})
  property var currentMessage: null
  property var searchResults: null
  property bool syncing: false
  property string lastError: ""
  property string selectedAccountId: ""
  property string selectedFolder: ""
  property var selectedMessageUids: ([])
  property int totalMessages: 0
  property bool loadingMessages: false
  property bool connected: false
  // ── Signals for MailEvent dispatch ────────────────────────────────────────
  signal accountAdded(string accountId, string displayName, string email)
  signal newMessages(string accountId, string folder, int count)
  signal flagsUpdated(string accountId, int uid, var flags)
  signal syncComplete(string accountId)
  signal syncStatusChanged(string accountId, string status)
  signal attachmentSaved(string accountId, int uid, string attachmentId, string destPath)
  signal mailError(string code, string message)
  // ── Connect via CrawlService (IPC router) ────────────────────────────────
  // CrawlService dispatches domain:"mail" events via mailChanged signal.
  //
  // Future: direct MailService socket (crawl-mail.sock) for standalone use.
  Component.onCompleted: {
    Logger.i("MailService", "Mail service ready via CrawlService")
    _wireEvents()
  }
  function _wireEvents() {
    if (typeof CrawlService !== "undefined" && CrawlService) {
      CrawlService.mailChanged.connect(function(data) {
        root._dispatchMailEvent(data)
      })
    }
  }
  function _dispatchMailEvent(event) {
    switch (event.event) {
      case "account_added":
        root.accountAdded(event.account_id, event.display_name, event.email)
        break
      case "new_messages":
        root.newMessages(event.account_id, event.folder, event.count)
        break
      case "flags_updated":
        root.flagsUpdated(event.account_id, event.uid, event.flags || [])
        break
      case "sync_complete":
        root.syncComplete(event.account_id)
        root.syncing = false
        break
      case "sync_status":
        root.syncStatusChanged(event.account_id, event.status)
        root.syncing = (event.status === "running")
        break
      case "attachment_saved":
        root.attachmentSaved(event.account_id, event.uid, event.attachment_id, event.dest_path)
        break
      default:
        Logger.d("MailService", "Unknown mail event:", event.event)
    }
  }
  // ── IPC: forward request through CrawlService ────────────────────────────
  function _send(method, params, callback) {
    if (typeof CrawlService !== "undefined" && CrawlService && CrawlService.connected) {
      CrawlService.sendRequest(method, params || {}, function(result) {
        if (result && result.error) {
          root.lastError = result.error.message || "Mail error"
          root.mailError(result.error.code || "unknown", root.lastError)
          if (callback) callback(null, result.error)
        } else {
          root.lastError = ""
          var parsed = result || {}
          if (callback) callback(parsed, null)
        }
      })
    } else {
      root.lastError = "CrawlService not connected"
      if (callback) callback(null, { error: { message: root.lastError } })
    }
  }
  function _parseMailResponse(response, callback) {
    if (!response) {
      if (callback) callback(null)
      return
    }
    var mailType = response.type
    var mailData = response.data
    if (callback) callback(mailType, mailData)
  }
  // ── Commands: Accounts ──────────────────────────────────────────────────
  function listAccounts(callback) {
    root._send("ListAccounts", {}, function(result, error) {
      if (error) { if (callback) callback([], error); return }
      root._parseMailResponse(result, function(type, data) {
        if (type === "account_list" && data && data.accounts) {
          root.accounts = data.accounts
          if (callback) callback(data.accounts, null)
        } else {
          if (callback) callback([], { message: "unexpected response" })
        }
      })
    })
  }
  function addAccount(params, callback) {
    root._send("AddAccount", params, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseMailResponse(result, function(type, data) {
        if (type === "ok") {
          if (callback) callback(true, null)
        } else {
          if (callback) callback(null, { message: "add account failed" })
        }
      })
    })
  }
  function removeAccount(accountId, callback) {
    root._send("RemoveAccount", { account_id: accountId }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }
  // ── Commands: Folders ───────────────────────────────────────────────────
  function listFolders(accountId, callback) {
    root._send("ListFolders", { account_id: accountId }, function(result, error) {
      if (error) { if (callback) callback([], error); return }
      root._parseMailResponse(result, function(type, data) {
        if (type === "folder_list" && data && data.folders) {
          var key = accountId
          var fldrs = root.folders
          fldrs[key] = data.folders
          root.folders = fldrs
          if (callback) callback(data.folders, null)
        } else {
          if (callback) callback([], { message: "unexpected response" })
        }
      })
    })
  }
  // ── Commands: Messages ──────────────────────────────────────────────────
  function listMessages(offset, limit, sort, callback) {
    if (typeof sort === "function") { callback = sort; sort = "date_desc" }
    if (typeof limit === "function") { callback = limit; limit = 50 }
    if (typeof offset === "function") { callback = offset; offset = 0 }
    if (!sort) sort = "date_desc"
    if (!root.selectedAccountId || !root.selectedFolder) {
      if (callback) callback([], { message: "no folder selected" })
      return
    }
    root.loadingMessages = true
    root._send("ListMessages", {
      account_id: root.selectedAccountId,
      folder: root.selectedFolder,
      offset: offset,
      limit: limit,
      sort: sort
    }, function(result, error) {
      root.loadingMessages = false
      if (error) { if (callback) callback([], error); return }
      root._parseMailResponse(result, function(type, data) {
        if (type === "message_list" && data) {
          root.messages = data.messages || []
          root.totalMessages = data.total || 0
          if (callback) callback(data.messages, null)
        } else {
          if (callback) callback([], { message: "unexpected response" })
        }
      })
    })
  }
  function getMessage(accountId, uid, fetchRemote, callback) {
    if (typeof fetchRemote === "function") { callback = fetchRemote; fetchRemote = false }
    root._send("GetMessage", { account_id: accountId, uid: uid, fetch_remote: fetchRemote }, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseMailResponse(result, function(type, data) {
        if (type === "message" && data && data.message) {
          root.currentMessage = data.message
          if (callback) callback(data.message, null)
        } else {
          if (callback) callback(null, { message: "unexpected response" })
        }
      })
    })
  }
  function searchMessages(query, folder, limit, callback) {
    if (typeof limit === "function") { callback = limit; limit = 50 }
    if (typeof folder === "function") { callback = folder; folder = null }
    root._send("SearchMessages", {
      account_id: root.selectedAccountId,
      query: query,
      folder: folder,
      limit: limit
    }, function(result, error) {
      if (error) { if (callback) callback([], error); return }
      root._parseMailResponse(result, function(type, data) {
        if (type === "search_results" && data && data.messages) {
          root.searchResults = data.messages
          if (callback) callback(data.messages, null)
        } else {
          if (callback) callback([], { message: "unexpected response" })
        }
      })
    })
  }
  function clearSearch() {
    root.searchResults = null
  }
  // ── Commands: Send ──────────────────────────────────────────────────────
  function sendMessage(params, callback) {
    root._send("SendMessage", params, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseMailResponse(result, function(type, data) {
        if (type === "send_queued" && data) {
          if (callback) callback(data.queue_id, null)
        } else {
          if (callback) callback(null, { message: "send failed" })
        }
      })
    })
  }
  // ── Commands: Move/Copy/Delete ──────────────────────────────────────────
  function moveMessage(accountId, uid, fromFolder, toFolder, callback) {
    root._send("MoveMessage", {
      account_id: accountId,
      uid: uid,
      from_folder: fromFolder,
      to_folder: toFolder
    }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }
  function copyMessage(accountId, uid, toFolder, callback) {
    root._send("CopyMessage", {
      account_id: accountId,
      uid: uid,
      to_folder: toFolder
    }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }
  function deleteMessage(accountId, folder, uid, callback) {
    root._send("DeleteMessage", {
      account_id: accountId,
      folder: folder,
      uid: uid
    }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }
  // ── Commands: Flags ─────────────────────────────────────────────────────
  function setFlags(accountId, folder, uid, add, remove, callback) {
    root._send("SetFlags", {
      account_id: accountId,
      folder: folder,
      uid: uid,
      add: add || [],
      remove: remove || []
    }, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseMailResponse(result, function(type, data) {
        if (callback) callback(data && data.flags ? data.flags : null, null)
      })
    })
  }
  function toggleSeen(accountId, folder, uid, currentlySeen, callback) {
    if (currentlySeen) {
      root.setFlags(accountId, folder, uid, [], ["seen"], callback)
    } else {
      root.setFlags(accountId, folder, uid, ["seen"], [], callback)
    }
  }
  function toggleFlagged(accountId, folder, uid, currentlyFlagged, callback) {
    if (currentlyFlagged) {
      root.setFlags(accountId, folder, uid, [], ["flagged"], callback)
    } else {
      root.setFlags(accountId, folder, uid, ["flagged"], [], callback)
    }
  }
  // ── Commands: Sync ──────────────────────────────────────────────────────
  function syncNow(accountId, callback) {
    root.syncing = true
    root._send("SyncNow", { account_id: accountId }, function(result, error) {
      if (error) { root.syncing = false; if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }
  // ── Commands: Fetch body ────────────────────────────────────────────────
  function fetchBody(accountId, uid, callback) {
    root._send("FetchBody", { account_id: accountId, uid: uid }, function(result, error) {
      if (error) { if (callback) callback(null, error); return }
      root._parseMailResponse(result, function(type, data) {
        if (type === "message" && data && data.message) {
          root.currentMessage = data.message
          if (callback) callback(data.message, null)
        } else {
          if (callback) callback(null, { message: "fetch body failed" })
        }
      })
    })
  }
  // ── Commands: Attachments ───────────────────────────────────────────────
  function saveAttachment(accountId, uid, attachmentId, destPath, callback) {
    root._send("SaveAttachment", {
      account_id: accountId,
      uid: uid,
      attachment_id: attachmentId,
      dest_path: destPath
    }, function(result, error) {
      if (error) { if (callback) callback(false, error); return }
      if (callback) callback(true, null)
    })
  }
  // ── Local state helpers ─────────────────────────────────────────────────
  function selectAccount(accountId) {
    root.selectedAccountId = accountId
    root.selectedFolder = ""
    root.messages = []
    root.currentMessage = null
    root.searchResults = null
    root.listFolders(accountId)
  }
  function selectFolder(accountId, folder) {
    root.selectedFolder = folder
    root.messages = []
    root.currentMessage = null
    root.searchResults = null
    root.listMessages(0, 50)
  }
  function selectMessage(uid) {
    root.selectedMessageUids = [uid]
    root.getMessage(root.selectedAccountId, uid, false)
  }
  function toggleMessageSelection(uid) {
    var sel = root.selectedMessageUids.slice()
    var idx = sel.indexOf(uid)
    if (idx >= 0) sel.splice(idx, 1)
    else sel.push(uid)
    root.selectedMessageUids = sel
  }
  function refresh() {
    if (root.selectedAccountId) {
      root.listFolders(root.selectedAccountId)
      if (root.selectedFolder) {
        root.listMessages(0, 50)
      }
    }
  }

  // ── Window management ──────────────────────────────────────────────────
  property bool isWindowOpen: false
  property var mailWindow: null

  signal windowOpened
  signal windowClosed

  function openWindow() {
    if (root.mailWindow) {
      root.mailWindow.visible = true
      root.isWindowOpen = true
      root.windowOpened()
    }
  }

  function closeWindow() {
    if (root.mailWindow) {
      root.mailWindow.visible = false
      root.isWindowOpen = false
      root.windowClosed()
    }
  }

  function toggleWindow() {
    if (root.isWindowOpen) root.closeWindow()
    else root.openWindow()
  }
}