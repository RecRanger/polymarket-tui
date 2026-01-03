//! Favorites tab rendering functions

use {
    super::utils::{event_has_yield, format_volume, truncate_to_width},
    crate::trending_tui::state::{FocusedPanel, TrendingAppState},
    ratatui::{
        Frame,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{
            Block, BorderType, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
            ScrollbarState, Wrap,
        },
    },
    unicode_width::UnicodeWidthStr,
};

// Re-use functions from parent module
use super::{
    calculate_orderbook_height, render_event_details, render_markets, render_orderbook,
    render_trades_panel,
};
// Note: render_trades_panel is now imported from the trades module via mod.rs

/// Render the favorites tab
pub fn render_favorites_tab(f: &mut Frame, app: &TrendingAppState, area: Rect) {
    let favorites_state = &app.favorites_state;

    // Check authentication first
    if !app.auth_state.is_authenticated {
        let message = Paragraph::new("Please login to view your favorites.\n\nPress Tab to go to Login button, then Enter to open login dialog.")
            .block(
                Block::default()
                    .title(" Favorites ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(message, area);
        return;
    }

    // Show loading state - use same layout but with loading indicator
    if favorites_state.is_loading {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Events list
                Constraint::Fill(1),        // Right side
            ])
            .split(area);

        // Events panel with "Loading..." title
        let loading_list = Paragraph::new("Loading favorites...")
            .block(
                Block::default()
                    .title(" Events (Loading...) ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(loading_list, main_chunks[0]);

        // Empty right panel
        let empty_details = Paragraph::new("").block(
            Block::default()
                .title(" Event Details ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
        f.render_widget(empty_details, main_chunks[1]);
        return;
    }

    // Show error/info state
    if let Some(ref error) = favorites_state.error_message {
        // Check if this is a "missing session cookie" info message vs actual error
        let is_session_cookie_missing =
            error.contains("session_cookie") || error.contains("Session cookie");

        if is_session_cookie_missing {
            // Get the actual config path
            let config_path = crate::auth::AuthConfig::config_path();
            let config_path_str = config_path.display().to_string();

            // Show helpful setup instructions, not an error
            let lines = vec![
                Line::from(Span::styled(
                    "Session Cookie Required",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Favorites require browser authentication."),
                Line::from(""),
                Line::from(Span::styled(
                    "To set up:",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from("1. Log in to polymarket.com in your browser"),
                Line::from("2. Open Developer Tools (F12)"),
                Line::from("3. Go to Application > Cookies > polymarket.com"),
                Line::from("4. Copy these cookie values and add to config:"),
                Line::from(""),
                Line::from(Span::styled(
                    format!("   {}", config_path_str),
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "   \"session_cookie\": \"<polymarketsession>\",",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "   \"session_nonce\": \"<polymarketnonce>\",",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "   \"session_auth_type\": \"magic\"",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'e' to open config in system editor",
                    Style::default().fg(Color::Green),
                )),
            ];

            let info_msg = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(" Favorites - Setup Required ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            f.render_widget(info_msg, area);
        } else {
            // Show actual error
            let error_msg = Paragraph::new(format!("Error: {}", error))
                .block(
                    Block::default()
                        .title(" Favorites ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Red));
            f.render_widget(error_msg, area);
        }
        return;
    }

    // Show empty state
    if favorites_state.events.is_empty() {
        let empty = Paragraph::new(
            "No favorites yet.\n\nBrowse events in the Events tab and press 'b' to bookmark them.",
        )
        .block(
            Block::default()
                .title(" Favorites ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, area);
        return;
    }

    // Use the same layout as Trending tab - events list + right side with details
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Events list
            Constraint::Fill(1),        // Right side takes remaining space
        ])
        .split(area);

    render_favorites_list(f, app, main_chunks[0]);

    // Right side: event details, markets, orderbook, trades (if event selected)
    if let Some(event) = favorites_state.selected_event() {
        let event_slug = &event.slug;
        let trades = app.get_trades(event_slug);
        let is_watching = app.is_watching(event_slug);

        // Calculate dynamic orderbook height based on data
        let orderbook_height = calculate_orderbook_height(app, Some(event));

        // Split right side into event details, markets, orderbook, and trades
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),                // Event details
                Constraint::Length(7),                // Markets panel
                Constraint::Length(orderbook_height), // Order Book panel (dynamic)
                Constraint::Min(0),                   // Trades table
            ])
            .split(main_chunks[1]);

        // Render event details
        render_event_details(f, app, event, is_watching, trades.len(), right_chunks[0]);

        // Render markets panel
        render_markets(f, app, event, right_chunks[1]);

        // Render order book panel
        render_orderbook(f, app, event, right_chunks[2]);

        // Render trades
        render_trades_panel(f, app, trades, is_watching, right_chunks[3]);
    } else {
        // No event selected - show empty panel
        let empty = Paragraph::new("Select a favorite event to view details")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Event Details"),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, main_chunks[1]);
    }
}

/// Render the favorites events list (separate from main events list)
fn render_favorites_list(f: &mut Frame, app: &TrendingAppState, area: Rect) {
    let favorites_state = &app.favorites_state;
    let events = &favorites_state.events;

    let scroll = favorites_state.scroll;
    let selected_index = favorites_state.selected_index;
    let visible_events: Vec<_> = events
        .iter()
        .enumerate()
        .skip(scroll)
        .take(area.height as usize - 2)
        .collect();

    // First pass: calculate max width of market count for alignment
    let max_markets_width = visible_events
        .iter()
        .map(|(_, event)| event.markets.len().to_string().len())
        .max()
        .unwrap_or(1);

    let items: Vec<ListItem> = visible_events
        .into_iter()
        .map(|(idx, event)| {
            let is_selected = idx == selected_index;

            // Check if event is closed/inactive
            let is_closed = event.closed || !event.active;

            // Calculate total volume from markets
            let total_volume: f64 = event
                .markets
                .iter()
                .map(|m| m.volume_24hr.or(m.volume_total).unwrap_or(0.0))
                .sum();

            // Format volume
            let volume_str = format_volume(total_volume);

            // Format market count with padding
            let markets_str = format!("{:>width$}", event.markets.len(), width = max_markets_width);

            // Calculate widths for proper alignment
            let usable_width = area.width.saturating_sub(2) as usize; // -2 for borders

            // Icons: favorite (always shown) + yield (if applicable)
            let favorite_icon = "⚑ ";
            let favorite_icon_width = favorite_icon.width();
            let has_yield = !is_closed && event_has_yield(event);
            let yield_icon = if has_yield {
                "$ "
            } else {
                ""
            };
            let yield_icon_width = yield_icon.width();

            // Build right-aligned text: "volume markets"
            let right_text = if volume_str.is_empty() {
                markets_str.clone()
            } else {
                format!("{} {}", volume_str, markets_str)
            };
            let right_text_width = right_text.width();

            // Calculate available width for title
            let reserved_width = favorite_icon_width + yield_icon_width + right_text_width + 1;
            let available_width = usable_width.saturating_sub(reserved_width);

            // Truncate title to fit
            let title = truncate_to_width(&event.title, available_width);
            let title_width = title.width();

            // Calculate spacing to right-align
            let remaining_width = usable_width
                .saturating_sub(favorite_icon_width)
                .saturating_sub(yield_icon_width)
                .saturating_sub(title_width)
                .saturating_sub(right_text_width);

            // Title style
            let title_style = if is_closed {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Build spans with proper alignment
            let mut spans = Vec::new();

            // Favorite icon (always shown for favorites)
            spans.push(Span::styled(
                favorite_icon,
                Style::default().fg(Color::Magenta),
            ));

            // Yield icon if applicable
            if has_yield {
                spans.push(Span::styled(yield_icon, Style::default().fg(Color::Green)));
            }

            // Title
            spans.push(Span::styled(title, title_style));

            // Spacing to right-align
            if remaining_width > 0 {
                spans.push(Span::styled(" ".repeat(remaining_width), Style::default()));
            }

            // Volume (right-aligned)
            if !volume_str.is_empty() {
                spans.push(Span::styled(
                    volume_str.clone(),
                    Style::default().fg(if is_closed {
                        Color::DarkGray
                    } else {
                        Color::Green
                    }),
                ));
                spans.push(Span::styled(" ", Style::default()));
            }

            // Market count (right-aligned)
            spans.push(Span::styled(
                markets_str,
                Style::default().fg(if is_closed {
                    Color::DarkGray
                } else {
                    Color::Cyan
                }),
            ));

            let line = Line::from(spans);
            let mut item = ListItem::new(line);

            if is_selected {
                item = item.style(
                    Style::default()
                        .bg(Color::Rgb(60, 60, 80))
                        .add_modifier(Modifier::BOLD),
                );
            }

            item
        })
        .collect();

    let is_focused = app.navigation.focused_panel == FocusedPanel::EventsList;
    let block_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    // Build position indicator for bottom right (lazygit style)
    let total_count = events.len();
    let position_indicator = if total_count > 0 {
        format!("{} of {}", selected_index + 1, total_count)
    } else {
        "0 of 0".to_string()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Favorites")
                .title_bottom(Line::from(format!("{}─", position_indicator)).right_aligned())
                .border_style(block_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(60, 60, 80))
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(list, area);

    // Render scrollbar if needed
    let total_items = events.len();
    let visible_height = area.height.saturating_sub(2) as usize;
    if total_items > visible_height {
        let mut scrollbar_state = ScrollbarState::new(total_items)
            .position(scroll)
            .viewport_content_length(visible_height);
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut scrollbar_state,
        );
    }
}
