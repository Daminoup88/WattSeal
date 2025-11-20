# UI Architecture

This UI package follows an idiomatic iced architecture with clear separation of concerns.

## Structure

```
ui/src/
├── main.rs              # Application entry point and configuration
├── app.rs               # Main application state and routing logic
├── message.rs           # Global application messages and page enum
├── components/          # Reusable UI components
│   ├── mod.rs
│   ├── chart.rs         # Chart component (SensorChart)
│   └── overlay.rs       # Common overlay component
└── pages/               # Application pages/screens
    ├── mod.rs
    └── home.rs          # Home page with chart display
```

## Architecture Overview

### Message (`message.rs`)
- Defines all application-level messages
- Contains: 
  - `Tick` for periodic chart updates
  - `NavigateTo(Page)` for page navigation
  - `ToggleOverlay` for showing/hiding the common overlay
- Includes `Page` enum to define available pages

### App (`app.rs`)
- Main application state container
- Manages current page state and routing
- Handles the common overlay that appears across all pages
- Uses `iced::widget::stack` to layer overlay on top of pages
- Coordinates updates across pages and components

### Components (`components/`)
- Reusable UI components that can be shared across pages
- `SensorChart`: A chart component for displaying time-series data
- `Overlay`: A common overlay component that can appear on top of any page
  - Demonstrates semi-transparent backdrop
  - Can be toggled via messages
  - Positioned using the stack widget

### Pages (`pages/`)
- Self-contained page modules
- Each page has its own state and view logic
- `HomePage`: Displays the sensor chart with periodic updates and an overlay demo button

## Multiple Pages with Common Overlay Support

✅ **Implemented Features:**

1. **Multiple Pages Support**: 
   - `Page` enum in `message.rs` defines available pages
   - `App` manages current page and routing via `NavigateTo` message
   - Each page module is independent and can be navigated to

2. **Common Overlay**:
   - `Overlay` component shared across all pages
   - Rendered on top of any page content using `stack![]` widget
   - Toggle functionality accessible from any page
   - Semi-transparent backdrop effect

3. **Layered Rendering**:
   - `App::view()` uses `stack![]` to layer overlay over page content
   - Overlay renders only when visible
   - All pages automatically support the overlay

### Adding a New Page

To add a new page (e.g., Settings):

1. **Create the page module** (`pages/settings.rs`):
```rust
pub struct SettingsPage {
    // page-specific state
}

impl SettingsPage {
    pub fn new() -> Self { /* ... */ }
    pub fn update(&mut self, message: &Message) { /* ... */ }
    pub fn view(&self) -> Element<'_, Message> { /* ... */ }
}
```

2. **Export in** `pages/mod.rs`:
```rust
pub mod settings;
pub use settings::SettingsPage;
```

3. **Add to Page enum** in `message.rs`:
```rust
pub enum Page {
    Home,
    Settings,  // Add this
}
```

4. **Update App** in `app.rs`:
```rust
pub struct App {
    settings_page: SettingsPage,  // Add state
    // ...
}

// In view():
Page::Settings => self.settings_page.view(),
```

### Adding Custom Overlay Content

The `Overlay` component can be extended to display dynamic content:
- Pass data to `view()` method
- Add message handlers for overlay interactions
- Create specialized overlay variants for different use cases

## Benefits of this Architecture

- **Modularity**: Each component/page is self-contained and reusable
- **Maintainability**: Easy to find and modify specific functionality
- **Scalability**: Simple to add new pages, overlays, and components
- **Testability**: Individual modules can be tested in isolation
- **Idiomatic**: Follows iced best practices and Rust module conventions
- **Layered UI**: Clean separation between page content and overlays
- **Type Safety**: Page navigation and message routing are type-checked
