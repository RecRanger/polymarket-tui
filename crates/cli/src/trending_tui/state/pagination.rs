//! Pagination and infinite scrolling state

/// Pagination and infinite scrolling state
#[derive(Debug)]
pub struct PaginationState {
    pub current_limit: usize,   // Current number of events fetched
    pub is_fetching_more: bool, // Whether we're currently fetching more events
    pub order_by: String,       // Order by parameter for API calls
    pub ascending: bool,        // Ascending parameter for API calls
}

impl PaginationState {
    pub fn new(order_by: String, ascending: bool, initial_limit: usize) -> Self {
        Self {
            current_limit: initial_limit,
            is_fetching_more: false,
            order_by,
            ascending,
        }
    }
}
