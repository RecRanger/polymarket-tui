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
    pub fn toggle(&self) -> Self {
        match self {
            TradeSide::Buy => TradeSide::Sell,
            TradeSide::Sell => TradeSide::Buy,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TradeSide::Buy => "BUY",
            TradeSide::Sell => "SELL",
        }
    }
}

/// Trade form field being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeField {
    Side,
    Amount,
}

impl TradeField {
    pub fn next(&self) -> Self {
        match self {
            TradeField::Side => TradeField::Amount,
            TradeField::Amount => TradeField::Side,
        }
    }

    pub fn prev(&self) -> Self {
        self.next() // Only 2 fields
    }
}

/// Trade form state
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TradeFormState {
    pub token_id: String,
    pub market_question: String,
    pub outcome: String,
    pub side: TradeSide,
    pub amount: String, // Amount in dollars (input as string for editing)
    pub price: f64,     // Current market price
    pub active_field: TradeField,
    pub error_message: Option<String>,
    pub is_submitting: bool,
}

impl TradeFormState {
    pub fn new(token_id: String, market_question: String, outcome: String, price: f64) -> Self {
        Self {
            token_id,
            market_question,
            outcome,
            side: TradeSide::Buy,
            amount: String::new(),
            price,
            active_field: TradeField::Amount,
            error_message: None,
            is_submitting: false,
        }
    }

    pub fn add_char(&mut self, c: char) {
        if self.active_field == TradeField::Amount {
            // Only allow numeric input and decimal point
            if c.is_ascii_digit() || (c == '.' && !self.amount.contains('.')) {
                self.amount.push(c);
            }
        }
        self.error_message = None;
    }

    pub fn delete_char(&mut self) {
        if self.active_field == TradeField::Amount {
            self.amount.pop();
        }
        self.error_message = None;
    }

    pub fn toggle_side(&mut self) {
        self.side = self.side.toggle();
        self.error_message = None;
    }

    pub fn amount_f64(&self) -> f64 {
        self.amount.parse().unwrap_or(0.0)
    }

    /// Calculate estimated shares based on amount and price
    pub fn estimated_shares(&self) -> f64 {
        let amount = self.amount_f64();
        if self.price > 0.0 {
            amount / self.price
        } else {
            0.0
        }
    }

    /// Calculate potential profit (for buy: payout - cost, for sell: proceeds)
    pub fn potential_profit(&self) -> f64 {
        let shares = self.estimated_shares();
        match self.side {
            TradeSide::Buy => shares - self.amount_f64(), // Shares pay $1 each if won
            TradeSide::Sell => self.amount_f64(),         // Proceeds from selling
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.amount.clear();
        self.side = TradeSide::Buy;
        self.active_field = TradeField::Amount;
        self.error_message = None;
        self.is_submitting = false;
    }
}
