//! Render functions for the trending TUI

mod clicks;
mod event_details;
mod events_list;
mod favorites;
mod header;
mod logs;
mod main_render;
mod markets;
mod orderbook;
mod popups;
mod trades;
pub mod utils;
mod yield_tab;

pub use {
    clicks::{ClickedTab, get_clicked_tab, is_login_button_clicked},
    main_render::render,
    orderbook::check_orderbook_title_click,
    popups::TRADE_POPUP_WIDTH,
    utils::{centered_rect_fixed_width, truncate},
};
