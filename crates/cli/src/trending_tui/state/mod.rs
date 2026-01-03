//! State types for the trending TUI
//!
//! This module contains all state types used by the TUI, organized into submodules:
//! - `app_state`: Main application state (TrendingAppState)
//! - `auth`: Authentication state (AuthState, LoginFormState, etc.)
//! - `favorites`: Favorites tab state
//! - `logs`: Logs panel state
//! - `navigation`: Navigation, focus, and scroll state
//! - `orderbook`: Orderbook panel state
//! - `pagination`: Pagination state for infinite scrolling
//! - `popup`: Popup/modal types
//! - `search`: Search state
//! - `trade`: Trade form and trade data types
//! - `trades_ws`: WebSocket trade management state
//! - `yield_state`: Yield tab state

mod app_state;
mod auth;
mod favorites;
mod logs;
mod navigation;
mod orderbook;
mod pagination;
mod popup;
mod search;
mod trade;
mod trades_ws;
mod yield_state;

// Re-export all public types
pub use {
    app_state::TrendingAppState,
    auth::{AuthState, LoginField, LoginFormState, UserProfile},
    favorites::FavoritesState,
    logs::LogsState,
    navigation::{EventFilter, EventSortBy, FocusedPanel, MainTab, NavigationState, ScrollState},
    orderbook::{OrderbookData, OrderbookLevel, OrderbookOutcome, OrderbookState},
    pagination::PaginationState,
    popup::PopupType,
    search::{SearchMode, SearchState},
    trade::{EventTrades, Trade, TradeField, TradeFormState, TradeSide},
    trades_ws::TradesState,
    yield_state::{YieldOpportunity, YieldSearchResult, YieldState},
};
