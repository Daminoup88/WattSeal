# UI Architecture

This UI package follows an idiomatic iced architecture with clear separation of concerns.

## Structure

```
ui/src/
├── main.rs              # Application entry point and configuration
├── app.rs               # Main application state and routing logic
├── message.rs           # Global application messages
├── components/          # Reusable UI components
│   ├── mod.rs
│   └── chart.rs         # Chart component (SensorChart)
└── pages/               # Application pages/screens
    ├── mod.rs
    └── home.rs          # Home page with chart display
```

## Architecture Overview

### Message (`message.rs`)
- Defines all application-level messages
- Currently contains: `Tick` for periodic updates
- Can be extended with navigation, overlay, and page-specific messages

### App (`app.rs`)
- Main application state container
- Manages routing between pages
- Handles global state and overlays
- Coordinates updates across pages

### Components (`components/`)
- Reusable UI components that can be shared across pages
- `SensorChart`: A chart component for displaying time-series data

### Pages (`pages/`)
- Self-contained page modules
- Each page has its own state and view logic
- `HomePage`: Displays the sensor chart with periodic updates

## Supporting Multiple Pages with Overlays

The current architecture is designed to easily support:
1. **Multiple pages**: Add new page modules in `pages/` directory
2. **Page navigation**: Extend `Message` enum with page navigation messages
3. **Common overlays**: Add overlay state in `App` that renders over any page
4. **Shared state**: Manage shared data in `App` that pages can access

### Example: Adding a new page

1. Create `pages/settings.rs` with a `SettingsPage` struct
2. Add it to `pages/mod.rs`
3. Add navigation message in `message.rs`
4. Update `App` to manage the current page and handle navigation
5. Update `App::view()` to render the current page with optional overlay

## Benefits of this Architecture

- **Modularity**: Each component/page is self-contained
- **Maintainability**: Easy to find and modify specific functionality
- **Scalability**: Simple to add new pages and components
- **Testability**: Individual modules can be tested in isolation
- **Idiomatic**: Follows iced best practices and Rust module conventions
