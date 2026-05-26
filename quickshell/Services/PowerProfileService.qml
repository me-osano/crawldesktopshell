pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Services.UPower
import qs.Common
import qs.Services

Singleton {
  id: root

  readonly property var powerProfiles: PowerProfiles
  // Available if we have any profile support (not just performance)
  readonly property bool available: powerProfiles && (powerProfiles.hasPerformanceProfile || powerProfiles.profile !== undefined)
  property int profile: powerProfiles ? powerProfiles.profile : PowerProfile.Balanced


  // Track if service has been initialized (to suppress startup notification)
  property bool initialized: false

  function getName(p) {
    if (!available)
      return "Unknown";

    const prof = (p !== undefined) ? p : profile;

    switch (prof) {
    case PowerProfile.Performance:
      return "Performance";
    case PowerProfile.Balanced:
      return "Balanced";
    case PowerProfile.PowerSaver:
      return "Power saver";
    default:
      return "Unknown";
    }
  }

  function getIcon(p) {
    if (!available)
      return "balanced";

    const prof = (p !== undefined) ? p : profile;

    switch (prof) {
    case PowerProfile.Performance:
      return "performance";
    case PowerProfile.Balanced:
      return "balanced";
    case PowerProfile.PowerSaver:
      return "powersaver";
    default:
      return "balanced";
    }
  }

  function init() {
    Logger.d("PowerProfileService", "Service started");
    // Mark as initialized after a short delay to skip initial profile read
    Qt.callLater(() => { root.initialized = true; });
  }

  function setProfile(p) {
    if (!available)
      return;
    try {
      powerProfiles.profile = p;
    } catch (e) {
      Logger.e("PowerProfileService", "Failed to set profile:", e);
    }
  }

  function cycleProfile() {
    if (!available)
      return;
    const current = powerProfiles.profile;
    const hasPerf = powerProfiles.hasPerformanceProfile;
    
    if (hasPerf) {
      // Full cycle: PowerSaver -> Balanced -> Performance -> PowerSaver
      if (current === PowerProfile.Performance)
        setProfile(PowerProfile.PowerSaver);
      else if (current === PowerProfile.Balanced)
        setProfile(PowerProfile.Performance);
      else if (current === PowerProfile.PowerSaver)
        setProfile(PowerProfile.Balanced);
    } else {
      // No performance profile: toggle between Balanced and PowerSaver
      if (current === PowerProfile.Balanced)
        setProfile(PowerProfile.PowerSaver);
      else
        setProfile(PowerProfile.Balanced);
    }
  }

  function cycleProfileReverse() {
    if (!available)
      return;
    const current = powerProfiles.profile;
    const hasPerf = powerProfiles.hasPerformanceProfile;
    
    if (hasPerf) {
      // Full reverse cycle: Performance -> Balanced -> PowerSaver -> Performance
      if (current === PowerProfile.Performance)
        setProfile(PowerProfile.Balanced);
      else if (current === PowerProfile.Balanced)
        setProfile(PowerProfile.PowerSaver);
      else if (current === PowerProfile.PowerSaver)
        setProfile(PowerProfile.Performance);
    } else {
      // No performance profile: toggle between PowerSaver and Balanced
      if (current === PowerProfile.PowerSaver)
        setProfile(PowerProfile.Balanced);
      else
        setProfile(PowerProfile.PowerSaver);
    }
  }

  function isDefault() {
    if (!available)
      return true;
    return (profile === PowerProfile.Balanced);
  }

  Connections {
    target: powerProfiles
    function onProfileChanged() {
      root.profile = powerProfiles.profile;
      // Only show toast if initialized (not on startup) and we have a valid profile name
      if (root.initialized) {
        const profileName = root.getName();
        if (profileName !== "Unknown") {
          ToastService.showNotice(profileName, "Power profile changed", profileName.toLowerCase().replace(" ", ""));
        }
      }
    }
  }
}
