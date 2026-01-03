//! Trades and WebSocket management state

use {super::trade::EventTrades, std::collections::HashMap, tokio::task::JoinHandle};

/// Trades and WebSocket management state
#[derive(Debug)]
pub struct TradesState {
    // Map from event slug to trades
    pub event_trades: HashMap<String, EventTrades>,
    // Map from event slug to websocket task handle
    pub ws_handles: HashMap<String, JoinHandle<()>>,
}

impl TradesState {
    pub fn new() -> Self {
        Self {
            event_trades: HashMap::new(),
            ws_handles: HashMap::new(),
        }
    }
}
