//! Orderbook state types

/// Orderbook outcome tab (Yes or No)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderbookOutcome {
    #[default]
    Yes,
    No,
}

impl OrderbookOutcome {
    pub fn toggle(&self) -> Self {
        match self {
            OrderbookOutcome::Yes => OrderbookOutcome::No,
            OrderbookOutcome::No => OrderbookOutcome::Yes,
        }
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            OrderbookOutcome::Yes => "Yes",
            OrderbookOutcome::No => "No",
        }
    }
}

/// A price level in the orderbook
#[derive(Debug, Clone)]
pub struct OrderbookLevel {
    pub price: f64,
    pub size: f64,
    pub total: f64, // cumulative total (running sum of price * size from best price)
}

/// Orderbook data for a market
#[derive(Debug, Clone, Default)]
pub struct OrderbookData {
    pub asks: Vec<OrderbookLevel>,
    pub bids: Vec<OrderbookLevel>,
    pub spread: Option<f64>,
    #[allow(dead_code)]
    pub last_price: Option<f64>,
}

/// State for the orderbook panel
#[derive(Debug)]
pub struct OrderbookState {
    pub selected_market_index: usize, // Which market in the current event is selected
    pub selected_outcome: OrderbookOutcome, // Yes or No tab
    pub orderbook: Option<OrderbookData>, // Current orderbook data
    pub is_loading: bool,
    pub last_fetch: Option<std::time::Instant>,
    pub token_id: Option<String>, // Current token ID being displayed
    pub last_height: u16,         // Last rendered height to prevent jumps during loading
}

impl OrderbookState {
    pub fn new() -> Self {
        Self {
            selected_market_index: 0,
            selected_outcome: OrderbookOutcome::default(),
            orderbook: None,
            is_loading: false,
            last_fetch: None,
            token_id: None,
            last_height: 5, // Start with min height
        }
    }

    pub fn reset(&mut self) {
        self.selected_market_index = 0;
        self.orderbook = None;
        self.is_loading = false;
        self.token_id = None;
    }

    pub fn toggle_outcome(&mut self) {
        self.selected_outcome = self.selected_outcome.toggle();
        // Clear orderbook data when switching outcomes
        self.orderbook = None;
        self.token_id = None;
    }

    pub fn needs_refresh(&self) -> bool {
        match self.last_fetch {
            Some(last) => last.elapsed() >= std::time::Duration::from_secs(5),
            None => true,
        }
    }
}

impl Default for OrderbookState {
    fn default() -> Self {
        Self::new()
    }
}
