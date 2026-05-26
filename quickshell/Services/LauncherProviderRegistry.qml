pragma Singleton

import QtQuick
import Quickshell
import qs.Common

Singleton {
  id: root

  function init() {
    Logger.i("LauncherProviderRegistry", "Service started");
  }
}
