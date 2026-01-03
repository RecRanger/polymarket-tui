//! Utility functions for rendering

use {
    chrono::{DateTime, Utc},
    polymarket_api::gamma::Event,
    ratatui::{
        Frame,
        layout::{Position, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Paragraph},
    },
    unicode_width::UnicodeWidthStr,
};

/// Format a number with thousands separators (e.g., 1234567 -> "1,234,567")
pub fn format_with_thousands(n: f64, decimals: usize) -> String {
    let formatted = format!("{:.prec$}", n, prec = decimals);
    let parts: Vec<&str> = formatted.split('.').collect();
    let int_part = parts[0];

    // Add thousands separators to integer part
    let chars: Vec<char> = int_part.chars().collect();
    let mut result = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }

    if decimals > 0 && parts.len() > 1 {
        format!("{}.{}", result, parts[1])
    } else {
        result
    }
}

/// Format a price (0.0-1.0) as cents like the Polymarket website
/// Uses 1 decimal place for sub-cent and high prices to match website rounding
/// Examples: 0.01 -> "1¢", 0.11 -> "11¢", 0.89 -> "89¢", 0.003 -> "0.3¢", 0.998 -> "99.8¢"
pub fn format_price_cents(price: f64) -> String {
    let cents = price * 100.0;
    if cents < 0.1 {
        // Very small prices, show with 2 decimal places
        format!("{:.2}¢", cents)
    } else if cents < 1.0 {
        // Sub-cent prices, show with 1 decimal place (e.g., 0.3¢)
        format!("{:.1}¢", cents)
    } else if cents < 10.0 {
        format!("{:.1}¢", cents)
    } else if cents > 99.0 && cents < 100.0 {
        // High prices (99-100%), show with 1 decimal place to match website
        format!("{:.1}¢", cents)
    } else {
        format!("{:.0}¢", cents)
    }
}

/// Format a volume/liquidity value with appropriate units (K, M)
pub fn format_volume(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("${:.0}K", value / 1_000.0)
    } else if value > 0.0 {
        format!("${:.0}", value)
    } else {
        String::new()
    }
}

/// Truncate a string to a maximum number of characters
pub fn truncate(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Format a profit/loss value with appropriate sign and color
/// Returns (formatted_string, color)
pub fn format_pnl(value: f64) -> (String, Color) {
    // Treat near-zero values as zero to avoid -$0.00
    if value.abs() < 0.005 {
        ("$0.00".to_string(), Color::DarkGray)
    } else if value > 0.0 {
        (format!("+${:.2}", value), Color::Green)
    } else {
        (format!("-${:.2}", value.abs()), Color::Red)
    }
}

/// Truncate a string to fit within a maximum display width (not byte length).
/// This properly handles Unicode characters that may have different display widths.
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    let current_width = s.width();
    if current_width <= max_width {
        return s.to_string();
    }

    // Need to truncate - account for "…" which is 1 column wide
    let target_width = max_width.saturating_sub(1);
    let mut result = String::new();
    let mut width = 0;

    for c in s.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + char_width > target_width {
            break;
        }
        result.push(c);
        width += char_width;
    }

    result.push('…');
    result
}

/// Yield opportunity threshold (95% probability = 5% potential return)
pub const YIELD_MIN_PROB: f64 = 0.95;

/// Check if a market has a yield opportunity (any outcome with price >= 95% and < 100%)
pub fn market_has_yield(market: &polymarket_api::gamma::Market) -> bool {
    // Skip closed/resolved markets - no yield opportunity
    if market.closed {
        return false;
    }

    market.outcome_prices.iter().any(|price_str| {
        price_str
            .parse::<f64>()
            .ok()
            .is_some_and(|price| (YIELD_MIN_PROB..1.0).contains(&price))
    })
}

/// Check if an event has any yield opportunities (any market with high probability outcome)
pub fn event_has_yield(event: &polymarket_api::gamma::Event) -> bool {
    event.markets.iter().any(market_has_yield)
}

/// Create a centered rectangle with percentage-based dimensions
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a centered rectangle with fixed width and percentage height
pub fn centered_rect_fixed_width(width: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Calculate horizontal margins
    let left_margin = (r.width.saturating_sub(width)) / 2;
    let right_margin = r.width.saturating_sub(width).saturating_sub(left_margin);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_margin),
            Constraint::Length(width),
            Constraint::Length(right_margin),
        ])
        .split(popup_layout[1])[1]
}

/// Render a search/filter input field with proper styling
/// Returns the cursor position if the field should show a cursor
pub fn render_search_input(
    f: &mut Frame,
    area: Rect,
    query: &str,
    title: &str,
    placeholder: &str,
    is_loading: bool,
    border_color: Color,
) {
    // Calculate inner area for the input text
    let inner_x = area.x + 1;
    let inner_y = area.y + 1;
    let inner_width = area.width.saturating_sub(2);

    // Render the block/border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::default().fg(border_color));
    f.render_widget(block, area);

    // Input field area with background
    let input_area = Rect {
        x: inner_x,
        y: inner_y,
        width: inner_width,
        height: 1,
    };

    // Determine display text
    let (display_text, text_style) = if query.is_empty() {
        // Show placeholder with dark background
        (
            placeholder.to_string(),
            Style::default().fg(Color::DarkGray),
        )
    } else if is_loading {
        // Show query with loading indicator
        (
            format!("{} (searching...)", query),
            Style::default().fg(Color::Cyan).bold(),
        )
    } else {
        // Show query
        (query.to_string(), Style::default().fg(Color::White).bold())
    };

    // Pad to fill the field width (creates visible input area with background)
    let padded_text = format!("{:<width$}", display_text, width = inner_width as usize);

    // Use background color to make input field visible
    let input_para = Paragraph::new(padded_text).style(text_style.bg(Color::Rgb(40, 40, 40)));
    f.render_widget(input_para, input_area);

    // Set cursor position at end of query text
    if !query.is_empty() || is_loading {
        // Don't show cursor when loading
        if !is_loading {
            let cursor_x = inner_x + query.len().min(inner_width as usize - 1) as u16;
            f.set_cursor_position(Position::new(cursor_x, inner_y));
        }
    } else {
        // Show cursor at start for empty field
        f.set_cursor_position(Position::new(inner_x, inner_y));
    }
}

/// Shared function to build event info lines for display
/// Used by both Events tab and Yield tab to show consistent event details
pub fn build_event_info_lines(
    event: &Event,
    is_watching: bool,
    trade_count_display: &str,
    trade_label: &str,
    area_width: u16,
) -> Vec<Line<'static>> {
    // Calculate total volume from all markets
    let total_volume: f64 = event
        .markets
        .iter()
        .map(|m| m.volume_24hr.or(m.volume_total).unwrap_or(0.0))
        .sum();

    // Format end date with relative time
    let end_date_str = event
        .end_date
        .as_ref()
        .and_then(|date_str| {
            DateTime::parse_from_rfc3339(date_str)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
        .map(|dt| {
            let now = Utc::now();
            let duration = dt.signed_duration_since(now);
            if duration.num_days() > 0 {
                format!("{} days", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!("{} hours", duration.num_hours())
            } else if duration.num_minutes() > 0 {
                format!("{} min", duration.num_minutes())
            } else if duration.num_seconds() < 0 {
                format!("Expired ({})", dt.format("%Y-%m-%d %H:%M UTC"))
            } else {
                format!("{}", dt.format("%Y-%m-%d %H:%M UTC"))
            }
        })
        .unwrap_or_else(|| "N/A".to_string());

    // Format volume
    let volume_str = if total_volume >= 1_000_000.0 {
        format!("${:.1}M", total_volume / 1_000_000.0)
    } else if total_volume >= 1_000.0 {
        format!("${:.1}K", total_volume / 1_000.0)
    } else {
        format!("${:.0}", total_volume)
    };

    let event_url = format!("https://polymarket.com/event/{}", event.slug);

    // Build lines
    let mut lines = vec![
        // Slug
        Line::from(vec![
            Span::styled("Slug: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(truncate(&event.slug, 60), Style::default().fg(Color::Blue)),
        ]),
        // URL
        Line::from(vec![
            Span::styled("URL: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(event_url, Style::default().fg(Color::Cyan)),
        ]),
        // Status: Active/Inactive | Open/Closed | Watching
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(
                if event.active {
                    "Active"
                } else {
                    "Inactive"
                },
                Style::default().fg(if event.active {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::styled(" | ", Style::default().fg(Color::Gray)),
            Span::styled(
                if event.closed {
                    "Closed"
                } else {
                    "Open"
                },
                Style::default().fg(if event.closed {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
            Span::styled(" | ", Style::default().fg(Color::Gray)),
            Span::styled(
                if is_watching {
                    "Watching"
                } else {
                    "Not Watching"
                },
                Style::default().fg(if is_watching {
                    Color::Red
                } else {
                    Color::Gray
                }),
            ),
        ]),
        // Estimated End
        Line::from(vec![
            Span::styled("Estimated End: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(end_date_str, Style::default().fg(Color::Magenta)),
        ]),
        // Total Volume | Trades
        Line::from(vec![
            Span::styled("Total Volume: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(
                volume_str,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" | ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}: ", trade_label),
                Style::default().fg(Color::Yellow).bold(),
            ),
            Span::styled(
                trade_count_display.to_string(),
                Style::default().fg(if trade_label == "Your Trades" {
                    Color::Green
                } else if trade_count_display == "..." {
                    Color::Yellow
                } else if is_watching {
                    Color::Cyan
                } else {
                    Color::Gray
                }),
            ),
        ]),
    ];

    // Add tags if available
    if !event.tags.is_empty() {
        let tag_labels: Vec<String> = event
            .tags
            .iter()
            .map(|tag| truncate(&tag.label, 20))
            .collect();
        let tags_text = tag_labels.join(", ");

        // Calculate available width for tags
        let available_width = (area_width as usize).saturating_sub(8);
        let tags_char_count = tags_text.chars().count();

        if tags_char_count <= available_width {
            lines.push(Line::from(vec![
                Span::styled("Tags: ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(tags_text, Style::default().fg(Color::Cyan)),
            ]));
        } else {
            // Truncate tags if too long
            lines.push(Line::from(vec![
                Span::styled("Tags: ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(
                    truncate(&tags_text, available_width),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }
    }

    lines
}
