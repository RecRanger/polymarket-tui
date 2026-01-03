//! Main render function for the trending TUI

use {
    super::{
        event_details::render_event_details,
        events_list::render_events_list,
        favorites::render_favorites_tab,
        header::render_header,
        logs::render_logs,
        markets::render_markets,
        orderbook::{calculate_orderbook_height, render_orderbook},
        popups::render_popup,
        trades::render_trades_table,
        yield_tab::render_yield_tab,
    },
    crate::trending_tui::state::{MainTab, SearchMode, TrendingAppState},
    ratatui::{
        Frame,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Style},
        widgets::{Block, BorderType, Borders, Paragraph},
    },
};

pub fn render(f: &mut Frame, app: &mut TrendingAppState) {
    // Header height: 2 lines for normal mode (tabs + separator), 5 for search mode
    let header_height = if app.is_in_filter_mode() {
        5
    } else {
        2
    };
    // No overlap - all panels have full borders with rounded corners
    // Conditionally include logs area based on show_logs
    let constraints: Vec<Constraint> = if app.show_logs {
        vec![
            Constraint::Length(header_height), // Header (with search if active)
            Constraint::Min(0),                // Main content
            Constraint::Length(8),             // Logs area
            Constraint::Length(3),             // Footer
        ]
    } else {
        vec![
            Constraint::Length(header_height), // Header (with search if active)
            Constraint::Min(0),                // Main content
            Constraint::Length(3),             // Footer
        ]
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.area());

    // Render header with main tabs
    render_header(f, app, chunks[0]);

    // Main content depends on active main tab
    match app.main_tab {
        MainTab::Trending => {
            // Main content - split into events list and trades view
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40), // Events list
                    Constraint::Fill(1),        // Right side takes remaining space
                ])
                .split(chunks[1]);

            render_events_list(f, app, main_chunks[0]);
            render_trades(f, app, main_chunks[1]);
        },
        MainTab::Favorites => {
            render_favorites_tab(f, app, chunks[1]);
        },
        MainTab::Yield => {
            render_yield_tab(f, app, chunks[1]);
        },
    }

    // Logs area (only if shown)
    // Footer index depends on whether logs are shown
    let footer_idx = if app.show_logs {
        render_logs(f, app, chunks[2]);
        3
    } else {
        2
    };

    // Footer - show focused panel info with context-sensitive help
    let panel_name = app.navigation.focused_panel.name();
    let panel_help = if app.main_tab == MainTab::Yield {
        "/: Search | f: Filter | s: Sort | r: Refresh | o: Open"
    } else {
        app.navigation.focused_panel.help_text()
    };
    let footer_text = if app.main_tab == MainTab::Yield && app.yield_state.is_searching {
        "Type to search | Esc: Cancel".to_string()
    } else if app.main_tab == MainTab::Yield && app.yield_state.is_filtering {
        "Type to filter | Esc: Cancel".to_string()
    } else if app.search.mode == SearchMode::ApiSearch {
        "Type to search | Esc: Cancel".to_string()
    } else if app.search.mode == SearchMode::LocalFilter {
        "Type to filter | Esc: Cancel".to_string()
    } else {
        format!(
            "{} | b: Bookmark | p: Profile | l: Logs | q: Quit | [{}]",
            panel_help, panel_name
        )
    };
    let footer = Paragraph::new(footer_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, chunks[footer_idx]);

    // Render popup if active (on top of everything)
    if let Some(ref popup) = app.popup {
        render_popup(f, app, popup);
    }
}

fn render_trades(f: &mut Frame, app: &TrendingAppState, area: Rect) {
    if let Some(event) = app.selected_event() {
        let event_slug = &event.slug;
        let trades = app.get_trades(event_slug);
        let is_watching = app.is_watching(event_slug);

        // Use a fixed minimum height for event details panel
        // Content will scroll if it exceeds this height
        let min_event_details_height = 8; // Minimum height (6 base lines + 2 for borders)

        // Calculate dynamic orderbook height based on data
        let orderbook_height = calculate_orderbook_height(app, Some(event));

        // Split area into event details, markets, orderbook, and trades
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(min_event_details_height as u16), // Event details (minimum height, scrollable)
                Constraint::Length(7), // Markets panel (5 lines + 2 for borders)
                Constraint::Length(orderbook_height), // Order Book panel (dynamic)
                Constraint::Min(0),    // Trades table
            ])
            .split(area);

        // Render event details
        render_event_details(f, app, event, is_watching, trades.len(), chunks[0]);

        // Render markets panel
        render_markets(f, app, event, chunks[1]);

        // Render order book panel
        render_orderbook(f, app, event, chunks[2]);

        // Render trades table
        render_trades_table(f, app, trades, Some(event), is_watching, chunks[3]);
    } else {
        let paragraph = Paragraph::new("No event selected")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_type(BorderType::Rounded)
                    .title("Event Details & Trades"),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
    }
}
