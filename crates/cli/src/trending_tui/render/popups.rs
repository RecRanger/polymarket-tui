//! Popup/modal rendering functions

use {
    super::utils::{centered_rect, centered_rect_fixed_width, format_pnl, truncate},
    crate::trending_tui::state::{
        LoginField, MainTab, PopupType, TradeField, TradeSide, TrendingAppState,
    },
    ratatui::{
        Frame,
        layout::{Alignment, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    },
};

/// Build context-aware help content based on current tab
#[allow(clippy::vec_init_then_push)]
fn build_help_content(app: &TrendingAppState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Common icons section
    lines.push(Line::from(vec![Span::styled(
        "Icons:",
        Style::default().fg(Color::Yellow).bold(),
    )]));
    lines.push(Line::from(vec![
        Span::styled("  ⚑ ", Style::default().fg(Color::Magenta)),
        Span::raw("Favorited event (synced from Polymarket)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  $ ", Style::default().fg(Color::Green)),
        Span::raw("Yield opportunity (market with >95% probability)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ✕ ", Style::default().fg(Color::Red)),
        Span::raw("Closed/inactive event"),
    ]));
    lines.push(Line::from(""));

    // Tab-specific content
    match app.main_tab {
        MainTab::Trending => {
            lines.push(Line::from(vec![Span::styled(
                "Events Tab - Line Values:",
                Style::default().fg(Color::Yellow).bold(),
            )]));
            lines.push(Line::from(
                "  Each line shows: [icons] Title ... [metric] [markets]",
            ));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "  Metric changes with sort (press 's'):",
                Style::default().fg(Color::Cyan),
            )]));
            lines.push(Line::from(vec![
                Span::styled("  24h Vol:     ", Style::default().fg(Color::Green)),
                Span::raw("24-hour trading volume across all markets"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Total Vol:   ", Style::default().fg(Color::Green)),
                Span::raw("Total trading volume since event creation"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Liquidity:   ", Style::default().fg(Color::Cyan)),
                Span::raw("Available liquidity for trading"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Newest:      ", Style::default().fg(Color::Cyan)),
                Span::raw("Shows liquidity, sorted by creation date"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Ending Soon: ", Style::default().fg(Color::Cyan)),
                Span::raw("Shows liquidity, sorted by end date"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Competitive: ", Style::default().fg(Color::Magenta)),
                Span::raw("Score 0-100% (closer to 50/50 = more competitive)"),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Markets count ", Style::default().fg(Color::Cyan)),
                Span::raw("(rightmost) = number of markets in event"),
            ]));
        },
        MainTab::Favorites => {
            lines.push(Line::from(vec![Span::styled(
                "Favorites Tab - Line Values:",
                Style::default().fg(Color::Yellow).bold(),
            )]));
            lines.push(Line::from(
                "  Each line shows: ⚑ [yield] Title ... [volume] [markets]",
            ));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Volume ", Style::default().fg(Color::Green)),
                Span::raw("= Total trading volume (24h or total)"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Markets ", Style::default().fg(Color::Cyan)),
                Span::raw("= Number of markets in the event"),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(
                "  Favorites are synced from your Polymarket account.",
            ));
            lines.push(Line::from("  Login with 'L' to sync your favorites."));
        },
        MainTab::Yield => {
            lines.push(Line::from(vec![Span::styled(
                "Yield Tab - Line Values:",
                Style::default().fg(Color::Yellow).bold(),
            )]));
            lines.push(Line::from(
                "  Each line shows: Event/Market ... [return] [prob] [volume]",
            ));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Return ", Style::default().fg(Color::Green)),
                Span::raw("= Estimated annual return if market resolves Yes"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Probability ", Style::default().fg(Color::Cyan)),
                Span::raw("= Current Yes price (e.g., 99.5¢ = 99.5%)"),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Volume ", Style::default().fg(Color::Yellow)),
                Span::raw("= 24h trading volume for the market"),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(
                "  Yield opportunities are markets with >95% probability.",
            ));
            lines.push(Line::from(
                "  Higher return = higher risk (further from 100%).",
            ));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "  Sort options (press 's'):",
                Style::default().fg(Color::Cyan),
            )]));
            lines.push(Line::from("    Return, Volume, End Date"));
        },
    }

    lines.push(Line::from(""));

    // Keyboard shortcuts
    lines.push(Line::from(vec![Span::styled(
        "Keyboard Shortcuts:",
        Style::default().fg(Color::Yellow).bold(),
    )]));
    lines.push(Line::from("  ↑/k, ↓/j  Move up/down in lists"));
    lines.push(Line::from("  Tab       Switch between panels"));
    lines.push(Line::from(
        "  1-4       Switch tabs (Events/Favorites/Breaking/Yield)",
    ));
    lines.push(Line::from("  s         Cycle sort options"));
    lines.push(Line::from("  /         API search (searches Polymarket)"));
    lines.push(Line::from(
        "  f         Local filter (filters current list)",
    ));
    lines.push(Line::from("  o         Open event in browser"));
    lines.push(Line::from(
        "  Enter     Toggle watching event for live trades",
    ));
    lines.push(Line::from("  L         Login to Polymarket"));
    lines.push(Line::from("  l         Toggle logs panel"));
    lines.push(Line::from("  Esc       Cancel/close"));
    lines.push(Line::from("  q         Quit"));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Press Esc to close",
        Style::default().fg(Color::DarkGray),
    )]));

    lines
}

/// Render a popup/modal dialog
pub fn render_popup(f: &mut Frame, app: &TrendingAppState, popup: &PopupType) {
    // Note: We don't dim the full screen - content behind remains visible.
    // Only the popup area itself is cleared and has a solid background.

    match popup {
        PopupType::Login => {
            render_login_popup(f, app);
            return;
        },
        PopupType::UserProfile => {
            render_user_profile_popup(f, app);
            return;
        },
        PopupType::Trade => {
            render_trade_popup(f, app);
            return;
        },
        _ => {},
    }

    // Use larger area for Help popup since it has more content
    let area = match popup {
        PopupType::Help => centered_rect(70, 80, f.area()),
        _ => centered_rect(60, 50, f.area()),
    };

    // Clear the area behind the popup
    f.render_widget(Clear, area);

    let (title, content) = match popup {
        PopupType::Help => {
            let content = build_help_content(app);
            ("Help", content)
        },
        PopupType::ConfirmQuit => ("Confirm Quit", vec![
            Line::from(""),
            Line::from("Are you sure you want to quit?"),
            Line::from(""),
            Line::from(vec![
                Span::styled("  y  ", Style::default().fg(Color::Green).bold()),
                Span::styled("- Yes, quit", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("  n  ", Style::default().fg(Color::Red).bold()),
                Span::styled("- No, cancel", Style::default().fg(Color::White)),
            ]),
        ]),
        PopupType::EventInfo(slug) => ("Event Info", vec![
            Line::from(format!("Slug: {}", slug)),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press Esc to close",
                Style::default().fg(Color::DarkGray),
            )]),
        ]),
        // These are handled above with early return
        PopupType::Login | PopupType::UserProfile | PopupType::Trade => unreachable!(),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Helper to render an input field in the login form
fn render_login_input_field(
    f: &mut Frame,
    inner_x: u16,
    field_y: u16,
    label_width: u16,
    field_width: u16,
    label: &str,
    value: &str,
    is_active: bool,
    is_secret: bool,
) -> Option<ratatui::layout::Position> {
    use ratatui::layout::Position;

    // Render label
    let label_para =
        Paragraph::new(format!("{}:", label)).style(Style::default().fg(Color::Yellow).bold());
    let label_area = Rect {
        x: inner_x,
        y: field_y,
        width: label_width,
        height: 1,
    };
    f.render_widget(label_para, label_area);

    // Render input box with background
    let input_area = Rect {
        x: inner_x + label_width,
        y: field_y,
        width: field_width,
        height: 1,
    };

    // Display value (masked for secrets)
    let display_value = if is_secret && !value.is_empty() {
        "*".repeat(value.len().min(field_width as usize - 2))
    } else {
        value.to_string()
    };

    // Style: different background for input field, highlighted when active
    let (fg_color, bg_color) = if is_active {
        (Color::White, Color::DarkGray)
    } else {
        (Color::Gray, Color::Rgb(30, 30, 30))
    };

    // Pad the display value to fill the field width (creates visible input area)
    let padded_value = format!(
        "{:<width$}",
        display_value,
        width = field_width as usize - 1
    );

    let input_para = Paragraph::new(padded_value).style(Style::default().fg(fg_color).bg(bg_color));
    f.render_widget(input_para, input_area);

    // Return cursor position if this field is active
    if is_active {
        let cursor_x = input_area.x + display_value.len() as u16;
        Some(Position::new(cursor_x, input_area.y))
    } else {
        None
    }
}

/// Render the login popup with input fields
fn render_login_popup(f: &mut Frame, app: &TrendingAppState) {
    use ratatui::layout::Position;

    let area = centered_rect(80, 85, f.area());
    f.render_widget(Clear, area);

    let form = &app.login_form;

    // Calculate inner area (inside the popup border)
    let inner_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Field width for input boxes (leaving room for label)
    let label_width = 14u16;
    let field_width = inner_area.width.saturating_sub(label_width + 2);

    // Render the main popup block
    let block = Block::default()
        .title("Login - API Credentials")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));
    f.render_widget(block, area);

    // Header text
    let header = Paragraph::new("Enter your Polymarket API credentials:")
        .style(Style::default().fg(Color::White));
    let header_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 1,
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(header, header_area);

    // Track cursor position for the active field
    let mut cursor_position: Option<Position> = None;

    // Starting y position for fields
    let base_y = inner_area.y + 3;

    // Render required fields (each field takes 2 rows)
    if let Some(pos) = render_login_input_field(
        f,
        inner_area.x,
        base_y,
        label_width,
        field_width,
        "API Key",
        &form.api_key,
        form.active_field == LoginField::ApiKey,
        false,
    ) {
        cursor_position = Some(pos);
    }

    if let Some(pos) = render_login_input_field(
        f,
        inner_area.x,
        base_y + 2,
        label_width,
        field_width,
        "Secret",
        &form.secret,
        form.active_field == LoginField::Secret,
        true,
    ) {
        cursor_position = Some(pos);
    }

    if let Some(pos) = render_login_input_field(
        f,
        inner_area.x,
        base_y + 4,
        label_width,
        field_width,
        "Passphrase",
        &form.passphrase,
        form.active_field == LoginField::Passphrase,
        true,
    ) {
        cursor_position = Some(pos);
    }

    if let Some(pos) = render_login_input_field(
        f,
        inner_area.x,
        base_y + 6,
        label_width,
        field_width,
        "Address",
        &form.address,
        form.active_field == LoginField::Address,
        false,
    ) {
        cursor_position = Some(pos);
    }

    // Section header for optional cookies (after 4 fields = 8 rows + 1 gap)
    let cookie_section_y = base_y + 9;
    let cookie_header = Paragraph::new(Line::from(vec![
        Span::styled(
            "Optional: Browser Cookies ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "(for Favorites feature)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    let cookie_header_area = Rect {
        x: inner_area.x,
        y: cookie_section_y,
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(cookie_header, cookie_header_area);

    // Help text for cookies
    let cookie_help = Paragraph::new("Get from browser DevTools > Application > Cookies")
        .style(Style::default().fg(Color::DarkGray).italic());
    let cookie_help_area = Rect {
        x: inner_area.x,
        y: cookie_section_y + 1,
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(cookie_help, cookie_help_area);

    // Render optional cookie fields
    let cookie_fields_y = cookie_section_y + 3;

    if let Some(pos) = render_login_input_field(
        f,
        inner_area.x,
        cookie_fields_y,
        label_width,
        field_width,
        "Session",
        &form.session_cookie,
        form.active_field == LoginField::SessionCookie,
        false,
    ) {
        cursor_position = Some(pos);
    }

    if let Some(pos) = render_login_input_field(
        f,
        inner_area.x,
        cookie_fields_y + 2,
        label_width,
        field_width,
        "Nonce",
        &form.session_nonce,
        form.active_field == LoginField::SessionNonce,
        false,
    ) {
        cursor_position = Some(pos);
    }

    if let Some(pos) = render_login_input_field(
        f,
        inner_area.x,
        cookie_fields_y + 4,
        label_width,
        field_width,
        "Auth Type",
        &form.session_auth_type,
        form.active_field == LoginField::SessionAuthType,
        false,
    ) {
        cursor_position = Some(pos);
    }

    // Error message area
    let error_y = cookie_fields_y + 7;
    if let Some(ref error) = form.error_message {
        let error_para = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });
        let error_area = Rect {
            x: inner_area.x,
            y: error_y,
            width: inner_area.width,
            height: 2,
        };
        f.render_widget(error_para, error_area);
    }

    // Validation status
    if form.is_validating {
        let validating_para =
            Paragraph::new("Validating credentials...").style(Style::default().fg(Color::Yellow));
        let validating_area = Rect {
            x: inner_area.x,
            y: error_y,
            width: inner_area.width,
            height: 1,
        };
        f.render_widget(validating_para, validating_area);
    }

    // Instructions at bottom
    let instructions = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan).bold()),
        Span::styled(" Next  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Shift+Tab", Style::default().fg(Color::Cyan).bold()),
        Span::styled(" Prev  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Green).bold()),
        Span::styled(" Submit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Red).bold()),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    let instructions_para = Paragraph::new(instructions);
    let instructions_area = Rect {
        x: inner_area.x,
        y: area.y + area.height - 3,
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(instructions_para, instructions_area);

    // Set cursor position for the active field
    if let Some(pos) = cursor_position {
        f.set_cursor_position(pos);
    }
}

/// Render user profile popup
fn render_user_profile_popup(f: &mut Frame, app: &TrendingAppState) {
    // Fixed width: 55 (content) + 2 (borders) = 57
    let area = centered_rect_fixed_width(57, 60, f.area());
    f.render_widget(Clear, area);

    let auth = &app.auth_state;

    let mut content = vec![Line::from("")];

    // Profile section header
    content.push(Line::from(vec![Span::styled(
        "Profile",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )]));
    content.push(Line::from(""));

    // Show profile information if available
    if let Some(ref profile) = auth.profile {
        // Name
        if let Some(ref name) = profile.name {
            content.push(Line::from(vec![
                Span::styled("Name:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(name.clone(), Style::default().fg(Color::White).bold()),
            ]));
        }

        // Pseudonym (if different from name)
        if let Some(ref pseudonym) = profile.pseudonym {
            let show_pseudonym = profile
                .name
                .as_ref()
                .map(|n| n != pseudonym)
                .unwrap_or(true);
            if show_pseudonym {
                content.push(Line::from(vec![
                    Span::styled("Pseudonym: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(pseudonym.clone(), Style::default().fg(Color::Cyan)),
                ]));
            }
        }

        // Bio
        if let Some(ref bio) = profile.bio
            && !bio.is_empty()
        {
            content.push(Line::from(""));
            content.push(Line::from(vec![
                Span::styled("Bio:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(truncate(bio, 50), Style::default().fg(Color::White)),
            ]));
        }

        // Profile image URL (truncated)
        if let Some(ref img) = profile.profile_image
            && !img.is_empty()
        {
            content.push(Line::from(vec![
                Span::styled("Avatar:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(truncate(img, 45), Style::default().fg(Color::Blue)),
            ]));
        }
    } else if auth.username.is_some() {
        // Fallback: just show username if we have it but no full profile
        content.push(Line::from(vec![
            Span::styled("Username:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                auth.username.clone().unwrap_or_default(),
                Style::default().fg(Color::White).bold(),
            ),
        ]));
    } else {
        content.push(Line::from(vec![Span::styled(
            "(No profile information available)",
            Style::default().fg(Color::DarkGray),
        )]));
    }

    content.push(Line::from(""));
    content.push(Line::from(vec![Span::styled(
        "─".repeat(55),
        Style::default().fg(Color::DarkGray),
    )]));

    // Account section header
    content.push(Line::from(""));
    content.push(Line::from(vec![Span::styled(
        "Account",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )]));
    content.push(Line::from(""));

    // Status
    content.push(Line::from(vec![
        Span::styled("Status:    ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if auth.is_authenticated {
                "Authenticated"
            } else {
                "Not authenticated"
            },
            Style::default().fg(if auth.is_authenticated {
                Color::Green
            } else {
                Color::Red
            }),
        ),
    ]));

    // Address
    if let Some(ref addr) = auth.address {
        content.push(Line::from(vec![
            Span::styled("Address:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(addr.clone(), Style::default().fg(Color::Cyan)),
        ]));
    }

    // Balance (cash)
    if let Some(balance) = auth.balance {
        content.push(Line::from(vec![
            Span::styled("Cash:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.2} USDC", balance),
                Style::default().fg(Color::Green),
            ),
        ]));
    }

    // Portfolio value
    if let Some(portfolio_value) = auth.portfolio_value {
        // Use abs() to avoid displaying -$0.00
        let display_value = if portfolio_value.abs() < 0.005 {
            0.0
        } else {
            portfolio_value
        };
        content.push(Line::from(vec![
            Span::styled("Portfolio: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.2}", display_value),
                Style::default().fg(Color::Green),
            ),
        ]));
    }

    // Positions count
    if let Some(positions_count) = auth.positions_count {
        content.push(Line::from(vec![
            Span::styled("Positions: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", positions_count),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    }

    // Total value (cash + portfolio)
    if auth.balance.is_some() || auth.portfolio_value.is_some() {
        let total = auth.balance.unwrap_or(0.0) + auth.portfolio_value.unwrap_or(0.0);
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled("Total:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.2}", total),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    // Profit/Loss section
    if auth.unrealized_pnl.is_some() || auth.realized_pnl.is_some() {
        content.push(Line::from(""));
        content.push(Line::from(vec![Span::styled(
            "─".repeat(55),
            Style::default().fg(Color::DarkGray),
        )]));

        content.push(Line::from(""));
        content.push(Line::from(vec![Span::styled(
            "Profit / Loss",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));
        content.push(Line::from(""));

        // Unrealized P&L
        if let Some(unrealized) = auth.unrealized_pnl {
            let (pnl_str, pnl_color) = format_pnl(unrealized);
            content.push(Line::from(vec![
                Span::styled("Unrealized:", Style::default().fg(Color::DarkGray)),
                Span::styled(format!(" {}", pnl_str), Style::default().fg(pnl_color)),
            ]));
        }

        // Realized P&L
        if let Some(realized) = auth.realized_pnl {
            let (pnl_str, pnl_color) = format_pnl(realized);
            content.push(Line::from(vec![
                Span::styled("Realized:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!(" {}", pnl_str), Style::default().fg(pnl_color)),
            ]));
        }

        // Total P&L
        let total_pnl = auth.unrealized_pnl.unwrap_or(0.0) + auth.realized_pnl.unwrap_or(0.0);
        let (total_pnl_str, total_pnl_color) = format_pnl(total_pnl);
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled("Total P&L: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(" {}", total_pnl_str),
                Style::default()
                    .fg(total_pnl_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(vec![Span::styled(
        "─".repeat(55),
        Style::default().fg(Color::DarkGray),
    )]));
    content.push(Line::from(""));

    // Instructions
    content.push(Line::from(vec![
        Span::styled("Esc", Style::default().fg(Color::Cyan).bold()),
        Span::styled(" close    ", Style::default().fg(Color::DarkGray)),
        Span::styled("L", Style::default().fg(Color::Red).bold()),
        Span::styled(" logout", Style::default().fg(Color::DarkGray)),
    ]));

    // Build title with username if available
    let title = if let Some(ref name) = auth.username {
        format!(" {} ", name)
    } else {
        " User Profile ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Render trade popup with buy/sell form
fn render_trade_popup(f: &mut Frame, app: &TrendingAppState) {
    let area = centered_rect(65, 55, f.area());
    f.render_widget(Clear, area);

    let form = match &app.trade_form {
        Some(form) => form,
        None => {
            // Fallback if no form state
            let block = Block::default()
                .title("Trade")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red))
                .style(Style::default().bg(Color::Black));
            let paragraph = Paragraph::new("Error: No trade form state")
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(paragraph, area);
            return;
        },
    };

    // Build content lines
    let mut content = vec![
        Line::from(""),
        // Market question (truncated)
        Line::from(vec![Span::styled(
            truncate(&form.market_question, 55),
            Style::default().fg(Color::White).bold(),
        )]),
        Line::from(""),
        // Outcome
        Line::from(vec![
            Span::styled("Outcome: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&form.outcome, Style::default().fg(Color::Cyan).bold()),
        ]),
        // Current price
        Line::from(vec![
            Span::styled("Price:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0}¢", form.price * 100.0),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
    ];

    // Show balance if authenticated
    if app.auth_state.is_authenticated
        && let Some(balance) = app.auth_state.balance
    {
        content.push(Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.2}", balance),
                Style::default().fg(Color::Green),
            ),
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(vec![Span::styled(
        "─".repeat(50),
        Style::default().fg(Color::DarkGray),
    )]));
    content.push(Line::from(""));

    // Side selection (BUY / SELL)
    let side_active = form.active_field == TradeField::Side;
    let buy_style = if form.side == TradeSide::Buy {
        Style::default().fg(Color::Black).bg(Color::Green).bold()
    } else if side_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let sell_style = if form.side == TradeSide::Sell {
        Style::default().fg(Color::Black).bg(Color::Red).bold()
    } else if side_active {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    content.push(Line::from(vec![
        Span::styled("Side:    ", Style::default().fg(Color::DarkGray)),
        Span::styled(" BUY ", buy_style),
        Span::raw("  "),
        Span::styled(" SELL ", sell_style),
        if side_active {
            Span::styled("  ← Tab to toggle", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]));

    content.push(Line::from(""));

    // Amount input field
    let amount_active = form.active_field == TradeField::Amount;
    let amount_style = if amount_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let amount_display = if form.amount.is_empty() {
        "0.00".to_string()
    } else {
        form.amount.clone()
    };

    content.push(Line::from(vec![
        Span::styled("Amount:  ", Style::default().fg(Color::DarkGray)),
        Span::styled("$ ", amount_style),
        Span::styled(
            &amount_display,
            if amount_active {
                Style::default().fg(Color::White).bold()
            } else {
                Style::default().fg(Color::White)
            },
        ),
        if amount_active {
            Span::styled("_", Style::default().fg(Color::Cyan))
        } else {
            Span::raw("")
        },
    ]));

    content.push(Line::from(""));

    // Estimated shares and profit
    let shares = form.estimated_shares();
    let profit = form.potential_profit();

    content.push(Line::from(vec![
        Span::styled("Est. Shares: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:.2}", shares), Style::default().fg(Color::White)),
    ]));

    let profit_color = if profit >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };
    content.push(Line::from(vec![
        Span::styled(
            if form.side == TradeSide::Buy {
                "Potential Profit: "
            } else {
                "Proceeds: "
            },
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!(
                "{}${:.2}",
                if profit >= 0.0 {
                    "+"
                } else {
                    ""
                },
                profit.abs()
            ),
            Style::default().fg(profit_color),
        ),
        if form.side == TradeSide::Buy {
            Span::styled(" (if outcome wins)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]));

    content.push(Line::from(""));
    content.push(Line::from(vec![Span::styled(
        "─".repeat(50),
        Style::default().fg(Color::DarkGray),
    )]));

    // Error message if any
    if let Some(ref error) = form.error_message {
        content.push(Line::from(""));
        content.push(Line::from(vec![Span::styled(
            error,
            Style::default().fg(Color::Red),
        )]));
    }

    // Not authenticated warning
    if !app.auth_state.is_authenticated {
        content.push(Line::from(""));
        content.push(Line::from(vec![Span::styled(
            "⚠ Login required to trade",
            Style::default().fg(Color::Yellow),
        )]));
    }

    content.push(Line::from(""));

    // Instructions
    content.push(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan).bold()),
        Span::styled(" switch field  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Space", Style::default().fg(Color::Cyan).bold()),
        Span::styled(" toggle side  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Green).bold()),
        Span::styled(" submit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Red).bold()),
        Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
    ]));

    let title = format!(" {} {} ", form.side.label(), truncate(&form.outcome, 20));
    let border_color = if form.side == TradeSide::Buy {
        Color::Green
    } else {
        Color::Red
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
