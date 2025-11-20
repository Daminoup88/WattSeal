# UI Package - Visual Architecture

## Component Flow Diagram

```
┌─────────────────────────────────────────────────────────┐
│                        main.rs                          │
│  ┌───────────────────────────────────────────────────┐  │
│  │ iced::application()                               │  │
│  │   .run_with(App::new)                            │  │
│  │   .subscription(|| Tick message every second)     │  │
│  └───────────────────────────────────────────────────┘  │
└───────────────────┬─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────┐
│                       App (app.rs)                       │
│  ┌───────────────────────────────────────────────────┐  │
│  │ State:                                            │  │
│  │  - current_page: Page                            │  │
│  │  - home_page: HomePage                           │  │
│  │  - overlay: Overlay                              │  │
│  └───────────────────────────────────────────────────┘  │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   update()   │  │    view()    │  │    new()     │  │
│  │              │  │              │  │              │  │
│  │ • Navigate   │  │ stack![      │  │ Initialize   │  │
│  │ • Toggle     │  │   page,      │  │ all state    │  │
│  │ • Forward to │  │   overlay    │  │              │  │
│  │   pages      │  │ ]            │  │              │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────┬────────────────────┬────────────────────┬─────────┘
      │                    │                    │
      │ Routes to:         │ Renders:           │ Uses:
      ▼                    ▼                    ▼
┌─────────────┐      ┌──────────┐        ┌──────────────┐
│   Pages     │      │   Stack  │        │  Messages    │
│  (pages/)   │      │ Layering │        │ (message.rs) │
└─────────────┘      └──────────┘        └──────────────┘
      │                    │
      │                    ├─── Layer 1: Page Content
      │                    └─── Layer 2: Overlay (if visible)
      ▼
┌─────────────────────────────────────────────────────────┐
│                      HomePage                            │
│                    (pages/home.rs)                       │
│  ┌───────────────────────────────────────────────────┐  │
│  │ State: chart: SensorChart                        │  │
│  └───────────────────────────────────────────────────┘  │
│                                                          │
│  ┌──────────────┐  ┌──────────────────────────────┐    │
│  │   update()   │  │         view()               │    │
│  │ • Handle Tick│  │ • Title                      │    │
│  │ • Update     │  │ • Chart (from component)     │    │
│  │   chart data │  │ • "Show Overlay" button      │    │
│  └──────────────┘  └──────────────────────────────┘    │
└───────────┬─────────────────────────────────────────────┘
            │ Uses:
            ▼
┌─────────────────────────────────────────────────────────┐
│                     Components                           │
│                   (components/)                          │
│  ┌────────────────────┐  ┌────────────────────────┐     │
│  │   SensorChart     │  │      Overlay           │     │
│  │  (chart.rs)       │  │   (overlay.rs)         │     │
│  │                   │  │                        │     │
│  │ • Time-series     │  │ • Semi-transparent     │     │
│  │   data storage    │  │   backdrop             │     │
│  │ • Chart rendering │  │ • Modal content        │     │
│  │ • Plotters impl   │  │ • Close button         │     │
│  └────────────────────┘  └────────────────────────┘     │
└─────────────────────────────────────────────────────────┘
```

## Message Flow

```
User Action
    │
    ▼
┌────────────────┐
│  Message Enum  │  (message.rs)
├────────────────┤
│ • Tick         │ ──► Update chart data
│ • NavigateTo   │ ──► Change current page
│ • ToggleOverlay│ ──► Show/hide overlay
└────────────────┘
    │
    ▼
┌────────────────┐
│  App::update() │  (app.rs)
├────────────────┤
│ Pattern match  │
│ on message     │
│ type           │
└────────────────┘
    │
    ├─► Page update (forward Tick, etc.)
    ├─► Navigation (set current_page)
    └─► Overlay toggle (toggle visibility)
```

## Adding a New Page - Visual Guide

```
Step 1: Create page module
┌──────────────────────────┐
│ pages/settings.rs        │
│ • SettingsPage struct    │
│ • new(), update(), view()│
└──────────────────────────┘

Step 2: Export module
┌──────────────────────────┐
│ pages/mod.rs             │
│ + pub mod settings;      │
│ + pub use settings::*;   │
└──────────────────────────┘

Step 3: Add to Page enum
┌──────────────────────────┐
│ message.rs               │
│ enum Page {              │
│   Home,                  │
│ + Settings,  ← Add       │
│ }                        │
└──────────────────────────┘

Step 4: Update App
┌──────────────────────────────────────┐
│ app.rs                               │
│ struct App {                         │
│ + settings_page: SettingsPage,       │
│   ...                                │
│ }                                    │
│                                      │
│ fn update(...) {                     │
│   Page::Settings =>                  │
│ +   self.settings_page.update(&msg) │
│ }                                    │
│                                      │
│ fn view(...) {                       │
│   Page::Settings =>                  │
│ +   self.settings_page.view()       │
│ }                                    │
└──────────────────────────────────────┘

Step 5: Navigate!
┌──────────────────────────┐
│ Anywhere in the app:     │
│                          │
│ button("Settings")       │
│   .on_press(             │
│     Message::NavigateTo( │
│       Page::Settings     │
│     )                    │
│   )                      │
└──────────────────────────┘
```

## Stack-based Overlay Rendering

```
┌─────────────────────────────────────┐
│          Browser Window             │
│                                     │
│  ┌───────────────────────────────┐  │
│  │ Layer 2: Overlay (if visible)│  │
│  │ ┌──────────────────────────┐  │  │
│  │ │  Semi-transparent        │  │  │
│  │ │  backdrop                │  │  │
│  │ │  ┌────────────────────┐  │  │  │
│  │ │  │  Overlay Content   │  │  │  │
│  │ │  │  • Title           │  │  │  │
│  │ │  │  • Message         │  │  │  │
│  │ │  │  • Close button    │  │  │  │
│  │ │  └────────────────────┘  │  │  │
│  │ └──────────────────────────┘  │  │
│  └───────────────────────────────┘  │
│           ▲                         │
│           │ Layered on top          │
│  ┌───────────────────────────────┐  │
│  │ Layer 1: Page Content        │  │
│  │ ┌──────────────────────────┐  │  │
│  │ │  Title                   │  │  │
│  │ │  Chart                   │  │  │
│  │ │  [Show Overlay] button   │  │  │
│  │ └──────────────────────────┘  │  │
│  └───────────────────────────────┘  │
│                                     │
└─────────────────────────────────────┘

Implementation:
stack![
    page_content,    // Always rendered
    overlay.view(),  // Rendered when visible
]
```

## File Dependency Graph

```
main.rs
  └─► app.rs
       ├─► message.rs (Message, Page)
       ├─► components/overlay.rs
       └─► pages/home.rs
            ├─► message.rs
            └─► components/chart.rs
                 └─► message.rs
```

All dependencies flow downward - no circular dependencies!
