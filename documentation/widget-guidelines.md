# DockBar — Widget Guidelines

Reference guide for creating widgets. A widget is a small web app — HTML, CSS, and JavaScript — that runs inside a
DockBar window. It is essentially a Single Page Application (SPA): you can use any framework you like (React, Svelte,
Vue, vanilla JS, anything) or none at all.

DockBar's own built-in widgets are mostly written in Svelte; the Settings panel is written in React.

Read [resource guidelines](./resource-guidelines) first for concepts shared across all resource kinds.

---

## Table of Contents

1. [How a Widget is Loaded](#1-how-a-widget-is-loaded)
2. [The HTML Entry Point](#2-the-html-entry-point)
3. [Widget Structure — metadata.yml](#3-widget-structure--metadatayml)
4. [Window Behavior — Preset and Instances](#4-window-behavior--preset-and-instances)
5. [User-Configurable Settings](#5-user-configurable-settings)
   - [Common Fields](#common-fields)
   - [Switch](#switch)
   - [Select](#select)
   - [InputText](#inputtext)
   - [InputNumber](#inputnumber)
   - [Range](#range)
   - [Color](#color)
   - [Font](#font)
   - [Grouping Settings](#grouping-settings)
6. [CSS — Use Global Classes, Not CSS Modules](#6-css--use-global-classes-not-css-modules)
7. [Inspecting Your Widget with DevTools](#7-inspecting-your-widget-with-devtools)
8. [Folder Structure](#8-folder-structure)

---

## 1. How a Widget is Loaded

After you build and bundle your project you will have three output files: an HTML shell, a bundled JS file, and a
bundled CSS file. These are referenced from `metadata.yml` using `!include` and loaded by DockBar into an isolated
webview window.

This guide covers how your widget is declared and packaged. For the runtime JS API your widget code calls once it's
actually running — `Widget.self.init()`/`.ready()`, `invoke`, `subscribe` — see [Widget JS API](./widget-js-api).

---

## 2. The HTML Entry Point

The HTML file is **injected directly into the `<body>`** of the widget's webview. You do not need to write `<html>`,
`<head>`, or `<body>` tags — just the markup that would normally live inside the body. For a framework-based SPA this is
typically just a single mount point:

```html
<!-- index.html -->
<div id="root"></div>
```

Or for Svelte:

```html
<div id="app"></div>
```

Your framework bootstraps from that element as usual. No `<link>` or `<script>` tags are needed — the CSS and JS are
declared in `metadata.yml` and injected by the widget loader.

---

## 3. Widget Structure — metadata.yml

```yaml
id: "@yourname/my-widget"

metadata:
  displayName: My Widget
  description: A short description.
  tags:
    - clock
    - minimal

# Icon shown in the DockBar settings panel.
# Must be a valid react-icons name (https://react-icons.github.io/react-icons/).
icon: PiClockFill

# Window behavior preset (see section 4)
preset: Overlay

# How many instances users can create (see section 4)
instances: Single

# If true, the widget window is not created until it is explicitly triggered.
lazy: false

# Widget source files (required)
html: !include index.html
js: !include index.js
css: !include index.css

# User-configurable settings (see section 5)
settings: []
```

All fields except `id`, `html`, `js`, and `css` are optional.

---

## 4. Window Behavior — Preset and Instances

### Preset

A preset is a shortcut to a standard window configuration. When your widget calls `init()` from the SLU library, it
reads the `preset` value and automatically applies the expected window behavior (always on top, no title bar, auto-hide
on focus loss, etc.).

Using a preset is **not mandatory**. If none of the built-in presets fit your needs you can set `preset: None` and
configure the window yourself through the SLU library API.

| Value     | What `init()` applies                                                                 |
| --------- | ------------------------------------------------------------------------------------- |
| `None`    | Nothing — full manual control.                                                        |
| `Desktop` | Always behind other windows. No title bar. Saves and restores last position and size. |
| `Overlay` | Always on top of other windows. No title bar.                                         |
| `Popup`   | Always on top. No title bar. Auto-hides on focus loss. Shown/hidden by triggers only. |

`Popup` widgets are not toggleable by the user from the settings panel — they only appear when explicitly triggered by
another widget.

### Instances

The `instances` field controls how many copies of the widget can run at the same time:

| Value              | Behavior                                                          |
| ------------------ | ----------------------------------------------------------------- |
| `Single`           | Only one instance allowed. Default.                               |
| `Multiple`         | The user can create as many instances as they want.               |
| `ReplicaByMonitor` | DockBar automatically creates one instance per connected monitor. |

---

## 5. User-Configurable Settings

The `settings` list lets users configure your widget from the DockBar settings panel without editing any files. Each
entry defines a control that reads and writes a value your widget can access at runtime.

Every setting has a unique `key` that your widget code uses to read the stored value.

> **Reserved keys:** `enabled` and `$instances` are reserved by DockBar and cannot be used as setting keys.

---

### Common Fields

All setting types share these base fields:

```yaml
key: my-setting # Unique identifier (required)
label: My Setting # Label shown in the settings panel (required)
description: Some help. # Extra text shown under the label (optional)
tip: A tooltip. # Tooltip shown on an icon next to the label (optional)
allowSetByMonitor: false # If true, the user can set different values per monitor
dependencies: # Keys that must be truthy for this setting to be active
  - some-other-key
```

---

### Switch

A toggle switch for boolean values.

```yaml
- type: switch
  key: show-seconds
  label: Show seconds
  defaultValue: true
```

---

### Select

A dropdown list or inline buttons for choosing one value from a fixed set.

```yaml
# Dropdown (default)
- type: select
  key: clock-format
  label: Time format
  defaultValue: "12h"
  options:
    - value: "12h"
      label: 12-hour
    - value: "24h"
      label: 24-hour

# Inline buttons (subtype: Inline)
- type: select
  key: position
  label: Position
  defaultValue: left
  subtype: Inline
  options:
    - value: left
      label: Left
    - value: center
      label: Center
    - value: right
      label: Right
```

Each option can also have an `icon` (a react-icons name) displayed alongside the label.

---

### InputText

A text input. Use `multiline: true` for a textarea.

```yaml
- type: text
  key: greeting
  label: Greeting message
  defaultValue: "Hello"
  maxLength: 100

- type: text
  key: custom-css
  label: Custom CSS
  defaultValue: ""
  multiline: true
```

---

### InputNumber

A numeric input with optional constraints.

```yaml
- type: number
  key: refresh-rate
  label: Refresh rate (ms)
  defaultValue: 1000
  min: 100
  max: 60000
  step: 100
```

---

### Range

A slider for numeric values.

```yaml
- type: range
  key: opacity
  label: Opacity
  defaultValue: 100
  min: 10
  max: 100
  step: 5
```

---

### Color

A color picker. Set `allowAlpha: true` to include a transparency channel.

```yaml
- type: color
  key: accent-color
  label: Accent color
  defaultValue: "#6c63ff"
  allowAlpha: true
```

---

### Font

A font picker that lets users choose a font family installed on their system.

```yaml
- type: font
  key: widget-font
  label: Font
  defaultValue: "Segoe UI"
```

The stored value is a font family name string (e.g. `"Segoe UI"`, `"JetBrains Mono"`). Apply it in your widget via CSS:

```js
document.documentElement.style.setProperty("--widget-font", settings["widget-font"]);
```

```css
.my-widget {
  font-family: var(--widget-font, "Segoe UI");
}
```

---

### Grouping Settings

Use `group` to organize settings into collapsible sections. Groups can be nested.

```yaml
settings:
  - type: switch
    key: enabled-clock
    label: Show clock

  - group:
      label: Appearance
      items:
        - type: color
          key: text-color
          label: Text color
          defaultValue: "#ffffff"

        - type: range
          key: font-size
          label: Font size
          defaultValue: 14
          min: 10
          max: 32
          step: 1
```

---

## 6. CSS — Use Global Classes, Not CSS Modules

Themes work by injecting CSS into your widget's webview and targeting your elements by class name. For this to work,
your class names must be **stable and global** — if you use CSS Modules (or any tool that hashes/scopes class names at
build time), the generated names will be unpredictable and theme authors won't be able to target them.

Use plain global class names in your HTML and CSS:

```html
<!-- Good — stable, targetable by themes -->
<div class="my-widget-toolbar">
  <button class="my-widget-btn">Click</button>
</div>
```

```css
/* Good — global, no scoping */
.my-widget-toolbar {
  display: flex;
}
.my-widget-btn {
  border-radius: 6px;
}
```

Avoid CSS Modules or any scoped/hashed class output:

```html
<!-- Avoid — theme authors can't rely on these names -->
<div class="toolbar_xK92a">
  <button class="btn_mP3z1">Click</button>
</div>
```

---

## 7. Inspecting Your Widget with DevTools

Every widget runs in an isolated webview. Click on your widget to focus it, then press **Ctrl + Shift + I** to open
DevTools for that specific widget.

From DevTools you can inspect the live DOM, tweak styles, run JavaScript in the console, and profile performance —
exactly like working in a browser. This is the fastest way to debug layout issues or verify that your settings values
are being applied correctly.

---

## 8. Folder Structure

```
my-widget/
├── metadata.yml   ← resource definition (references all other files)
├── index.html     ← HTML body content
├── index.js       ← bundled JavaScript
├── index.css      ← bundled CSS
└── i18n/          ← translations (optional)
    ├── display_name.yml
    └── description.yml
```

The HTML, JS, and CSS files must be explicitly referenced from `metadata.yml` via `!include`. You can name these files
however you like — the names in `metadata.yml` are the source of truth.
