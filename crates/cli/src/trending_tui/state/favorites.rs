//! Favorites tab state

use {polymarket_api::gamma::Event, std::collections::HashSet};

/// Favorites tab state
#[derive(Debug)]
pub struct FavoritesState {
    pub events: Vec<Event>,
    pub favorite_ids: Vec<polymarket_api::FavoriteEvent>, // Favorite entries from API
    pub favorite_event_slugs: HashSet<String>,            // Quick lookup for favorite slugs
    pub selected_index: usize,
    pub scroll: usize,
    pub is_loading: bool,
    pub error_message: Option<String>,
}

#[allow(dead_code)]
impl FavoritesState {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            favorite_ids: Vec::new(),
            favorite_event_slugs: HashSet::new(),
            selected_index: 0,
            scroll: 0,
            is_loading: false,
            error_message: None,
        }
    }

    pub fn selected_event(&self) -> Option<&Event> {
        self.events.get(self.selected_index)
    }

    /// Check if an event slug is in favorites
    pub fn is_favorite(&self, slug: &str) -> bool {
        self.favorite_event_slugs.contains(slug)
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll {
                self.scroll = self.selected_index;
            }
        }
    }

    pub fn move_down(&mut self, visible_height: usize) {
        if self.selected_index + 1 < self.events.len() {
            self.selected_index += 1;
            if self.selected_index >= self.scroll + visible_height {
                self.scroll = self.selected_index - visible_height + 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.favorite_ids.clear();
        self.selected_index = 0;
        self.scroll = 0;
        self.error_message = None;
    }
}
