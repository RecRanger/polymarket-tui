//! Search state types

use polymarket_api::gamma::Event;

/// Search mode enum to replace boolean flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    None,        // No search/filter active
    ApiSearch,   // API search mode (triggered by '/')
    LocalFilter, // Local filter mode (triggered by 'f')
}

/// Search-related state
#[derive(Debug)]
pub struct SearchState {
    pub mode: SearchMode,
    pub query: String,
    pub results: Vec<Event>,         // Results from API search
    pub is_searching: bool,          // Whether a search API call is in progress
    pub last_searched_query: String, // Last query that was searched
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            mode: SearchMode::None,
            query: String::new(),
            results: Vec::new(),
            is_searching: false,
            last_searched_query: String::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.mode != SearchMode::None
    }
}
