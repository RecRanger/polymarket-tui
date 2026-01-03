# polymarket-tui

[![Crates.io](https://img.shields.io/crates/v/polymarket-tui.svg)](https://crates.io/crates/polymarket-tui)
[![Documentation](https://docs.rs/polymarket-tui/badge.svg)](https://docs.rs/polymarket-tui)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A terminal UI for browsing and monitoring Polymarket prediction markets in real-time.

## Features

### TUI Interface

- **Multiple tabs**: Events, Favorites, Breaking, and Yield views
- **Events**: All events sorted by 24h trading volume
- **Favorites**: Your bookmarked events (requires authentication)
- **Breaking**: Markets that moved the most in the last 24 hours (shows price change %)
- **Yield**: High-probability markets for yield opportunities

### Live Data

- **Real-time trades**: Watch live trades via WebSocket
- **Live prices**: Current prices for all market outcomes
- **Trade counts**: Number of trades per event

### Search & Navigation

- **API search** (`/`): Search all Polymarket events
- **Local filter** (`f`): Filter current list locally
- **Keyboard navigation**: Vim-style bindings (`j`/`k`)
- **Mouse support**: Click to select, scroll, switch tabs
- **Panel navigation**: Tab between Events, Details, Markets, Trades, Logs

### Market Information

- **Event details**: Title, slug, status, end date, tags
- **Market outcomes**: Prices for Yes/No or multiple outcomes
- **Volume indicators**: 24h and total volume
- **Status indicators**: Active, closed, in-review states

### Authentication (Optional)

- **Bookmark events**: Save favorites for quick access
- **Trade counts**: View trade activity (requires CLOB auth)
- **User profile**: View your Polymarket profile

## Installation

```bash
cargo install polymarket-tui
```

## Usage

```bash
# Start TUI (default)
polymarket-tui

# Or explicitly
polymarket-tui trending

# With options
polymarket-tui trending --order-by volume24hr --limit 100

# Other commands
polymarket-tui watch-event <event-slug>
polymarket-tui monitor --rtds --event <slug>
polymarket-tui orderbook <market-id>
polymarket-tui trades <market-id>
polymarket-tui event <event-slug>
polymarket-tui market <market-slug>
polymarket-tui yield --min-prob 0.95 --expires-in 7d

# View help
polymarket-tui --help
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑`/`k`, `↓`/`j` | Navigate up/down |
| `Tab` | Switch between panels |
| `←`/`→` | Switch tabs |
| `1`-`4` | Jump to tab (Events/Favorites/Breaking/Yield) |
| `Enter` | Toggle live trade watching |
| `/` | Search markets (API) |
| `f` | Filter current list (local) |
| `r` | Refresh current panel |
| `b` | Toggle bookmark |
| `o` | Open event URL in browser |
| `l` | Toggle logs panel |
| `p` | Show user profile |
| `?` | Show help |
| `Esc` | Cancel/close |
| `q` | Quit |

## Screenshot

![Screenshot](https://raw.githubusercontent.com/penso/polymarket-tui/main/screenshot.png)

The TUI displays:
- **Left panel**: Event list with volume/price-change and market count
- **Right panels**: Event details, markets with prices, and live trades

## Authentication

For favorites and trade counts, set environment variables:

```bash
export POLYMARKET_API_KEY="your-api-key"
export POLYMARKET_SECRET="your-secret"
export POLYMARKET_PASSPHRASE="your-passphrase"
```

## Related

- [polymarket-api](https://crates.io/crates/polymarket-api) - The underlying API library

## License

MIT
