//! Navigation and focus state types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Header,       // Top panel with filter options
    EventsList,   // Left panel with events
    EventDetails, // Right panel - event details
    Markets,      // Right panel - markets
    Trades,       // Right panel - trades
    Logs,         // Bottom panel - logs
}

/// Main tab at the top level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainTab {
    Trending,
    Favorites,
    Yield,
}

impl MainTab {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            MainTab::Trending => "Trending",
            MainTab::Favorites => "Favorites",
            MainTab::Yield => "Yield",
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            MainTab::Trending => MainTab::Favorites,
            MainTab::Favorites => MainTab::Yield,
            MainTab::Yield => MainTab::Trending,
        }
    }

    #[allow(dead_code)]
    pub fn prev(&self) -> Self {
        match self {
            MainTab::Trending => MainTab::Yield,
            MainTab::Favorites => MainTab::Trending,
            MainTab::Yield => MainTab::Favorites,
        }
    }
}

/// Event filter type for different views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventFilter {
    Trending, // Order by volume24hr (default)
    Breaking, // Order by oneDayPriceChange (biggest movers)
}

impl EventFilter {
    pub fn order_by(&self) -> &'static str {
        match self {
            EventFilter::Trending => "volume24hr",
            EventFilter::Breaking => "oneDayPriceChange",
        }
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            EventFilter::Trending => "Events",
            EventFilter::Breaking => "Breaking",
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            EventFilter::Trending => EventFilter::Breaking,
            EventFilter::Breaking => EventFilter::Trending,
        }
    }

    #[allow(dead_code)]
    pub fn prev(&self) -> Self {
        match self {
            EventFilter::Trending => EventFilter::Breaking,
            EventFilter::Breaking => EventFilter::Trending,
        }
    }
}

/// Sort options for events list (matches Polymarket website options)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventSortBy {
    #[default]
    Volume24hr, // 24h Volume (default for Trending)
    VolumeTotal, // Total Volume
    Liquidity,   // Liquidity
    Newest,      // Newest (by created date)
    EndingSoon,  // Ending Soon
    Competitive, // Competitive (closer odds)
}

impl EventSortBy {
    pub fn label(&self) -> &'static str {
        match self {
            EventSortBy::Volume24hr => "24h Vol",
            EventSortBy::VolumeTotal => "Total Vol",
            EventSortBy::Liquidity => "Liquidity",
            EventSortBy::Newest => "Newest",
            EventSortBy::EndingSoon => "Ending Soon",
            EventSortBy::Competitive => "Competitive",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            EventSortBy::Volume24hr => EventSortBy::VolumeTotal,
            EventSortBy::VolumeTotal => EventSortBy::Liquidity,
            EventSortBy::Liquidity => EventSortBy::Newest,
            EventSortBy::Newest => EventSortBy::EndingSoon,
            EventSortBy::EndingSoon => EventSortBy::Competitive,
            EventSortBy::Competitive => EventSortBy::Volume24hr,
        }
    }

    /// Get the API order parameter for this sort option
    #[allow(dead_code)]
    pub fn api_order_param(&self) -> &'static str {
        match self {
            EventSortBy::Volume24hr => "volume24hr",
            EventSortBy::VolumeTotal => "volume",
            EventSortBy::Liquidity => "liquidity",
            EventSortBy::Newest => "createdAt",
            EventSortBy::EndingSoon => "endDate",
            EventSortBy::Competitive => "competitive",
        }
    }

    /// Whether this sort should be ascending (true) or descending (false)
    #[allow(dead_code)]
    pub fn is_ascending(&self) -> bool {
        match self {
            EventSortBy::EndingSoon => true, // Soonest first
            EventSortBy::Newest => false,    // Most recent first
            _ => false,                      // Highest values first
        }
    }
}

/// Scroll positions for all panels
#[derive(Debug)]
pub struct ScrollState {
    pub events_list: usize,   // Scroll position for events list
    pub markets: usize,       // Scroll position for markets panel
    pub trades: usize,        // Scroll position for trades table
    pub event_details: usize, // Scroll position for event details
    #[allow(dead_code)]
    pub logs: usize, // Scroll position for logs panel
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            events_list: 0,
            markets: 0,
            trades: 0,
            event_details: 0,
            logs: 0,
        }
    }
}

/// Navigation state (selection and focus)
#[derive(Debug)]
pub struct NavigationState {
    pub selected_index: usize,
    pub focused_panel: FocusedPanel,
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            focused_panel: FocusedPanel::EventsList, // Start with events list focused
        }
    }
}
