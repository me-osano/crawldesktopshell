//! Shared utility functions for the network module.

use zbus::zvariant::{OwnedValue, Value};

pub fn ssid_to_string(bytes: Vec<u8>) -> String {
    String::from_utf8_lossy(&bytes)
        .trim_matches(char::from(0))
        .to_string()
}

pub fn ssid_from_value(value: &OwnedValue) -> Option<String> {
    let s = owned_value_str(value)?;
    if s.is_empty() { None } else { Some(s) }
}

pub fn owned_value_str(value: &OwnedValue) -> Option<String> {
    if let Ok(s) = value.downcast_ref::<zbus::zvariant::Str>() {
        return Some(s.as_str().to_string());
    }
    if let Ok(s) = value.downcast_ref::<String>() {
        return Some(s.clone());
    }
    None
}

pub fn owned_value<V>(value: V) -> OwnedValue
where
    V: Into<Value<'static>>,
{
    OwnedValue::try_from(value.into()).expect("owned value conversion should not fail")
}

pub fn frequency_band_label(freq: u32) -> Option<String> {
    match freq {
        2400..=2499 => Some("2.4GHz".to_string()),
        5000..=5899 => Some("5GHz".to_string()),
        5925..=7125 => Some("6GHz".to_string()),
        _ => None,
    }
}

pub fn frequency_channel(freq: u32) -> Option<u32> {
    if (2412..=2472).contains(&freq) {
        return Some(((freq - 2407) / 5) as u32);
    }
    if freq == 2484 {
        return Some(14);
    }
    if (5000..=5895).contains(&freq) {
        return Some(((freq - 5000) / 5) as u32);
    }
    if (5925..=7125).contains(&freq) {
        return Some(((freq - 5950) / 5) as u32);
    }
    None
}
