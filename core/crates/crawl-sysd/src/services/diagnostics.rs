use crawl_ipc::protocol::CrawlResponse;

use crate::audio;
use crate::bluetooth;
use crate::brightness;
use crate::network;
use crate::state::SharedState;

async fn audio_ok(state: &SharedState) -> bool {
    let cfg = state.config.audio.clone();
    audio::list_sinks(&cfg).await.is_ok() && audio::list_sources(&cfg).await.is_ok()
}

async fn bt_ok(state: &SharedState) -> bool {
    match state.bluetooth.as_deref() {
        Some(rt) => bluetooth::get_status(rt).await.is_ok(),
        None => false,
    }
}

async fn bt_status(state: &SharedState) -> Option<String> {
    match state.bluetooth.as_deref() {
        Some(rt) => bluetooth::get_status(rt).await.ok().map(|_| "ok".into()),
        None => None,
    }
}

async fn brightness_ok(state: &SharedState) -> bool {
    match brightness::Backlight::open_async(&state.config.brightness).await {
        Ok(b) => b.status_async().await.is_ok(),
        Err(_) => false,
    }
}

async fn network_ok() -> bool {
    network::get_status().await.is_ok()
}

pub async fn handle_health(state: SharedState, id: Option<serde_json::Value>) -> CrawlResponse {
    let (audio, bt, brightness, network) = tokio::join!(
        audio_ok(&state),
        bt_ok(&state),
        brightness_ok(&state),
        network_ok(),
    );

    CrawlResponse::success(
        id,
        serde_json::json!({
            "audio": audio,
            "bluetooth": bt,
            "brightness": brightness,
            "network": network,
            "ok": audio && bt && brightness && network,
        }),
    )
}

pub async fn handle_status(state: SharedState, id: Option<serde_json::Value>) -> CrawlResponse {
    let (audio, bt_present, brightness, network) = tokio::join!(
        audio_ok(&state),
        bt_status(&state),
        brightness_ok(&state),
        network_ok(),
    );

    CrawlResponse::success(
        id,
        serde_json::json!({
            "audio": { "ok": audio },
            "bluetooth": { "ok": bt_present.is_some() },
            "brightness": { "ok": brightness },
            "network": { "ok": network },
        }),
    )
}
