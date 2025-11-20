/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    /// Periodic tick for chart updates
    Tick,
    /// Navigate to a different page
    NavigateTo(Page),
    /// Toggle overlay visibility
    ToggleOverlay,
}

/// Available pages in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    // Future pages can be added here:
    // Settings,
    // About,
}
