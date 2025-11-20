# UI Package Refactoring - Migration Guide

This document helps understand the changes made during the refactoring.

## What Changed?

The UI package was refactored from a single monolithic `main.rs` file into a modular architecture following iced best practices.

### Before

```
ui/src/
└── main.rs (293 lines)
    ├── Message enum
    ├── State struct
    ├── SensorChart struct
    ├── Chart implementation
    ├── Helper functions
    └── main() function
```

### After

```
ui/src/
├── main.rs (20 lines)               # Entry point only
├── message.rs (19 lines)            # Message enum + Page enum
├── app.rs (63 lines)                # App state (was State)
├── components/                      # Reusable components
│   ├── mod.rs (5 lines)
│   ├── chart.rs (142 lines)         # SensorChart (extracted)
│   └── overlay.rs (61 lines)        # New: Common overlay
└── pages/                           # Page modules
    ├── mod.rs (3 lines)
    └── home.rs (62 lines)           # Home page logic
```

## Functionality Mapping

### Original → New Location

| Original (main.rs) | New Location | Notes |
|-------------------|--------------|-------|
| `Message` enum | `message.rs` | Extended with NavigateTo and ToggleOverlay |
| `State` struct | `app.rs` as `App` | Renamed, added page routing |
| `State::new()` | `App::new()` | Same functionality |
| `State::update()` | `App::update()` | Added navigation handling |
| `State::view()` | `App::view()` | Added stack-based overlay rendering |
| `SensorChart` | `components/chart.rs` | Extracted as reusable component |
| `build_chart_2d()` | `components/chart.rs` | Moved with SensorChart |
| Chart view logic | `pages/home.rs` | Extracted to HomePage |

## New Features

### 1. Page Navigation System

```rust
// Define pages
pub enum Page {
    Home,
    // Add more pages here
}

// Navigate
Message::NavigateTo(Page::Home)
```

### 2. Common Overlay

```rust
// Toggle overlay from any page
Message::ToggleOverlay

// The overlay appears on top of all pages
```

### 3. Modular Components

Components can now be reused across pages:
```rust
use crate::components::SensorChart;
```

## How to Add a New Page

1. Create `ui/src/pages/your_page.rs`:
```rust
pub struct YourPage {
    // page state
}

impl YourPage {
    pub fn new() -> Self { /* ... */ }
    pub fn update(&mut self, message: &Message) { /* ... */ }
    pub fn view(&self) -> Element<'_, Message> { /* ... */ }
}
```

2. Export in `ui/src/pages/mod.rs`:
```rust
pub mod your_page;
pub use your_page::YourPage;
```

3. Add to `Page` enum in `message.rs`:
```rust
pub enum Page {
    Home,
    YourPage,  // Add here
}
```

4. Update `App` in `app.rs`:
```rust
pub struct App {
    your_page: YourPage,  // Add state
    // ...
}

// In new():
your_page: YourPage::new(),

// In update():
Page::YourPage => self.your_page.update(&message),

// In view():
Page::YourPage => self.your_page.view(),
```

## Benefits

- ✅ **Separation of Concerns**: Each file has a single responsibility
- ✅ **Reusability**: Components can be shared across pages
- ✅ **Maintainability**: Easier to find and modify code
- ✅ **Scalability**: Simple to add new pages and features
- ✅ **Testing**: Individual modules can be unit tested
- ✅ **Idiomatic**: Follows Rust and iced best practices

## No Breaking Changes

The application functionality remains identical:
- Same chart display
- Same update frequency
- Same visual appearance
- Added: Overlay demo button (new feature)

All existing functionality works exactly as before!
