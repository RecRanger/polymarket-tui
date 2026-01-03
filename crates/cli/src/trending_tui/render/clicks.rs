//! Click detection for the trending TUI

use {crate::trending_tui::state::TrendingAppState, ratatui::layout::Rect};

/// Unified tab enum for click detection (combines MainTab and EventFilter)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickedTab {
    Trending, // Displayed as "Events"
    Favorites,
    Breaking,
    Yield,
}

/// Check if the login button was clicked (top right)
/// Returns true if click is on the login button area
pub fn is_login_button_clicked(x: u16, y: u16, size: Rect, app: &TrendingAppState) -> bool {
    // Login button is on the first line (y = 0) and at the right edge
    if y != 0 {
        return false;
    }

    // Calculate button width dynamically based on auth state
    let button_width = if app.auth_state.is_authenticated {
        let name = app.auth_state.display_name();
        (name.len() + 4) as u16 // "[ " + name + " ]"
    } else {
        10 // "[ Login ]"
    };

    let login_button_start = size.width.saturating_sub(button_width);
    x >= login_button_start
}

/// Tabs are rendered on the first line (y = 0)
/// Returns which tab was clicked: Trending [1], Breaking [2], New [3], Yield [4]
pub fn get_clicked_tab(x: u16, y: u16, size: Rect, app: &TrendingAppState) -> Option<ClickedTab> {
    // Tabs are on the first line (y = 0)
    if y != 0 {
        return None;
    }

    // Don't match tabs if clicking on login button area (right side)
    // Calculate button width dynamically based on auth state
    let button_width = if app.auth_state.is_authenticated {
        let name = app.auth_state.display_name();
        (name.len() + 4) as u16 // "[ " + name + " ]"
    } else {
        10 // "[ Login ]"
    };
    let login_button_start = size.width.saturating_sub(button_width);
    if x >= login_button_start {
        return None;
    }

    // Actual rendered output (Tabs widget adds leading space and " " divider):
    // " Events [1] Favorites [2] Breaking [3] Yield [4]"
    // 0         1         2         3         4         5
    // 012345678901234567890123456789012345678901234567890
    //  Events [1] Favorites [2] Breaking [3] Yield [4]
    // Positions: 1-10 = Events, 12-25 = Favorites, 27-38 = Breaking, 40-49 = Yield
    if x <= 10 {
        return Some(ClickedTab::Trending);
    } else if (12..26).contains(&x) {
        return Some(ClickedTab::Favorites);
    } else if (27..39).contains(&x) {
        return Some(ClickedTab::Breaking);
    } else if (40..50).contains(&x) {
        return Some(ClickedTab::Yield);
    }
    None
}
