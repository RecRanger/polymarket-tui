//! Header rendering functions (tabs, search input, portfolio info)

use {
    super::render_search_input,
    crate::trending_tui::state::{EventFilter, MainTab, SearchMode, TrendingAppState},
    ratatui::{
        Frame,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Paragraph, Tabs},
    },
};

pub fn render_header(f: &mut Frame, app: &TrendingAppState, area: Rect) {
    // Calculate unified tab index: 0=Events, 1=Favorites, 2=Breaking, 3=Yield
    let tab_index = match app.main_tab {
        MainTab::Trending => match app.event_filter {
            EventFilter::Trending => 0,
            EventFilter::Breaking => 2,
        },
        MainTab::Favorites => 1,
        MainTab::Yield => 3,
    };

    if app.is_in_filter_mode() {
        // Split header into tabs, separator, and search input
        let header_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tabs line
                Constraint::Length(1), // Horizontal separator
                Constraint::Length(3), // Search input
            ])
            .split(area);

        // Render unified tabs
        let tab_titles: Vec<Line> = vec![
            Line::from("Events [1]"),
            Line::from("Favorites [2]"),
            Line::from("Breaking [3]"),
            Line::from("Yield [4]"),
        ];
        let tabs = Tabs::new(tab_titles)
            .select(tab_index)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )
            .divider(" ");
        f.render_widget(tabs, header_chunks[0]);

        // Horizontal separator line (gitui-style) - full width line of ─ characters
        let line_width = header_chunks[1].width as usize;
        let separator_line = "─".repeat(line_width);
        let separator = Paragraph::new(separator_line).style(Style::default().fg(Color::DarkGray));
        f.render_widget(separator, header_chunks[1]);

        // Search input field with proper styling
        let placeholder = match app.search.mode {
            SearchMode::ApiSearch => "Type to search via API...",
            SearchMode::LocalFilter => "Type to filter current list...",
            SearchMode::None => "Type to search...",
        };
        let title = if app.search.is_searching {
            "Search (loading...)"
        } else {
            "Search (Esc to close)"
        };
        render_search_input(
            f,
            header_chunks[2],
            &app.search.query,
            title,
            placeholder,
            app.search.is_searching,
            Color::Yellow,
        );
    } else {
        // Normal mode: Split header into tabs and horizontal line
        let header_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tabs line
                Constraint::Length(1), // Horizontal separator line
            ])
            .split(area);

        // Build right side: portfolio info + profile button
        let mut right_spans: Vec<Span> = Vec::new();

        // Add portfolio info if authenticated and available
        if app.auth_state.is_authenticated {
            // Total value (cash + portfolio)
            if app.auth_state.balance.is_some() || app.auth_state.portfolio_value.is_some() {
                let total = app.auth_state.balance.unwrap_or(0.0)
                    + app.auth_state.portfolio_value.unwrap_or(0.0);
                right_spans.push(Span::styled(
                    format!("${:.0}", total),
                    Style::default().fg(Color::Green),
                ));
                right_spans.push(Span::raw(" "));
            }

            // P&L
            if app.auth_state.unrealized_pnl.is_some() || app.auth_state.realized_pnl.is_some() {
                let total_pnl = app.auth_state.unrealized_pnl.unwrap_or(0.0)
                    + app.auth_state.realized_pnl.unwrap_or(0.0);
                let (pnl_str, pnl_color) = if total_pnl.abs() < 0.005 {
                    ("$0".to_string(), Color::DarkGray)
                } else if total_pnl > 0.0 {
                    (format!("+${:.0}", total_pnl), Color::Green)
                } else {
                    (format!("-${:.0}", total_pnl.abs()), Color::Red)
                };
                right_spans.push(Span::styled(pnl_str, Style::default().fg(pnl_color)));
                right_spans.push(Span::raw(" "));
            }

            // Profile button
            let name = app.auth_state.display_name();
            right_spans.push(Span::styled(
                format!("[ {} ]", name),
                Style::default().fg(Color::Green),
            ));
        } else {
            right_spans.push(Span::styled("[ Login ]", Style::default().fg(Color::Cyan)));
        }

        // API status indicator dot (using smaller bullet •)
        // Green = both APIs healthy, Yellow = one API down, Red = both down, Gray = unknown
        let status_dot = match (app.gamma_api_status, app.data_api_status) {
            (Some(true), Some(true)) => Span::styled(" •", Style::default().fg(Color::Green)),
            (Some(false), Some(false)) => Span::styled(" •", Style::default().fg(Color::Red)),
            (Some(true), Some(false)) | (Some(false), Some(true)) => {
                Span::styled(" •", Style::default().fg(Color::Yellow))
            },
            _ => Span::styled(" •", Style::default().fg(Color::DarkGray)),
        };
        right_spans.push(status_dot);

        let right_line = Line::from(right_spans);
        let right_width = right_line.width() as u16;

        // Split tabs line: tabs on left, portfolio + button on right
        let tabs_line_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),              // Tabs (fill remaining space)
                Constraint::Length(right_width), // Portfolio info + button
            ])
            .split(header_chunks[0]);

        // Render unified tabs in gitui-style (underline for selected, keyboard shortcuts)
        let tab_titles: Vec<Line> = vec![
            Line::from("Events [1]"),
            Line::from("Favorites [2]"),
            Line::from("Breaking [3]"),
            Line::from("Yield [4]"),
        ];
        let tabs = Tabs::new(tab_titles)
            .select(tab_index)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )
            .divider(" ");
        f.render_widget(tabs, tabs_line_chunks[0]);

        // Render portfolio info + login/user button on the right
        let right_paragraph = Paragraph::new(right_line).alignment(Alignment::Right);
        f.render_widget(right_paragraph, tabs_line_chunks[1]);

        // Horizontal separator line (gitui-style) - full width line of ─ characters
        let line_width = header_chunks[1].width as usize;
        let separator_line = "─".repeat(line_width);
        let separator = Paragraph::new(separator_line).style(Style::default().fg(Color::DarkGray));
        f.render_widget(separator, header_chunks[1]);
    }
}
