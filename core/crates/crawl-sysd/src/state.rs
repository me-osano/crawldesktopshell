use crate::bluetooth::BluetoothRuntime;
use crate::config::Config;
use crate::daemon::EventBus;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub event_bus: EventBus,
    pub bluetooth: Option<Arc<BluetoothRuntime>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new(
        config: Config,
        event_bus: EventBus,
        bluetooth: Option<Arc<BluetoothRuntime>>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            event_bus,
            bluetooth,
        }
    }
}
