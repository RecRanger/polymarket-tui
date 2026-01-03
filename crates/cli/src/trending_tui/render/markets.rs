//! Markets panel rendering functions

use {
    super::utils::{
        YIELD_MIN_PROB, format_price_cents, market_has_yield, truncate, truncate_to_width,
    },
    crate::trending_tui::state::{FocusedPanel, TrendingAppState},
    polymarket_api::gamma::Event,
    ratatui::{
        Frame,
        layout::{Alignment, Rect},
        style::{Color, Style},
        text::{Line, Span},
        widgets::{
            Block, BorderType, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
            ScrollbarState,
        },
    },
    unicode_width::UnicodeWidthStr,
};

pub fn render_markets(f: &mut Frame, app: &TrendingAppState, event: &Event, area: Rect) {
    if event.markets.is_empty() {
        let paragraph = Paragraph::new("No markets available")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Markets"),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
        return;
    }

    // Calculate visible height (accounting for borders: top and bottom)
    // The List widget with borders takes 2 lines (top border + title, bottom border)
    let visible_height = (area.height as usize).saturating_sub(2);
    let total_markets = event.markets.len();

    // Calculate maximum scroll position (can't scroll past the end)
    let max_scroll = total_markets.saturating_sub(visible_height.max(1));
    // Clamp scroll position to valid range
    let scroll = app.scroll.markets.min(max_scroll);

    // Sort markets: non-closed (active) first, then closed (resolved)
    let mut sorted_markets: Vec<_> = event.markets.iter().collect();
    sorted_markets.sort_by_key(|m| m.closed);

    // Fixed column widths for alignment - compact layout
    // Yield: "+XX.X%" = 6 chars max
    // Volume: "$XXX.XM" = 7 chars max
    // Buttons combined: "[XXXXXXXX XX.X¢][XXXXXXXX XX.X¢]" = 32 chars max (adjacent, no space)
    const YIELD_COL_WIDTH: usize = 6;
    const VOLUME_COL_WIDTH: usize = 7;
    const BUTTONS_COL_WIDTH: usize = 32; // Both buttons combined

    // Calculate total fixed right content width for active markets
    // Layout: [yield 6][space][volume 7][space][buttons 32] = 46
    let fixed_right_width = YIELD_COL_WIDTH + 1 + VOLUME_COL_WIDTH + 1 + BUTTONS_COL_WIDTH;
    let usable_width = (area.width as usize).saturating_sub(2); // -2 for borders
    let icon_width = 2; // "● " or "$ " etc.

    // Create list items for markets with scroll
    let items: Vec<ListItem> = sorted_markets
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(idx, market)| {
            // Use 24hr volume (more reliable) or fall back to total volume
            let volume = market.volume_24hr.or(market.volume_total);
            let volume_str = volume
                .map(|v| {
                    if v >= 1_000_000.0 {
                        format!("${:.1}M", v / 1_000_000.0)
                    } else if v >= 1_000.0 {
                        format!("${:.0}K", v / 1_000.0)
                    } else if v > 0.0 {
                        format!("${:.0}", v)
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default();

            // Status indicator: ● for active, ◐ for in-review, ○ for resolved
            // Add $ for yield opportunity (high probability market)
            let has_yield = market_has_yield(market);

            // Calculate yield return if there's a yield opportunity
            // Find the highest price outcome that qualifies as yield (>= 95%)
            let yield_return: Option<f64> = if has_yield {
                market
                    .outcome_prices
                    .iter()
                    .filter_map(|price_str| price_str.parse::<f64>().ok())
                    .filter(|&price| (YIELD_MIN_PROB..1.0).contains(&price))
                    .map(|price| (1.0 / price - 1.0) * 100.0) // Convert to percentage return
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)) // Best (lowest cost = highest price) yield
            } else {
                None
            };

            // Check if this market is selected for orderbook display
            let is_orderbook_selected = idx == app.orderbook_state.selected_market_index;

            // Status indicator: ● for active, ◐ for in-review, ○ for resolved, $ for yield
            let status_icon = if market.closed {
                "○ "
            } else if has_yield {
                "$ " // Yield opportunity indicator
            } else if market.is_in_review() {
                "◐ "
            } else {
                "● "
            };

            // Build outcome display string for closed markets
            let outcomes_str = if market.closed {
                // For resolved markets, show only the winning side
                let winner = market
                    .outcomes
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, outcome)| {
                        let price = market
                            .outcome_prices
                            .get(idx)
                            .and_then(|p| p.parse::<f64>().ok())?;
                        Some((outcome, price))
                    })
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                winner
                    .map(|(outcome, _)| format!("Winner: {}", outcome))
                    .unwrap_or_else(|| "Resolved".to_string())
            } else {
                String::new()
            };

            // Get prices for active markets (for Buy buttons)
            // Priority: 1) orderbook best ask (for selected market), 2) market_prices from batch API, 3) outcome_prices
            let (yes_price, no_price): (Option<f64>, Option<f64>) = if !market.closed {
                // Check if this is the selected market with orderbook data
                let orderbook_price = if is_orderbook_selected {
                    app.orderbook_state
                        .orderbook
                        .as_ref()
                        .and_then(|ob| ob.asks.first().map(|level| level.price))
                } else {
                    None
                };

                // For the selected market, use orderbook price based on which outcome is displayed
                let (yes_from_orderbook, no_from_orderbook) = if is_orderbook_selected {
                    match app.orderbook_state.selected_outcome {
                        crate::trending_tui::state::OrderbookOutcome::Yes => {
                            (orderbook_price, None)
                        },
                        crate::trending_tui::state::OrderbookOutcome::No => (None, orderbook_price),
                    }
                } else {
                    (None, None)
                };

                let yes = yes_from_orderbook.or_else(|| {
                    if let Some(ref token_ids) = market.clob_token_ids {
                        token_ids
                            .first()
                            .and_then(|asset_id| app.market_prices.get(asset_id).copied())
                            .or_else(|| {
                                market
                                    .outcome_prices
                                    .first()
                                    .and_then(|p| p.parse::<f64>().ok())
                            })
                    } else {
                        market
                            .outcome_prices
                            .first()
                            .and_then(|p| p.parse::<f64>().ok())
                    }
                });
                let no = no_from_orderbook.or_else(|| {
                    if let Some(ref token_ids) = market.clob_token_ids {
                        token_ids
                            .get(1)
                            .and_then(|asset_id| app.market_prices.get(asset_id).copied())
                            .or_else(|| {
                                market
                                    .outcome_prices
                                    .get(1)
                                    .and_then(|p| p.parse::<f64>().ok())
                            })
                    } else {
                        market
                            .outcome_prices
                            .get(1)
                            .and_then(|p| p.parse::<f64>().ok())
                    }
                });
                (yes, no)
            } else {
                (None, None)
            };

            // Build Buy buttons for active markets using actual outcome names
            let (yes_button, no_button) = if !market.closed {
                let yes_price_str = yes_price
                    .map(format_price_cents)
                    .unwrap_or_else(|| "N/A".to_string());
                let no_price_str = no_price
                    .map(format_price_cents)
                    .unwrap_or_else(|| "N/A".to_string());

                // Get outcome names, truncate to max 8 chars to keep buttons reasonable
                let outcome_0 = market
                    .outcomes
                    .first()
                    .map(|s| truncate(s, 8))
                    .unwrap_or_else(|| "Yes".to_string());
                let outcome_1 = market
                    .outcomes
                    .get(1)
                    .map(|s| truncate(s, 8))
                    .unwrap_or_else(|| "No".to_string());

                (
                    format!("[{} {}]", outcome_0, yes_price_str),
                    format!("[{} {}]", outcome_1, no_price_str),
                )
            } else {
                (String::new(), String::new())
            };

            // Format yield return string if applicable
            let yield_str = yield_return.map(|ret| format!("+{:.1}%", ret));

            let has_buttons = !market.closed;

            // Calculate available width for question
            let right_content_width = if has_buttons {
                fixed_right_width
            } else {
                // For closed markets: just outcomes + volume
                let outcomes_width = outcomes_str.width();
                let vol_width = volume_str.len();
                outcomes_width + 1 + vol_width
            };
            let available_width = usable_width
                .saturating_sub(right_content_width)
                .saturating_sub(icon_width)
                .saturating_sub(1); // 1 space padding

            // Truncate question to fit available width
            let display_name = market
                .group_item_title
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(&market.question);
            let question = truncate_to_width(display_name, available_width);
            let question_width = question.width();

            // Calculate remaining width for spacing
            let remaining_width = usable_width
                .saturating_sub(icon_width)
                .saturating_sub(question_width)
                .saturating_sub(right_content_width);

            // Start with status icon - use original colors
            let icon_color = if market.closed {
                Color::DarkGray
            } else if has_yield {
                Color::Green // Yield opportunity in green
            } else if market.is_in_review() {
                Color::Cyan
            } else {
                Color::Green
            };
            let mut line_spans = vec![
                Span::styled(status_icon, Style::default().fg(icon_color)),
                Span::styled(question, Style::default().fg(Color::White)),
            ];

            // Add spaces to push right content to the right
            if remaining_width > 0 {
                line_spans.push(Span::styled(" ".repeat(remaining_width), Style::default()));
            }

            if has_buttons {
                // For active markets: compact layout with buttons right-aligned to panel edge
                // Yield column (right-aligned within YIELD_COL_WIDTH)
                let yield_display = yield_str.as_deref().unwrap_or("");
                let yield_padded = format!("{:>width$}", yield_display, width = YIELD_COL_WIDTH);
                line_spans.push(Span::styled(
                    yield_padded,
                    Style::default().fg(Color::Yellow),
                ));
                line_spans.push(Span::styled(" ", Style::default()));

                // Volume column (right-aligned within VOLUME_COL_WIDTH)
                let volume_padded = format!("{:>width$}", volume_str, width = VOLUME_COL_WIDTH);
                line_spans.push(Span::styled(
                    volume_padded,
                    Style::default().fg(Color::Green),
                ));
                line_spans.push(Span::styled(" ", Style::default()));

                // Both buttons combined and right-aligned as a single unit (no space between)
                let buttons_combined = format!("{}{}", yes_button, no_button);
                let buttons_width = buttons_combined.len();
                // Add padding before buttons to right-align them
                let buttons_padding = BUTTONS_COL_WIDTH.saturating_sub(buttons_width);
                if buttons_padding > 0 {
                    line_spans.push(Span::raw(" ".repeat(buttons_padding)));
                }
                line_spans.push(Span::styled(yes_button, Style::default().fg(Color::Green)));
                line_spans.push(Span::styled(no_button, Style::default().fg(Color::Red)));
            } else {
                // For closed markets: show outcomes and volume
                if !outcomes_str.is_empty() {
                    line_spans.push(Span::styled(
                        outcomes_str.clone(),
                        Style::default().fg(Color::Cyan),
                    ));
                    if !volume_str.is_empty() {
                        line_spans.push(Span::styled(" ", Style::default()));
                    }
                }
                if !volume_str.is_empty() {
                    line_spans.push(Span::styled(
                        volume_str.clone(),
                        Style::default().fg(Color::Green),
                    ));
                }
            }

            // Background color: highlight selected market, otherwise zebra striping
            let bg_color = if is_orderbook_selected {
                Color::Rgb(60, 60, 80) // Highlight selected market (same as events list)
            } else if idx % 2 == 0 {
                Color::Reset
            } else {
                Color::Rgb(30, 30, 40)
            };

            ListItem::new(Line::from(line_spans)).style(Style::default().bg(bg_color))
        })
        .collect();

    let is_focused = app.navigation.focused_panel == FocusedPanel::Markets;
    let block_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    // Build title (without count, moved to bottom)
    let title = if is_focused {
        "Markets (Focused)"
    } else {
        "Markets"
    };

    // Build position indicator for bottom right (lazygit style)
    let selected_idx = app.orderbook_state.selected_market_index;
    let position_indicator = if total_markets > 0 {
        format!("{} of {}", selected_idx + 1, total_markets)
    } else {
        "0 of 0".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_bottom(Line::from(format!("{}─", position_indicator)).right_aligned())
        .border_style(block_style);

    let list = List::new(items).block(block);

    f.render_widget(list, area);

    // Render scrollbar if needed
    // The scrollbar thumb size is: (visible_height / total_markets) * track_height
    // This ensures proportional thumb that moves correctly when scrolling
    if total_markets > visible_height {
        // Calculate the scrollable range (max scroll position)
        let max_scroll = total_markets.saturating_sub(visible_height);
        // Ensure scroll position is within valid bounds
        let clamped_scroll = scroll.min(max_scroll);

        // ScrollbarState calculates thumb size as: (viewport_content_length / content_length) * track_height
        // content_length = total_markets (total number of items, set in new())
        // viewport_content_length = visible_height (how many items fit in viewport)
        // position = clamped_scroll (current scroll offset)
        let mut scrollbar_state = ScrollbarState::new(total_markets)
            .position(clamped_scroll)
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
