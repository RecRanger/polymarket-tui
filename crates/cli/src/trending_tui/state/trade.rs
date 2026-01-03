//! Trade-related state types

use polymarket_api::rtds::RTDSMessage;

#[derive(Debug)]
pub struct Trade {
    pub timestamp: i64,
    pub side: String,
    pub outcome: String,
    pub price: f64,
    pub shares: f64,
    pub total_value: f64,
    pub title: String,
    pub asset_id: String,
    pub user: String,
    #[allow(dead_code)]
    pub pseudonym: String,
}

#[derive(Debug)]
pub struct EventTrades {
    pub trades: Vec<Trade>,
    pub is_watching: bool,
}

impl EventTrades {
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
            is_watching: false,
        }
    }

    pub fn add_trade(&mut self, msg: &RTDSMessage) {
        let rounded_shares = (msg.payload.size * 100.0).round() / 100.0;
        let total_value = msg.payload.price * msg.payload.size;

        let trade = Trade {
            timestamp: msg.payload.timestamp,
            side: msg.payload.side.clone(),
            outcome: msg.payload.outcome.clone(),
            price: msg.payload.price,
            shares: rounded_shares,
            total_value,
            title: msg.payload.title.clone(),
            asset_id: msg.payload.asset.clone(),
            user: msg.payload.name.clone(),
            pseudonym: msg.payload.pseudonym.clone(),
        };

        self.trades.insert(0, trade);
        // Keep only the last 500 trades per event
        if self.trades.len() > 500 {
            self.trades.truncate(500);
        }
    }
}

/// Trade side (Buy or Sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

impl TradeSide {
    #[allow(dead_code)]
    pub fn toggle(&self) -> Self {
        match self {
            TradeSide::Buy => TradeSide::Sell,
            TradeSide::Sell => TradeSide::Buy,
        }
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            TradeSide::Buy => "BUY",
            TradeSide::Sell => "SELL",
        }
    }
}

/// Order type (Limit or Market)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderType {
    #[default]
    Limit,
    Market,
}

impl OrderType {
    pub fn toggle(&self) -> Self {
        match self {
            OrderType::Limit => OrderType::Market,
            OrderType::Market => OrderType::Limit,
        }
    }

    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            OrderType::Limit => "LIMIT",
            OrderType::Market => "MARKET",
        }
    }
}

/// Trade form field being edited
/// Note: Side is now controlled via clickable title tabs, not a field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeField {
    OrderType,
    LimitPrice,
    Shares,
    Amount,
}

impl TradeField {
    /// Get next field based on order type
    pub fn next(&self, order_type: OrderType) -> Self {
        match order_type {
            OrderType::Limit => match self {
                TradeField::OrderType => TradeField::LimitPrice,
                TradeField::LimitPrice => TradeField::Shares,
                TradeField::Shares => TradeField::OrderType,
                TradeField::Amount => TradeField::OrderType, // Shouldn't happen in limit mode
            },
            OrderType::Market => match self {
                TradeField::OrderType => TradeField::Amount,
                TradeField::Amount => TradeField::OrderType,
                TradeField::LimitPrice => TradeField::OrderType, // Shouldn't happen in market mode
                TradeField::Shares => TradeField::OrderType,     // Shouldn't happen in market mode
            },
        }
    }

    /// Get previous field based on order type
    pub fn prev(&self, order_type: OrderType) -> Self {
        match order_type {
            OrderType::Limit => match self {
                TradeField::OrderType => TradeField::Shares,
                TradeField::LimitPrice => TradeField::OrderType,
                TradeField::Shares => TradeField::LimitPrice,
                TradeField::Amount => TradeField::Shares, // Shouldn't happen in limit mode
            },
            OrderType::Market => match self {
                TradeField::OrderType => TradeField::Amount,
                TradeField::Amount => TradeField::OrderType,
                TradeField::LimitPrice => TradeField::OrderType, // Shouldn't happen in market mode
                TradeField::Shares => TradeField::OrderType,     // Shouldn't happen in market mode
            },
        }
    }
}

/// Outcome with its token ID and price
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OutcomeInfo {
    pub name: String,
    pub token_id: String,
    pub price: f64,
}

/// Trade form state
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TradeFormState {
    pub market_question: String,
    pub outcomes: Vec<OutcomeInfo>, // All outcomes with their token IDs and prices
    pub selected_outcome_idx: usize, // Index of currently selected outcome
    pub side: TradeSide,
    pub order_type: OrderType,
    pub limit_price: f64, // Limit price (for limit orders)
    pub shares: String,   // Number of shares (input as string for editing, for limit orders)
    pub amount: String,   // Amount in dollars (input as string for editing, for market orders)
    pub active_field: TradeField,
    pub error_message: Option<String>,
    pub is_submitting: bool,
}

impl TradeFormState {
    pub fn new(market_question: String, outcomes: Vec<OutcomeInfo>, selected_idx: usize) -> Self {
        let initial_price = outcomes.get(selected_idx).map(|o| o.price).unwrap_or(0.5);
        Self {
            market_question,
            outcomes,
            selected_outcome_idx: selected_idx,
            side: TradeSide::Buy,
            order_type: OrderType::Limit,
            limit_price: initial_price,
            shares: String::new(),
            amount: String::new(),
            active_field: TradeField::Shares, // Default to shares input for limit orders
            error_message: None,
            is_submitting: false,
        }
    }

    /// Get the currently selected outcome
    pub fn selected_outcome(&self) -> Option<&OutcomeInfo> {
        self.outcomes.get(self.selected_outcome_idx)
    }

    /// Get the current token ID
    #[allow(dead_code)]
    pub fn token_id(&self) -> Option<&str> {
        self.selected_outcome().map(|o| o.token_id.as_str())
    }

    /// Get the current outcome name
    pub fn outcome_name(&self) -> &str {
        self.selected_outcome()
            .map(|o| o.name.as_str())
            .unwrap_or("Unknown")
    }

    /// Get the best ask price for the selected outcome
    pub fn best_ask(&self) -> f64 {
        self.selected_outcome().map(|o| o.price).unwrap_or(0.5)
    }

    /// Select an outcome by index
    pub fn select_outcome(&mut self, idx: usize) {
        if idx < self.outcomes.len() && idx != self.selected_outcome_idx {
            self.selected_outcome_idx = idx;
            // Update limit price to the new outcome's price
            if let Some(outcome) = self.outcomes.get(idx) {
                self.limit_price = outcome.price;
            }
            self.error_message = None;
        }
    }

    pub fn add_char(&mut self, c: char) {
        let target = match self.active_field {
            TradeField::Shares => Some(&mut self.shares),
            TradeField::Amount => Some(&mut self.amount),
            _ => None,
        };

        if let Some(field) = target {
            // Only allow numeric input and decimal point
            if c.is_ascii_digit() || (c == '.' && !field.contains('.')) {
                field.push(c);
            }
        }
        self.error_message = None;
    }

    pub fn delete_char(&mut self) {
        let target = match self.active_field {
            TradeField::Shares => Some(&mut self.shares),
            TradeField::Amount => Some(&mut self.amount),
            _ => None,
        };

        if let Some(field) = target {
            field.pop();
        }
        self.error_message = None;
    }

    #[allow(dead_code)]
    pub fn toggle_side(&mut self) {
        self.side = self.side.toggle();
        self.error_message = None;
    }

    pub fn toggle_order_type(&mut self) {
        self.order_type = self.order_type.toggle();
        // Reset active field to appropriate default for the new order type
        self.active_field = match self.order_type {
            OrderType::Limit => TradeField::Shares,
            OrderType::Market => TradeField::Amount,
        };
        self.error_message = None;
    }

    /// Increment limit price by 0.1 cents (0.001)
    pub fn increment_limit_price(&mut self) {
        self.limit_price = (self.limit_price + 0.001).min(1.0);
        // Round to avoid floating point issues
        self.limit_price = (self.limit_price * 1000.0).round() / 1000.0;
        self.error_message = None;
    }

    /// Decrement limit price by 0.1 cents (0.001)
    pub fn decrement_limit_price(&mut self) {
        self.limit_price = (self.limit_price - 0.001).max(0.001);
        // Round to avoid floating point issues
        self.limit_price = (self.limit_price * 1000.0).round() / 1000.0;
        self.error_message = None;
    }

    pub fn shares_f64(&self) -> f64 {
        self.shares.parse().unwrap_or(0.0)
    }

    pub fn amount_f64(&self) -> f64 {
        self.amount.parse().unwrap_or(0.0)
    }

    /// Calculate total cost for limit orders (shares * limit_price)
    pub fn total_cost(&self) -> f64 {
        self.shares_f64() * self.limit_price
    }

    /// Calculate estimated shares for market orders (amount / best_ask)
    pub fn estimated_shares(&self) -> f64 {
        let amount = self.amount_f64();
        let best_ask = self.best_ask();
        if best_ask > 0.0 {
            amount / best_ask
        } else {
            0.0
        }
    }

    /// Calculate potential profit (for buy: payout - cost, for sell: proceeds)
    pub fn potential_profit(&self) -> f64 {
        match self.order_type {
            OrderType::Limit => {
                let shares = self.shares_f64();
                let cost = self.total_cost();
                match self.side {
                    TradeSide::Buy => shares - cost, // Shares pay $1 each if won
                    TradeSide::Sell => cost,         // Proceeds from selling
                }
            },
            OrderType::Market => {
                let shares = self.estimated_shares();
                match self.side {
                    TradeSide::Buy => shares - self.amount_f64(), // Shares pay $1 each if won
                    TradeSide::Sell => self.amount_f64(),         // Proceeds from selling
                }
            },
        }
    }

    pub fn next_field(&mut self) {
        self.active_field = self.active_field.next(self.order_type);
    }

    pub fn prev_field(&mut self) {
        self.active_field = self.active_field.prev(self.order_type);
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.shares.clear();
        self.amount.clear();
        self.limit_price = self.best_ask();
        self.side = TradeSide::Buy;
        self.order_type = OrderType::Limit;
        self.active_field = TradeField::Shares;
        self.error_message = None;
        self.is_submitting = false;
    }
}
