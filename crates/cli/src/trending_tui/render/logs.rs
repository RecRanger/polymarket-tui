//! Logs panel rendering

use {
    crate::trending_tui::state::{FocusedPanel, TrendingAppState},
    ratatui::{
        Frame,
        layout::Rect,
        style::{Color, Style},
        widgets::{
            Block, BorderType, Borders, List, ListItem, Scrollbar, ScrollbarOrientation,
            ScrollbarState,
        },
    },
};

pub fn render_logs(f: &mut Frame, app: &mut TrendingAppState, area: Rect) {
    // Calculate the actual visible height (accounting for borders)
    let visible_height = (area.height as usize).saturating_sub(2);

    // Auto-scroll to bottom only if Logs panel is NOT focused
    // When focused, user controls scrolling manually
    let is_focused = app.navigation.focused_panel == FocusedPanel::Logs;

    if !is_focused {
        // Auto-scroll to bottom if we're near the bottom or if logs have grown
        // This ensures new logs are always visible when panel is not focused
        if app.logs.messages.len() > visible_height {
            // Check if we're already showing the bottom (within 1 line)
            let current_bottom = app.logs.scroll + visible_height;
            if current_bottom >= app.logs.messages.len().saturating_sub(1) {
                // We're at or near the bottom, keep it there
                app.logs.scroll = app.logs.messages.len() - visible_height;
            }
        } else {
            // Not enough logs to scroll, show from the beginning
            app.logs.scroll = 0;
        }
    } else {
        // When focused, ensure scroll position is within valid bounds
        let max_scroll = app
            .logs
            .messages
            .len()
            .saturating_sub(visible_height.max(1));
        app.logs.scroll = app.logs.scroll.min(max_scroll);
    }

    // First, flatten logs by wrapping long lines
    let max_width = (area.width as usize).saturating_sub(2); // Account for borders
    let wrapped_logs: Vec<String> = app
        .logs
        .messages
        .iter()
        .skip(app.logs.scroll)
        .flat_map(|log| {
            // Split long lines by wrapping them to fit the available width
            if log.len() > max_width {
                // Split into multiple lines
                log.chars()
                    .collect::<Vec<_>>()
                    .chunks(max_width)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect::<Vec<_>>()
            } else {
                vec![log.clone()]
            }
        })
        .take(visible_height)
        .collect();

    let log_items: Vec<ListItem> = wrapped_logs
        .iter()
        .map(|log| {
            let color = if log.starts_with("[WARN]") {
                Color::Yellow
            } else if log.starts_with("[ERROR]") {
                Color::Red
            } else {
                Color::Gray
            };
            ListItem::new(log.as_str()).style(Style::default().fg(color))
        })
        .collect();
    let is_focused = app.navigation.focused_panel == FocusedPanel::Logs;
    let block_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let logs_list = List::new(log_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(if is_focused {
                    "Logs (Focused)"
                } else {
                    "Logs"
                })
                .border_style(block_style),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(logs_list, area);

    // Render scrollbar for logs if needed
    // Note: We scroll by message count, but display wrapped lines
    // The scrollbar represents message positions, and thumb size is proportional to visible messages
    let total_log_messages = app.logs.messages.len();
    if total_log_messages > 0 {
        // Estimate visible messages based on visible height and average wrapping
        // This is approximate but ensures the scrollbar thumb is reasonably proportional
        let estimated_visible_messages = visible_height.max(1);
        let mut scrollbar_state = ScrollbarState::new(total_log_messages)
            .position(app.logs.scroll)
            .viewport_content_length(estimated_visible_messages);
        f.render_stateful_widget(
            Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
            area,
            &mut scrollbar_state,
        );
    }
}
