//! State types for the trending TUI
//!
//! This module contains all state types used by the TUI, organized into submodules:
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

use {
    polymarket_api::gamma::Event, ratatui::widgets::TableState, std::collections::HashMap,
    tokio::task::JoinHandle,
};

/// Main application state
pub struct TrendingAppState {
    pub events: Vec<Event>,
    pub should_quit: bool,
    pub search: SearchState,
    pub scroll: ScrollState,
    pub pagination: PaginationState,
    pub logs: LogsState,
    pub navigation: NavigationState,
    pub trades: TradesState,
    pub event_filter: EventFilter, // Current filter (Trending, Breaking)
    pub market_prices: HashMap<String, f64>, // asset_id -> current price from API
    pub event_trade_counts: HashMap<String, usize>, // event_slug -> total trade count from API
    pub has_clob_auth: bool,       // Whether CLOB API authentication is available
    pub popup: Option<PopupType>,  // Currently active popup/modal
    pub trades_table_state: TableState, // State for trades table selection
    pub events_cache: HashMap<EventFilter, Vec<Event>>, // Cache for each filter tab
    /// Global event cache keyed by slug - single source of truth for event data
    pub event_cache: HashMap<String, Event>,
    pub show_logs: bool,   // Whether to show the logs panel (toggle with 'l')
    pub main_tab: MainTab, // Current main tab (Trending vs Yield)
    pub yield_state: YieldState, // State for the Yield tab
    pub favorites_state: FavoritesState, // State for the Favorites tab
    pub auth_state: AuthState, // Authentication state
    pub login_form: LoginFormState, // Login form state
    pub trade_form: Option<TradeFormState>, // Trade form state (when trade popup is open)
    pub event_sort_by: EventSortBy, // Current sort option for events list
    pub gamma_api_status: Option<bool>, /* Gamma API health: Some(true) = healthy, Some(false) = unhealthy, None = unknown */
    pub data_api_status: Option<bool>, /* Data API health: Some(true) = healthy, Some(false) = unhealthy, None = unknown */
    pub orderbook_state: OrderbookState, // Orderbook panel state
}

impl TrendingAppState {
    pub fn new(events: Vec<Event>, order_by: String, ascending: bool, has_clob_auth: bool) -> Self {
        let current_limit = events.len();
        // Determine initial filter based on order_by
        let event_filter = if order_by == "startDate"
            || order_by == "startTime"
            || order_by == "oneDayPriceChange"
        {
            EventFilter::Breaking
        } else {
            EventFilter::Trending
        };
        // Initialize cache with the initial events for the current filter
        let mut events_cache = HashMap::new();
        events_cache.insert(event_filter, events.clone());
        // Initialize global event cache with initial events
        let mut event_cache = HashMap::new();
        for event in &events {
            event_cache.insert(event.slug.clone(), event.clone());
        }
        Self {
            events,
            should_quit: false,
            search: SearchState::new(),
            scroll: ScrollState::new(),
            pagination: PaginationState::new(order_by, ascending, current_limit),
            logs: LogsState::new(),
            navigation: NavigationState::new(),
            trades: TradesState::new(),
            event_filter,
            market_prices: HashMap::new(),
            event_trade_counts: HashMap::new(),
            has_clob_auth,
            popup: None,
            trades_table_state: TableState::default(),
            events_cache,
            event_cache,
            show_logs: false, // Hidden by default
            main_tab: MainTab::Trending,
            yield_state: YieldState::new(),
            favorites_state: FavoritesState::new(),
            auth_state: AuthState::new(),
            login_form: LoginFormState::new(),
            trade_form: None,
            event_sort_by: EventSortBy::default(),
            gamma_api_status: None,
            data_api_status: None,
            orderbook_state: OrderbookState::new(),
        }
    }

    /// Add events to the global cache
    pub fn cache_events(&mut self, events: &[Event]) {
        for event in events {
            self.event_cache.insert(event.slug.clone(), event.clone());
        }
    }

    /// Get an event from the global cache by slug
    pub fn get_cached_event(&self, slug: &str) -> Option<&Event> {
        self.event_cache.get(slug)
    }

    /// Sort events by the current sort option
    pub fn sort_events(&mut self) {
        match self.event_sort_by {
            EventSortBy::Volume24hr => {
                self.events.sort_by(|a, b| {
                    b.volume_24hr
                        .partial_cmp(&a.volume_24hr)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            },
            EventSortBy::VolumeTotal => {
                self.events.sort_by(|a, b| {
                    b.volume
                        .partial_cmp(&a.volume)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            },
            EventSortBy::Liquidity => {
                self.events.sort_by(|a, b| {
                    b.liquidity
                        .partial_cmp(&a.liquidity)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            },
            EventSortBy::Newest => {
                // Sort by created_at descending (newest first)
                self.events
                    .sort_by(|a, b| match (&b.created_at, &a.created_at) {
                        (Some(b_date), Some(a_date)) => b_date.cmp(a_date),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    });
            },
            EventSortBy::EndingSoon => {
                // Sort by end_date ascending (soonest first), None at end
                self.events
                    .sort_by(|a, b| match (&a.end_date, &b.end_date) {
                        (Some(a_date), Some(b_date)) => a_date.cmp(b_date),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    });
            },
            EventSortBy::Competitive => {
                // Sort by competitive score descending (most competitive first)
                self.events.sort_by(|a, b| {
                    b.competitive
                        .partial_cmp(&a.competitive)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            },
        }
    }

    /// Show a popup
    pub fn show_popup(&mut self, popup: PopupType) {
        self.popup = Some(popup);
    }

    /// Close the active popup
    pub fn close_popup(&mut self) {
        self.popup = None;
        // Clear trade form when closing trade popup
        self.trade_form = None;
    }

    /// Open trade popup for a specific market
    pub fn open_trade_popup(
        &mut self,
        token_id: String,
        market_question: String,
        outcome: String,
        price: f64,
    ) {
        self.trade_form = Some(TradeFormState::new(
            token_id,
            market_question,
            outcome,
            price,
        ));
        self.popup = Some(PopupType::Trade);
    }

    /// Check if a popup is active
    pub fn has_popup(&self) -> bool {
        self.popup.is_some()
    }

    /// Check if we need to fetch more events (when user is near the end)
    pub fn should_fetch_more(&self) -> bool {
        // Only fetch more if not in search/filter mode and not already fetching
        if self.search.is_active()
            || !self.search.query.is_empty()
            || self.pagination.is_fetching_more
        {
            return false;
        }

        let filtered_len = self.filtered_events().len();
        // Fetch more when user is within 5 items of the end
        self.navigation.selected_index >= filtered_len.saturating_sub(5)
            && filtered_len >= self.pagination.current_limit
    }

    #[allow(dead_code)]
    pub fn add_log(&mut self, level: &str, message: String) {
        // Format: [LEVEL] message
        let formatted = format!("[{}] {}", level, message);
        self.logs.messages.push(formatted);
        // Keep only last 1000 logs
        if self.logs.messages.len() > 1000 {
            self.logs.messages.remove(0);
        }
        // Auto-scroll to bottom - always show the latest logs
        // The logs area is Constraint::Length(8), so visible height is ~6 lines (minus borders)
        // We'll set scroll to show from the bottom, and render_logs will adjust if needed
        let estimated_visible_height = 6; // Approximate visible lines (8 - 2 for borders)
        if self.logs.messages.len() > estimated_visible_height {
            self.logs.scroll = self.logs.messages.len() - estimated_visible_height;
        } else {
            self.logs.scroll = 0;
        }
    }

    /// Get filtered events based on search query
    /// If in local filter mode, always filter locally from current list
    /// If in API search mode and results are available, use those
    /// Otherwise filter locally
    /// For Favorites tab, returns favorites events (search not supported yet)
    pub fn filtered_events(&self) -> Vec<&Event> {
        // For Favorites tab, just return favorites events (no search support yet)
        if self.main_tab == MainTab::Favorites {
            return self.favorites_state.events.iter().collect();
        }

        if self.search.query.is_empty() {
            // No query, return all events from the current source
            // If we have search results and not in local filter mode, return those; otherwise return all events
            if !self.search.results.is_empty()
                && self.search.mode != SearchMode::LocalFilter
                && self.search.mode == SearchMode::ApiSearch
            {
                return self.search.results.iter().collect();
            }
            return self.events.iter().collect();
        }

        // If in local filter mode, always filter from the source list
        if self.search.mode == SearchMode::LocalFilter {
            // Determine source list: if we have search results, filter from those; otherwise filter from events
            let query_lower = self.search.query.to_lowercase();
            if !self.search.results.is_empty() {
                // Filter from search results (current displayed list)
                return self
                    .search
                    .results
                    .iter()
                    .filter(|event| {
                        event.title.to_lowercase().contains(&query_lower)
                            || event.slug.to_lowercase().contains(&query_lower)
                            || event
                                .tags
                                .iter()
                                .any(|tag| tag.label.to_lowercase().contains(&query_lower))
                            || event
                                .markets
                                .iter()
                                .any(|market| market.question.to_lowercase().contains(&query_lower))
                    })
                    .collect();
            } else {
                // Filter from events list
                return self
                    .events
                    .iter()
                    .filter(|event| {
                        event.title.to_lowercase().contains(&query_lower)
                            || event.slug.to_lowercase().contains(&query_lower)
                            || event
                                .tags
                                .iter()
                                .any(|tag| tag.label.to_lowercase().contains(&query_lower))
                            || event
                                .markets
                                .iter()
                                .any(|market| market.question.to_lowercase().contains(&query_lower))
                    })
                    .collect();
            }
        }

        // API search mode: use API results if available
        if !self.search.results.is_empty() && self.search.query == self.search.last_searched_query {
            return self.search.results.iter().collect();
        }

        // Fall back to local filtering
        let query_lower = self.search.query.to_lowercase();
        self.events
            .iter()
            .filter(|event| {
                event.title.to_lowercase().contains(&query_lower)
                    || event.slug.to_lowercase().contains(&query_lower)
                    || event
                        .tags
                        .iter()
                        .any(|tag| tag.label.to_lowercase().contains(&query_lower))
                    || event
                        .markets
                        .iter()
                        .any(|market| market.question.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get the currently selected event from filtered list
    /// Uses current_selected_index() to be tab-aware
    pub fn selected_event_filtered(&self) -> Option<&Event> {
        let filtered = self.filtered_events();
        let selected_idx = self.current_selected_index();
        filtered.get(selected_idx).copied()
    }

    pub fn enter_search_mode(&mut self) {
        self.search.mode = SearchMode::ApiSearch;
        self.search.query.clear();
    }

    pub fn enter_local_filter_mode(&mut self) {
        self.search.mode = SearchMode::LocalFilter;
        self.search.query.clear();
    }

    pub fn exit_search_mode(&mut self) {
        self.search.mode = SearchMode::None;
        self.search.query.clear();
        self.search.results.clear();
        self.search.last_searched_query.clear();
        self.navigation.selected_index = 0;
        self.scroll.events_list = 0;
    }

    pub fn is_in_filter_mode(&self) -> bool {
        self.search.is_active()
    }

    pub fn add_search_char(&mut self, c: char) {
        self.search.query.push(c);
        self.navigation.selected_index = 0;
        self.scroll.events_list = 0;
    }

    pub fn delete_search_char(&mut self) {
        self.search.query.pop();
        self.navigation.selected_index = 0;
        self.scroll.events_list = 0;
        // Clear search results when query changes
        if self.search.query != self.search.last_searched_query {
            self.search.results.clear();
        }
    }

    pub fn set_search_results(&mut self, results: Vec<Event>, query: String) {
        self.search.results = results;
        self.search.last_searched_query = query;
        self.search.is_searching = false;
        self.navigation.selected_index = 0;
        self.scroll.events_list = 0;
    }

    pub fn set_searching(&mut self, searching: bool) {
        self.search.is_searching = searching;
    }

    pub fn selected_event(&self) -> Option<&Event> {
        // Always use filtered events to ensure we get the event from the currently displayed list
        // This works for:
        // - Local filter mode (filters current list)
        // - API search mode (uses search results)
        // - Normal mode (uses all events)
        // Using filtered_events() ensures consistency between what's displayed and what's selected
        self.selected_event_filtered()
    }

    pub fn selected_event_slug(&self) -> Option<String> {
        self.selected_event().map(|e| e.slug.clone())
    }

    /// Get the current tab's selected index
    pub fn current_selected_index(&self) -> usize {
        match self.main_tab {
            MainTab::Favorites => self.favorites_state.selected_index,
            _ => self.navigation.selected_index,
        }
    }

    /// Get the current tab's scroll position for the events list
    pub fn current_events_scroll(&self) -> usize {
        match self.main_tab {
            MainTab::Favorites => self.favorites_state.scroll,
            _ => self.scroll.events_list,
        }
    }

    /// Get events for the current tab (without filtering)
    #[allow(dead_code)]
    pub fn current_events(&self) -> Vec<&Event> {
        match self.main_tab {
            MainTab::Favorites => self.favorites_state.events.iter().collect(),
            _ => self.events.iter().collect(),
        }
    }

    pub fn move_up(&mut self) {
        if self.navigation.selected_index > 0 {
            self.navigation.selected_index -= 1;
            if self.navigation.selected_index < self.scroll.events_list {
                self.scroll.events_list = self.navigation.selected_index;
            }
            // Reset markets scroll when changing events
            self.scroll.markets = 0;
        }
    }

    pub fn move_down(&mut self) {
        let filtered_len = self.filtered_events().len();
        if self.navigation.selected_index < filtered_len.saturating_sub(1) {
            self.navigation.selected_index += 1;
            let visible_height = 20;
            if self.navigation.selected_index >= self.scroll.events_list + visible_height {
                self.scroll.events_list = self.navigation.selected_index - visible_height + 1;
            }
            // Reset markets scroll when changing events
            self.scroll.markets = 0;
        }
    }

    pub fn is_watching(&self, event_slug: &str) -> bool {
        self.trades
            .event_trades
            .get(event_slug)
            .map(|et| et.is_watching)
            .unwrap_or(false)
    }

    pub fn get_trades(&self, event_slug: &str) -> &[Trade] {
        self.trades
            .event_trades
            .get(event_slug)
            .map(|et| et.trades.as_slice())
            .unwrap_or(&[])
    }

    pub fn start_watching(&mut self, event_slug: String, ws_handle: JoinHandle<()>) {
        self.trades
            .event_trades
            .entry(event_slug.clone())
            .or_insert_with(EventTrades::new)
            .is_watching = true;
        self.trades.ws_handles.insert(event_slug, ws_handle);
    }

    pub fn stop_watching(&mut self, event_slug: &str) {
        if let Some(handle) = self.trades.ws_handles.remove(event_slug) {
            handle.abort();
        }
        if let Some(event_trades) = self.trades.event_trades.get_mut(event_slug) {
            event_trades.is_watching = false;
        }
    }

    pub fn cleanup(&mut self) {
        for handle in self.trades.ws_handles.values() {
            handle.abort();
        }
        self.trades.ws_handles.clear();
    }
}
