//! Yield tab state types

use chrono::{DateTime, Utc};

/// A single yield opportunity (high probability market)
/// Full event details are looked up from the global event_cache using event_slug
/// Some event data is cached here for filtering and sorting purposes
#[derive(Debug, Clone)]
pub struct YieldOpportunity {
    pub market_name: String,
    pub market_status: &'static str,
    pub outcome: String,
    pub price: f64,
    pub est_return: f64,
    pub volume: f64,
    pub event_slug: String,
    // Cached event data for filtering/sorting (full details from event_cache)
    pub event_title: String,
    pub end_date: Option<DateTime<Utc>>,
}

/// A search result in the Yield tab - an event with its best yield opportunity (if any)
/// Event details are looked up from the global event_cache using event_slug
#[derive(Debug, Clone)]
pub struct YieldSearchResult {
    pub event_slug: String,
    /// Best yield opportunity for this event (highest return), if any
    pub best_yield: Option<YieldOpportunity>,
}

/// Yield tab state
#[derive(Debug)]
pub struct YieldState {
    pub opportunities: Vec<YieldOpportunity>,
    pub selected_index: usize,
    pub scroll: usize,
    pub is_loading: bool,
    pub min_prob: f64,
    pub max_prob: f64,
    pub min_volume: f64,
    pub sort_by: YieldSortBy,
    pub filter_query: String, // Current filter query
    pub is_filtering: bool,   // Whether filter input is active
    // API search state
    pub search_query: String,                   // Current search query
    pub search_results: Vec<YieldSearchResult>, // Search results with yield info
    pub is_searching: bool,                     // Whether search input is active
    pub is_search_loading: bool,                // Whether API search is in progress
    pub last_searched_query: String,            // Last query that was searched
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YieldSortBy {
    Return,  // Sort by estimated return (default)
    Volume,  // Sort by 24h volume
    EndDate, // Sort by end date (soonest first)
}

impl YieldSortBy {
    pub fn label(&self) -> &'static str {
        match self {
            YieldSortBy::Return => "Return",
            YieldSortBy::Volume => "Volume",
            YieldSortBy::EndDate => "End Date",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            YieldSortBy::Return => YieldSortBy::Volume,
            YieldSortBy::Volume => YieldSortBy::EndDate,
            YieldSortBy::EndDate => YieldSortBy::Return,
        }
    }
}

impl YieldState {
    pub fn new() -> Self {
        Self {
            opportunities: Vec::new(),
            selected_index: 0,
            scroll: 0,
            is_loading: false,
            min_prob: 0.95,
            max_prob: 1.0,
            min_volume: 0.0,
            sort_by: YieldSortBy::Return,
            filter_query: String::new(),
            is_filtering: false,
            search_query: String::new(),
            search_results: Vec::new(),
            is_searching: false,
            is_search_loading: false,
            last_searched_query: String::new(),
        }
    }

    pub fn sort_opportunities(&mut self) {
        match self.sort_by {
            YieldSortBy::Return => {
                self.opportunities
                    .sort_by(|a, b| b.est_return.partial_cmp(&a.est_return).unwrap());
            },
            YieldSortBy::Volume => {
                self.opportunities
                    .sort_by(|a, b| b.volume.partial_cmp(&a.volume).unwrap());
            },
            YieldSortBy::EndDate => {
                self.opportunities
                    .sort_by(|a, b| match (&a.end_date, &b.end_date) {
                        (Some(a_date), Some(b_date)) => a_date.cmp(b_date),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    });
            },
        }
    }

    pub fn move_up(&mut self) {
        let filtered_len = self.filtered_opportunities().len();
        if filtered_len == 0 {
            return;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll {
                self.scroll = self.selected_index;
            }
        }
    }

    pub fn move_down(&mut self, visible_height: usize) {
        let filtered_len = self.filtered_opportunities().len();
        if filtered_len == 0 {
            return;
        }
        if self.selected_index < filtered_len.saturating_sub(1) {
            self.selected_index += 1;
            if self.selected_index >= self.scroll + visible_height {
                self.scroll = self.selected_index - visible_height + 1;
            }
        }
    }

    pub fn selected_opportunity(&self) -> Option<&YieldOpportunity> {
        self.filtered_opportunities()
            .get(self.selected_index)
            .copied()
    }

    /// Get filtered opportunities based on the current filter query
    pub fn filtered_opportunities(&self) -> Vec<&YieldOpportunity> {
        if self.filter_query.is_empty() {
            return self.opportunities.iter().collect();
        }

        let query_lower = self.filter_query.to_lowercase();
        self.opportunities
            .iter()
            .filter(|opp| {
                opp.event_title.to_lowercase().contains(&query_lower)
                    || opp.event_slug.to_lowercase().contains(&query_lower)
                    || opp.market_name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn enter_filter_mode(&mut self) {
        self.is_filtering = true;
        self.filter_query.clear();
    }

    pub fn exit_filter_mode(&mut self) {
        self.is_filtering = false;
        // Keep the filter query so results stay filtered
    }

    #[allow(dead_code)]
    pub fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.selected_index = 0;
        self.scroll = 0;
    }

    pub fn add_filter_char(&mut self, c: char) {
        self.filter_query.push(c);
        self.selected_index = 0;
        self.scroll = 0;
    }

    pub fn delete_filter_char(&mut self) {
        self.filter_query.pop();
        self.selected_index = 0;
        self.scroll = 0;
    }

    // API search methods
    pub fn enter_search_mode(&mut self) {
        self.is_searching = true;
        self.search_query.clear();
        self.search_results.clear();
        self.last_searched_query.clear();
    }

    pub fn exit_search_mode(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        self.search_results.clear();
        self.last_searched_query.clear();
        self.selected_index = 0;
        self.scroll = 0;
    }

    /// Hide the search input but keep the search results displayed
    pub fn hide_search_input(&mut self) {
        self.is_searching = false;
        // Keep search_query, search_results, and last_searched_query intact
    }

    pub fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.selected_index = 0;
        self.scroll = 0;
    }

    pub fn delete_search_char(&mut self) {
        self.search_query.pop();
        self.selected_index = 0;
        self.scroll = 0;
    }

    pub fn set_search_results(&mut self, results: Vec<YieldSearchResult>, query: String) {
        self.search_results = results;
        self.last_searched_query = query;
        self.is_search_loading = false;
        self.selected_index = 0;
        self.scroll = 0;
    }

    /// Check if we're in any input mode (filter or search)
    #[allow(dead_code)]
    pub fn is_in_input_mode(&self) -> bool {
        self.is_filtering || self.is_searching
    }

    /// Get the currently displayed items count
    #[allow(dead_code)]
    pub fn displayed_count(&self) -> usize {
        if self.is_searching && !self.search_results.is_empty() {
            self.search_results.len()
        } else {
            self.filtered_opportunities().len()
        }
    }

    /// Get selected search result (when in search mode)
    pub fn selected_search_result(&self) -> Option<&YieldSearchResult> {
        if self.is_searching || !self.search_results.is_empty() {
            self.search_results.get(self.selected_index)
        } else {
            None
        }
    }
}
