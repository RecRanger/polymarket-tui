//! Popup types for modal dialogs

/// Popup types for modal dialogs
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PopupType {
    Help,              // Show help/keyboard shortcuts
    ConfirmQuit,       // Confirm before quitting
    EventInfo(String), // Show detailed event info (slug)
    Login,             // Login modal with credential input
    UserProfile,       // Show authenticated user profile
    Trade,             // Trade modal (form state is in app.trade_form)
}
