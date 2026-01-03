//! Trades panel rendering functions

use {
    super::utils::truncate,
    crate::trending_tui::state::{FocusedPanel, Trade, TrendingAppState},
    chrono::DateTime,
    polymarket_api::gamma::Event,
    ratatui::{
        Frame,
        layout::{Alignment, Constraint, Rect},
        style::{Color, Modifier, Style},
        widgets::{
            Block, BorderType, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation,
            ScrollbarState, Table,
        },
    },
};

/// Render the trades table with event context (for finding market names)
pub fn render_trades_table(
    f: &mut Frame,
    app: &TrendingAppState,
    trades: &[Trade],
    event: Option<&Event>,
    is_watching: bool,
    area: Rect,
) {
    let is_focused = app.navigation.focused_panel == FocusedPanel::Trades;
    let block_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    if trades.is_empty() {
        let status_text = if is_watching {
            "Watching for trades... (Press Enter to stop)"
        } else {
            "Not watching. Press Enter to start watching this event."
        };
        let paragraph = Paragraph::new(status_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(if is_focused {
                        format!("Trades ({}) (Focused)", trades.len())
                    } else {
                        format!("Trades ({})", trades.len())
                    })
                    .border_style(block_style),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
        return;
    }

    // Calculate visible rows and apply scroll
    let visible_height = (area.height as usize).saturating_sub(3); // -3 for header
    let total_rows = trades.len();
    let scroll = app
        .scroll
        .trades
        .min(total_rows.saturating_sub(visible_height.max(1)));

    let rows: Vec<Row> = trades
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(idx, trade)| {
            let time = DateTime::from_timestamp(trade.timestamp, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "now".to_string());

            let side_style = if trade.side == "BUY" {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let outcome_style = if trade.outcome == "Yes" {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            // Find the market by asset_id and use short name if available
            let market_name = event
                .and_then(|e| {
                    e.markets
                        .iter()
                        .find(|m| {
                            m.clob_token_ids
                                .as_ref()
                                .is_some_and(|ids| ids.contains(&trade.asset_id))
                        })
                        .and_then(|m| {
                            m.group_item_title
                                .as_deref()
                                .filter(|s| !s.is_empty())
                                .or(Some(m.question.as_str()))
                        })
                })
                .unwrap_or(&trade.title);

            let title_truncated = truncate(market_name, 30);
            // Use user, fall back to pseudonym, or show "-" if both empty
            let user_display = if !trade.user.is_empty() {
                &trade.user
            } else if !trade.pseudonym.is_empty() {
                &trade.pseudonym
            } else {
                "-"
            };
            let user_truncated = truncate(user_display, 15);

            // Alternating row colors (zebra striping) for better readability
            let bg_color = if idx % 2 == 0 {
                Color::Reset
            } else {
                Color::Rgb(30, 30, 40)
            };

            Row::new(vec![
                Cell::from(time).style(Style::default().fg(Color::Gray)),
                Cell::from(trade.side.clone()).style(side_style),
                Cell::from(trade.outcome.clone()).style(outcome_style),
                Cell::from(format!("${:.4}", trade.price)),
                Cell::from(format!("{:.2}", trade.shares)),
                Cell::from(format!("${:.2}", trade.total_value)),
                Cell::from(title_truncated),
                Cell::from(user_truncated),
            ])
            .style(Style::default().bg(bg_color))
        })
        .collect();

    let table = Table::new(rows, [
        Constraint::Length(9),  // Time
        Constraint::Length(5),  // Side
        Constraint::Length(4),  // Outcome
        Constraint::Length(8),  // Price
        Constraint::Length(9),  // Shares
        Constraint::Length(9),  // Value
        Constraint::Fill(1),    // Market (takes remaining space)
        Constraint::Length(12), // User
    ])
    .header(
        Row::new(vec![
            "Time", "Side", "Out", "Price", "Shares", "Value", "Market", "User",
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if is_focused {
                format!("Trades ({}) (Focused)", trades.len())
            } else {
                format!("Trades ({})", trades.len())
            })
            .border_style(block_style),
    )
    .column_spacing(1)
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(60, 60, 80))
            .add_modifier(Modifier::BOLD),
    );

    // Use TableState for proper row selection (when Trades panel is focused)
    if is_focused && !trades.is_empty() {
        // Copy the state (TableState implements Copy in ratatui 0.30)
        let mut table_state = app.trades_table_state;
        // Set selection if not already set
        if table_state.selected().is_none() {
            table_state.select(Some(0));
        }
        f.render_stateful_widget(table, area, &mut table_state);
    } else {
        f.render_widget(table, area);
    }

    // Render scrollbar for trades if needed
    if total_rows > visible_height {
        let mut scrollbar_state = ScrollbarState::new(total_rows)
            .position(scroll)
            .viewport_content_length(visible_height);
        f.render_stateful_widget(
            Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
            area,
            &mut scrollbar_state,
        );
    }
}

/// Render the trades panel for a given set of trades and watching status (simpler version without event context)
pub fn render_trades_panel(
    f: &mut Frame,
    app: &TrendingAppState,
    trades: &[Trade],
    is_watching: bool,
    area: Rect,
) {
    let is_focused = app.navigation.focused_panel == FocusedPanel::Trades;
    let block_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    if trades.is_empty() {
        let status_text = if is_watching {
            "Watching for trades... (Press Enter to stop)"
        } else {
            "Not watching. Press Enter to start watching this event."
        };
        let paragraph = Paragraph::new(status_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(if is_focused {
                        format!("Trades ({}) (Focused)", trades.len())
                    } else {
                        format!("Trades ({})", trades.len())
                    })
                    .border_style(block_style),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
    } else {
        // Calculate visible rows and apply scroll
        let visible_height = (area.height as usize).saturating_sub(3);
        let total_rows = trades.len();
        let scroll = app
            .scroll
            .trades
            .min(total_rows.saturating_sub(visible_height.max(1)));

        let rows: Vec<Row> = trades
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_height)
            .map(|(idx, trade)| {
                let time = DateTime::from_timestamp(trade.timestamp, 0)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "now".to_string());

                let side_style = if trade.side == "BUY" {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };

                let outcome_style = if trade.outcome == "Yes" {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };

                let title_truncated = truncate(&trade.title, 30);
                let user_display = if !trade.user.is_empty() {
                    &trade.user
                } else if !trade.pseudonym.is_empty() {
                    &trade.pseudonym
                } else {
                    "-"
                };
                let user_truncated = truncate(user_display, 15);

                let bg_color = if idx % 2 == 0 {
                    Color::Reset
                } else {
                    Color::Rgb(30, 30, 40)
                };

                Row::new(vec![
                    Cell::from(time).style(Style::default().fg(Color::Gray)),
                    Cell::from(trade.side.clone()).style(side_style),
                    Cell::from(trade.outcome.clone()).style(outcome_style),
                    Cell::from(format!("${:.4}", trade.price)),
                    Cell::from(format!("{:.2}", trade.shares)),
                    Cell::from(format!("${:.2}", trade.total_value)),
                    Cell::from(title_truncated),
                    Cell::from(user_truncated),
                ])
                .style(Style::default().bg(bg_color))
            })
            .collect();

        let table = Table::new(rows, [
            Constraint::Length(9),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Fill(1),
            Constraint::Length(12),
        ])
        .header(
            Row::new(vec![
                "Time", "Side", "Out", "Price", "Shares", "Value", "Market", "User",
            ])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(if is_focused {
                    format!("Trades ({}) (Focused)", trades.len())
                } else {
                    format!("Trades ({})", trades.len())
                })
                .border_style(block_style),
        )
        .column_spacing(1);

        f.render_widget(table, area);

        // Render scrollbar if needed
        if total_rows > visible_height {
            let mut scrollbar_state = ScrollbarState::new(total_rows)
                .position(scroll)
                .viewport_content_length(visible_height);
            f.render_stateful_widget(
                Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
        }
    }
}
