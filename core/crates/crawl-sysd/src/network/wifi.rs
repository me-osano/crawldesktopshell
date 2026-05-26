//! WiFi scanning, listing, and connection logic.

use std::collections::HashMap;
use tokio::time::Instant;
use zbus::Connection;

use crate::network::NetError;
use crate::network::dbus::{
    NM_DEVICE_TYPE_WIFI, NMAccessPointProxy, NMDeviceProxy, NMDeviceWirelessProxy,
    NMIP4ConfigProxy, NMIP6ConfigProxy, NMSettingsConnectionProxy, NetworkManagerProxy,
};
use crate::network::util;
use crawl_ipc::types::{ActiveWifiDetails, WifiNetwork};

pub struct NetworkCache {
    wifi_list: Option<CachedWifiList>,
    wifi_details: Option<CachedWifiDetails>,
}

impl Default for NetworkCache {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkCache {
    pub fn new() -> Self {
        Self {
            wifi_list: None,
            wifi_details: None,
        }
    }
}

pub struct CachedWifiList {
    data: Vec<WifiNetwork>,
    fetched_at: Instant,
}

pub struct CachedWifiDetails {
    data: ActiveWifiDetails,
    fetched_at: Instant,
}

pub const WIFI_LIST_TTL: tokio::time::Duration = tokio::time::Duration::from_secs(25);
pub const WIFI_DETAILS_TTL: tokio::time::Duration = tokio::time::Duration::from_secs(25);

#[derive(Debug, Clone, Default)]
pub struct WifiConnectConfig {
    pub ssid: String,
    pub password: Option<String>,
    pub security_key: Option<String>,
    pub is_hidden: bool,
    pub identity: Option<String>,
    pub eap: Option<String>,
    pub phase2: Option<String>,
    pub anon_identity: Option<String>,
    pub ca_cert: Option<String>,
}

pub async fn list_wifi_with_cache(
    conn: &Connection,
    nm: &NetworkManagerProxy<'_>,
    cache: &mut NetworkCache,
) -> Result<Vec<WifiNetwork>, NetError> {
    if let Some(entry) = &cache.wifi_list {
        if entry.fetched_at.elapsed() < WIFI_LIST_TTL {
            return Ok(entry.data.clone());
        }
    }
    let list = list_wifi_from_conn(conn, nm).await?;
    cache.wifi_list = Some(CachedWifiList {
        data: list.clone(),
        fetched_at: Instant::now(),
    });
    Ok(list)
}

pub async fn refresh_wifi_details_with_cache(
    conn: &Connection,
    nm: &NetworkManagerProxy<'_>,
    cache: &mut NetworkCache,
) -> Result<ActiveWifiDetails, NetError> {
    if let Some(entry) = &cache.wifi_details {
        if entry.fetched_at.elapsed() < WIFI_DETAILS_TTL {
            return Ok(entry.data.clone());
        }
    }
    let details = refresh_wifi_details(conn, nm).await?;
    cache.wifi_details = Some(CachedWifiDetails {
        data: details.clone(),
        fetched_at: Instant::now(),
    });
    Ok(details)
}

pub async fn list_wifi_from_conn(
    conn: &Connection,
    nm: &NetworkManagerProxy<'_>,
) -> Result<Vec<WifiNetwork>, NetError> {
    let mut seen: HashMap<String, WifiNetwork> = HashMap::new();

    let existing = list_known_wifi_ssids(conn, nm).await.unwrap_or_default();

    for path in nm.get_devices().await? {
        let dev = NMDeviceProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;
        if dev.device_type().await.unwrap_or(0) != NM_DEVICE_TYPE_WIFI {
            continue;
        }

        let wifi = NMDeviceWirelessProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;
        let active_ap = wifi.active_access_point().await.ok();
        let aps = wifi.access_points().await.unwrap_or_default();

        for ap_path in aps {
            let ap = NMAccessPointProxy::builder(conn)
                .path(ap_path.clone())?
                .build()
                .await?;
            let ssid = util::ssid_to_string(ap.ssid().await.unwrap_or_default());
            if ssid.is_empty() {
                continue;
            }
            let signal = ap.strength().await.unwrap_or(0);
            let secured = ap.flags().await.unwrap_or(0) != 0
                || ap.wpa_flags().await.unwrap_or(0) != 0
                || ap.rsn_flags().await.unwrap_or(0) != 0;
            let connected = active_ap.as_ref().map(|p| p == &ap_path).unwrap_or(false);

            let frequency_mhz = ap.frequency().await.ok();
            let bssid = ap.hw_address().await.ok();
            let security = wifi_security_label(
                ap.flags().await.unwrap_or(0),
                ap.wpa_flags().await.unwrap_or(0),
                ap.rsn_flags().await.unwrap_or(0),
            );
            let password_required = secured;
            let is_existing = existing.contains(&ssid);

            let entry = WifiNetwork {
                ssid: ssid.clone(),
                signal,
                secured,
                connected,
                existing: is_existing,
                cached: !is_existing,
                password_required,
                security,
                frequency_mhz,
                bssid,
                last_seen_ms: None,
            };

            let needs_update = match seen.get(&ssid) {
                Some(existing) => {
                    if existing.connected {
                        false
                    } else if entry.connected {
                        true
                    } else {
                        entry.signal > existing.signal
                    }
                }
                None => true,
            };
            if needs_update {
                seen.insert(ssid, entry);
            }
        }
    }

    let mut out: Vec<WifiNetwork> = seen.into_values().collect();
    out.sort_by(|a, b| b.signal.cmp(&a.signal));
    Ok(out)
}

pub async fn request_wifi_scan(
    conn: &Connection,
    nm: &NetworkManagerProxy<'_>,
) -> Result<(), NetError> {
    for path in nm.get_devices().await? {
        let dev = NMDeviceProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;
        if dev.device_type().await.unwrap_or(0) != NM_DEVICE_TYPE_WIFI {
            continue;
        }
        let wifi = NMDeviceWirelessProxy::builder(conn)
            .path(path)?
            .build()
            .await?;
        let options: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
        let _ = wifi.request_scan(options).await;
    }
    Ok(())
}

pub async fn scan_wifi() -> Result<(), NetError> {
    let conn = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&conn).await?;
    request_wifi_scan(&conn, &nm).await
}

pub async fn connect_wifi(cfg: WifiConnectConfig) -> Result<(), NetError> {
    let conn = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&conn).await?;

    if !nm.wireless_enabled().await.unwrap_or(false) {
        return Err(NetError::Unsupported("wifi disabled".into()));
    }

    let devices = nm.get_devices().await?;
    let wifi_device = find_wifi_device(&conn, &devices)
        .await
        .ok_or_else(|| NetError::NotFound("wifi device".into()))?;

    let ap_path = find_wifi_ap(&conn, &wifi_device, &cfg.ssid)
        .await
        .ok_or_else(|| NetError::NotFound(format!("ssid '{}'", cfg.ssid)))?;

    let settings = build_wifi_settings(&cfg);
    nm.add_and_activate_connection(settings, wifi_device, ap_path)
        .await?;
    Ok(())
}

pub async fn disconnect_wifi() -> Result<(), NetError> {
    let conn = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&conn).await?;

    let devices = nm.get_devices().await?;
    let wifi_device = find_active_wifi_device(&conn, &devices)
        .await
        .ok_or_else(|| NetError::NotFound("active wifi connection".into()))?;

    let dev = NMDeviceProxy::builder(&conn)
        .path(wifi_device)?
        .build()
        .await?;
    let active = dev.active_connection().await?;
    if active.as_str() == "/" {
        return Err(NetError::NotFound("active wifi connection".into()));
    }

    nm.deactivate_connection(active).await?;
    Ok(())
}

pub async fn refresh_wifi_details(
    conn: &Connection,
    nm: &NetworkManagerProxy<'_>,
) -> Result<ActiveWifiDetails, NetError> {
    let devices = nm.get_devices().await?;
    let wifi_device = find_active_wifi_device(conn, &devices)
        .await
        .ok_or_else(|| NetError::NotFound("active wifi connection".into()))?;

    let dev = NMDeviceProxy::builder(conn)
        .path(wifi_device.clone())?
        .build()
        .await?;
    let iface = dev.interface().await.unwrap_or_default();

    let mut details = ActiveWifiDetails {
        ifname: if iface.is_empty() {
            None
        } else {
            Some(iface.clone())
        },
        ssid: None,
        signal: None,
        frequency_mhz: None,
        band: None,
        channel: None,
        rate_mbps: None,
        ip4: None,
        ip6: Vec::new(),
        gateway4: None,
        gateway6: Vec::new(),
        dns4: Vec::new(),
        dns6: Vec::new(),
        security: None,
        bssid: None,
        mac: None,
    };

    if let Ok(wifi) = NMDeviceWirelessProxy::builder(conn)
        .path(wifi_device.clone())?
        .build()
        .await
    {
        details.mac = wifi.hw_address().await.ok();
        if let Ok(active_ap) = wifi.active_access_point().await {
            if let Ok(ap) = NMAccessPointProxy::builder(conn)
                .path(active_ap)?
                .build()
                .await
            {
                let ssid = util::ssid_to_string(ap.ssid().await.unwrap_or_default());
                details.ssid = if ssid.is_empty() { None } else { Some(ssid) };
                details.signal = ap.strength().await.ok();
                let freq = ap.frequency().await.ok();
                details.frequency_mhz = freq;
                details.band = freq.and_then(util::frequency_band_label);
                details.channel = freq.and_then(util::frequency_channel);
                details.bssid = ap.hw_address().await.ok();
                let security = wifi_security_label(
                    ap.flags().await.unwrap_or(0),
                    ap.wpa_flags().await.unwrap_or(0),
                    ap.rsn_flags().await.unwrap_or(0),
                );
                details.security = Some(security);
            }
        }
    }

    if let Ok(ip4_path) = dev.ip4_config().await {
        if ip4_path.as_str() != "/" {
            let ip4 = NMIP4ConfigProxy::builder(conn)
                .path(ip4_path)?
                .build()
                .await?;
            if let Ok(addr) = ip4.address_data().await {
                if let Some(first) = addr.first() {
                    if let Some(value) = first.get("address") {
                        details.ip4 = util::owned_value_str(value);
                    }
                }
            }
            if let Ok(gw) = ip4.gateway().await {
                if !gw.is_empty() {
                    details.gateway4 = Some(gw);
                }
            }
            if let Ok(names) = ip4.nameserver_data().await {
                for entry in names {
                    if let Some(value) = entry.get("address") {
                        if let Some(addr) = util::owned_value_str(value) {
                            details.dns4.push(addr);
                        }
                    }
                }
            }
        }
    }

    if let Ok(ip6_path) = dev.ip6_config().await {
        if ip6_path.as_str() != "/" {
            let ip6 = NMIP6ConfigProxy::builder(conn)
                .path(ip6_path)?
                .build()
                .await?;
            if let Ok(addr) = ip6.address_data().await {
                for entry in addr {
                    if let Some(value) = entry.get("address") {
                        if let Some(addr) = util::owned_value_str(value) {
                            details.ip6.push(addr);
                        }
                    }
                }
            }
            if let Ok(gw) = ip6.gateway().await {
                if !gw.is_empty() {
                    details.gateway6.push(gw);
                }
            }
            if let Ok(names) = ip6.nameserver_data().await {
                for entry in names {
                    if let Some(value) = entry.get("address") {
                        if let Some(addr) = util::owned_value_str(value) {
                            details.dns6.push(addr);
                        }
                    }
                }
            }
        }
    }

    if details.ssid.is_none() && !iface.is_empty() {
        details.ssid = Some(iface);
    }

    Ok(details)
}

fn wifi_security_label(flags: u32, wpa_flags: u32, rsn_flags: u32) -> String {
    if flags == 0 && wpa_flags == 0 && rsn_flags == 0 {
        return "open".to_string();
    }
    if rsn_flags & 0x00100000 != 0 {
        return "wpa3".to_string();
    }
    if rsn_flags & 0x00000020 != 0 {
        return "wpa2-enterprise".to_string();
    }
    if rsn_flags != 0 {
        if rsn_flags & 0x0000000c != 0 {
            return "wpa2".to_string();
        }
        return "wpa2".to_string();
    }
    if wpa_flags != 0 {
        if wpa_flags & 0x00000010 != 0 {
            return "wpa".to_string();
        }
        return "wpa".to_string();
    }
    if flags != 0 {
        return "wep".to_string();
    }
    "secured".to_string()
}

fn build_wifi_settings(
    cfg: &WifiConnectConfig,
) -> HashMap<String, HashMap<String, zbus::zvariant::OwnedValue>> {
    let mut connection = HashMap::new();
    connection.insert("id".to_string(), util::owned_value(cfg.ssid.clone()));
    connection.insert(
        "type".to_string(),
        util::owned_value("802-11-wireless".to_string()),
    );
    connection.insert("autoconnect".to_string(), util::owned_value(true));
    if cfg.is_hidden {
        connection.insert(
            "interface-name".to_string(),
            util::owned_value(String::new()),
        );
    }

    let mut wifi = HashMap::new();
    wifi.insert(
        "ssid".to_string(),
        util::owned_value(cfg.ssid.as_bytes().to_vec()),
    );
    wifi.insert(
        "mode".to_string(),
        util::owned_value("infrastructure".to_string()),
    );
    if cfg.is_hidden {
        wifi.insert("hidden".to_string(), util::owned_value(true));
    }

    let mut ipv4 = HashMap::new();
    ipv4.insert("method".to_string(), util::owned_value("auto".to_string()));

    let mut ipv6 = HashMap::new();
    ipv6.insert("method".to_string(), util::owned_value("auto".to_string()));

    let mut settings = HashMap::new();
    settings.insert("connection".to_string(), connection);
    settings.insert("802-11-wireless".to_string(), wifi);
    settings.insert("ipv4".to_string(), ipv4);
    settings.insert("ipv6".to_string(), ipv6);

    let security_key = cfg.security_key.as_deref().unwrap_or("");
    match security_key {
        "sae" | "wpa3" => {
            if let Some(pwd) = &cfg.password {
                let mut security = HashMap::new();
                security.insert("key-mgmt".to_string(), util::owned_value("sae".to_string()));
                security.insert("psk".to_string(), util::owned_value(pwd.clone()));
                settings.insert("802-11-wireless-security".to_string(), security);
            }
        }
        "wep" => {
            if let Some(pwd) = &cfg.password {
                let mut security = HashMap::new();
                security.insert(
                    "key-mgmt".to_string(),
                    util::owned_value("none".to_string()),
                );
                security.insert("wep-key0".to_string(), util::owned_value(pwd.clone()));
                settings.insert("802-11-wireless-security".to_string(), security);
            }
        }
        "wpa-eap" | "wpa2-eap" | "wpa3-eap" | "ieee8021x" => {
            let mut security = HashMap::new();
            security.insert(
                "key-mgmt".to_string(),
                util::owned_value("wpa-eap".to_string()),
            );
            if let Some(pwd) = &cfg.password {
                security.insert("psk".to_string(), util::owned_value(pwd.clone()));
            }
            settings.insert("802-11-wireless-security".to_string(), security);

            let mut dot1x = HashMap::new();
            if let Some(e) = &cfg.eap {
                dot1x.insert("eap".to_string(), util::owned_value(vec![e.clone()]));
            }
            if let Some(p2) = &cfg.phase2 {
                dot1x.insert("phase2-auth".to_string(), util::owned_value(p2.clone()));
            }
            if let Some(id) = &cfg.identity {
                dot1x.insert("identity".to_string(), util::owned_value(id.clone()));
            }
            if let Some(pwd) = &cfg.password {
                dot1x.insert("password".to_string(), util::owned_value(pwd.clone()));
            }
            if let Some(anon) = &cfg.anon_identity {
                dot1x.insert(
                    "anonymous-identity".to_string(),
                    util::owned_value(anon.clone()),
                );
            }
            if let Some(cert) = &cfg.ca_cert {
                dot1x.insert("ca-cert".to_string(), util::owned_value(cert.clone()));
            }
            settings.insert("802-1x".to_string(), dot1x);
        }
        "wpa-psk" | "wpa2-psk" | "" => {
            if let Some(pwd) = &cfg.password {
                let mut security = HashMap::new();
                security.insert(
                    "key-mgmt".to_string(),
                    util::owned_value("wpa-psk".to_string()),
                );
                security.insert("psk".to_string(), util::owned_value(pwd.clone()));
                settings.insert("802-11-wireless-security".to_string(), security);
            }
        }
        "open" => {}
        _ => {
            if let Some(pwd) = &cfg.password {
                let mut security = HashMap::new();
                security.insert(
                    "key-mgmt".to_string(),
                    util::owned_value("wpa-psk".to_string()),
                );
                security.insert("psk".to_string(), util::owned_value(pwd.clone()));
                settings.insert("802-11-wireless-security".to_string(), security);
            }
        }
    }

    settings
}

async fn list_known_wifi_ssids(
    conn: &Connection,
    _nm: &NetworkManagerProxy<'_>,
) -> Result<Vec<String>, NetError> {
    use crate::network::dbus::NMSettingsProxy;

    let settings = NMSettingsProxy::new(conn).await?;
    let paths = settings.list_connections().await?;
    let mut ssids = Vec::new();

    for path in paths {
        if let Ok(conn_obj) = NMSettingsConnectionProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await
        {
            if let Ok(settings_map) = conn_obj.get_settings().await {
                if let Some(wireless) = settings_map.get("802-11-wireless") {
                    if let Some(value) = wireless.get("ssid") {
                        if let Some(ssid) = ssid_from_value(value) {
                            ssids.push(ssid);
                        }
                    }
                }
            }
        }
    }

    Ok(ssids)
}

fn ssid_from_value(value: &zbus::zvariant::OwnedValue) -> Option<String> {
    util::ssid_from_value(value)
}

async fn find_wifi_device(
    conn: &Connection,
    devices: &[zbus::zvariant::OwnedObjectPath],
) -> Option<zbus::zvariant::OwnedObjectPath> {
    for path in devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(path.clone())
            .ok()?
            .build()
            .await
            .ok()?;
        if dev.device_type().await.unwrap_or(0) != NM_DEVICE_TYPE_WIFI {
            continue;
        }
        let state = dev.state().await.unwrap_or(0);
        if state == 100 || state == 70 || state == 30 {
            return Some(path.clone());
        }
    }
    None
}

async fn find_active_wifi_device(
    conn: &Connection,
    devices: &[zbus::zvariant::OwnedObjectPath],
) -> Option<zbus::zvariant::OwnedObjectPath> {
    for path in devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(path.clone())
            .ok()?
            .build()
            .await
            .ok()?;
        if dev.device_type().await.unwrap_or(0) != NM_DEVICE_TYPE_WIFI {
            continue;
        }
        let active = dev.active_connection().await.ok()?;
        if active.as_str() != "/" {
            return Some(path.clone());
        }
    }
    None
}

async fn find_wifi_ap(
    conn: &Connection,
    wifi_device: &zbus::zvariant::OwnedObjectPath,
    ssid: &str,
) -> Option<zbus::zvariant::OwnedObjectPath> {
    let wifi = NMDeviceWirelessProxy::builder(conn)
        .path(wifi_device.clone())
        .ok()?
        .build()
        .await
        .ok()?;
    let aps = wifi.access_points().await.ok()?;
    for ap_path in aps {
        let ap = NMAccessPointProxy::builder(conn)
            .path(ap_path.clone())
            .ok()?
            .build()
            .await
            .ok()?;
        let ap_ssid = util::ssid_to_string(ap.ssid().await.unwrap_or_default());
        if ap_ssid == ssid {
            return Some(ap_path);
        }
    }
    None
}
