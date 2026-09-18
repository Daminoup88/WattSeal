# Overlay widget

The overlay is a small always-on-top window that mirrors the dashboard's live power metrics, so
you can keep an eye on consumption while the dashboard is closed or hidden behind a full-screen
game.

`overlay/` is a library crate that the main binary runs in **its own process** through
`WattSeal --overlay`. It owns its window, theme and configuration file, and never touches the
main application's database schema, so a slow or wedged overlay cannot take the dashboard down
with it.

## Opening and closing it

| Entry point      | Action                                  |
|------------------|-----------------------------------------|
| Dashboard footer | `Show overlay` / `Hide overlay` button  |
| Tray menu        | `Toggle Overlay`                        |
| Command line     | `WattSeal --overlay`                    |

The three entry points do not talk to each other over IPC. They agree through the
`overlay_requested` flag in the config file: whoever opens the overlay sets it, and the overlay
sets it back to `false` when it exits from its own menu. That is also how the tray can ask an
overlay to close when the overlay was launched by the dashboard rather than by the tray itself.

## Using the widget

- **Move it** — drag the widget with the left mouse button.
- **Menu** — right-click it. The metrics are replaced in place by four segments: `Resume`,
  `Settings`, `Pin` / `Unpin`, `Exit`. Swapping the bar's own content instead of raising an OS
  popup means the menu behaves identically on every platform and can never be clipped by the
  widget's own size.
- **Settings** — two columns: `Appearance` on the left (opacity, background and text color,
  transparency, layout, density, text size, theme, decimals, refresh interval, labels, units,
  short labels), `Window` and `Content` on the right. `Done` returns to the metrics and
  `Quit overlay` ends the process.

### Sizing

The height always follows the content that is currently enabled, so the widget never carries
blank space below the text. The width is measured from the text as well — most visibly in the
horizontal layout, which is a single line of metrics and would otherwise sit in a window much
wider than its contents.

In the vertical layout the `Width` slider (60–600 px) sets a floor rather than an exact size: the
widget never becomes narrower than its own text, which would clip the values.

## Language

The overlay follows the language the dashboard is set to. Changing it in the dashboard's settings
is enough: the overlay notices within a second and redraws itself, without a restart. It offers
the same five languages as the dashboard (English, German, French, Chinese, Romanian).

The overlay reads the setting from the same `ui_settings` row the dashboard writes, so there is no
second place to configure it. Until the dashboard has saved a language, English is used — which is
also what a standalone `WattSeal --overlay` run shows. The strings live in
`overlay/src/translations.rs`.

## Transparency

| Mode      | What it does                                                   |
|-----------|----------------------------------------------------------------|
| `Auto`    | Layered window on Windows, per-pixel surface alpha elsewhere   |
| `Layered` | Force the Win32 layered path                                   |
| `Off`     | Fully opaque                                                   |

On Windows the default is the **layered window** (`WS_EX_LAYERED` plus
`SetLayeredWindowAttributes`, see `winlayer.rs`), and that choice is not cosmetic. Many GPUs only
expose an `Opaque` composite mode for their swapchain; on such a machine a window created with
per-pixel alpha renders fully opaque no matter what the application draws, because the compositor
ignores the alpha channel it is handed. The layered path composites at the DWM level instead, so
it works with any GPU and any rendering backend.

The price of that path is that it applies **one alpha to the whole window** — the card and its
text fade together, and no per-element opacity can be expressed. That is why the overlay has a
single `Opacity` setting and no separate text opacity: on a machine whose surface offers no
alpha mode, the two cannot be decoupled, and offering the second slider would only suggest a
control that does nothing. Contrast is tuned with `Bg color` and `Text color`, which always
work because they change the *hue*, not the alpha.

> Set `WATTSEAL_OVERLAY_LOG=1` to have the overlay write its renderer diagnostics to
> `overlay.log` next to the executable: selected adapter, surface format and the alpha modes the
> surface actually accepted. It is opt-in, so a normal run never writes to disk, and it is the
> fastest way to find out which transparency path a given machine took.

## Pin mode

`Pin` does two things, and only the first one exists on every platform:

1. **The position is locked.** A pinned widget ignores dragging. This is plain application logic
   and works everywhere.
2. **The mouse passes through.** The window gets `WS_EX_TRANSPARENT | WS_EX_NOACTIVATE`, so every
   click — including the right-click that would reopen the menu — lands on whatever is underneath
   the widget, and the widget can never steal focus.

The second part is Windows-only and is controlled by `Pin makes it click-through` in the settings
panel (see `winlayer::click_through_supported`). macOS would need `setIgnoresMouseEvents` and X11
an input shape; Wayland has no protocol for it at all. Where click-through is unavailable the
settings panel says so rather than offering a toggle that cannot do anything.

Because a click-through widget no longer receives mouse events, **an overlay that is both pinned
and click-through can only be released from the tray menu**, or by editing the config file. This
is deliberate: the widget cannot be grabbed by accident while it is meant to be out of the way.

## Configuration

Everything is persisted to `overlay_config.json`, next to the executable. Missing or unknown keys
fall back to the defaults, so the file is safe to edit by hand.

| Key                  | Default | Meaning                                                              |
|----------------------|---------|----------------------------------------------------------------------|
| `opacity`            | `0.80`  | Window alpha (whole window on the layered path)                       |
| `bg_color`           | `auto`  | Card color swatch, `auto` follows `theme`                             |
| `text_color`         | `auto`  | Text color swatch, `auto` follows `theme`                             |
| `transparency`       | `auto`  | `auto` / `layered` / `off`                                            |
| `layout`             | `vertical` | `vertical` (one metric per line) or `horizontal` (single line)     |
| `density`            | `compact` | Padding and spacing: `ultra` / `compact` / `normal`                 |
| `font_size`          | `small` | `small` / `medium` / `large`                                          |
| `theme`              | `dark`  | `dark` / `light`                                                      |
| `show_labels`        | `true`  | Show the metric name next to its value                                |
| `show_units`         | `true`  | Show `W` after the value                                              |
| `abbreviated`        | `false` | Shorten labels (`Total` → `T`, `CPU` → `C`)                           |
| `decimals`           | `1`     | Decimals shown on the values                                           |
| `refresh_secs`       | `1`     | How often the metrics are re-read                                      |
| `always_on_top`      | `true`  | Keep the widget above other windows                                    |
| `width`              | `140.0` | Minimum width in the vertical layout, ignored by the horizontal one     |
| `overlay_requested`  | `true`  | Whether the overlay should be running                                  |
| `pin_mode`           | `false` | Position locked, and click-through if the next key allows it            |
| `pin_click_through`  | `true`  | Whether pinning also makes the window ignore the mouse                  |
| `blur`               | `false` | Ask the window system to blur what is behind the card (config file only) |
| `position`           | unset   | Last window position, restored on the next launch                      |
| `metrics`            | total, cpu, gpu, ram, top_apps | Which metrics are shown, in order          |
| `top_apps`           | `3`     | How many apps the `Top apps` metric lists (clamped to 1–8)             |

Deleting the file restores every default.
