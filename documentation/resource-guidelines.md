# DockBar — Resource Guidelines

Reference guide for creating and sharing resources — themes, widgets, icon packs, or wallpapers — for DockBar UI.

---

## Table of Contents

1. [What is a Resource?](#1-what-is-a-resource)
2. [Resource Kinds](#2-resource-kinds)
3. [Resource ID — Naming Your Resource](#3-resource-id--naming-your-resource)
4. [Metadata — Describing Your Resource](#4-metadata--describing-your-resource)
5. [Supporting Multiple Languages](#5-supporting-multiple-languages)
6. [Extended YAML — Splitting Your Files](#6-extended-yaml--splitting-your-files)
   - [!include — embed a file as text](#include--embed-a-file-as-text)
   - [!extend — embed a YAML file](#extend--embed-a-yaml-file)
7. [Folder Structure](#7-folder-structure)
8. [Loading and Unloading Resources](#8-loading-and-unloading-resources)
9. [Bundling for Publication](#9-bundling-for-publication)

---

## 1. What is a Resource?

A **resource** is a folder (or a single file) that you create and that DockBar can load to add new visuals or
functionality. You write a `metadata.yml` file that describes your resource, and optionally other files (stylesheets,
scripts, translations) that it references.

Once your resource is ready you can:

- **Use it locally** by placing the folder in the DockBar resources directory.
- **Share it** by publishing it to the DockBar marketplace.

---

## 2. Resource Kinds

Choose the kind that matches what you want to create:

| Kind        | What it does                                                        |
| ----------- | ------------------------------------------------------------------- |
| `Theme`     | Customizes the look of widgets with CSS styles and color variables. |
| `Widget`    | A new UI component with its own HTML, JS, CSS, and user settings.   |
| `IconPack`  | Replaces app icons and file-type icons system-wide.                 |
| `Wallpaper` | An image, video, or CSS-animated wallpaper.                         |

Each kind shares the same basic structure described in this guide, and then adds its own specific fields. Separate
guidelines cover each kind in detail.

---

## 3. Resource ID — Naming Your Resource

Every resource needs a unique ID. You choose this ID yourself when you create the resource. The format is:

```
@your-username/resource-name
```

Examples:

```
@johndoe/my-dark-theme
@johndoe/weather-widget
@acme/corporate-icons
```

**Rules:**

- Must start with `@`.
- Your username: 3–32 characters, letters, digits and hyphens, must start and end with a letter or digit.
- The resource name: at least 3 characters, letters, digits and hyphens, must start and end with a letter or digit.

> When you publish a resource to the marketplace it gets a permanent internal ID assigned automatically. You don't need
> to worry about that — you always use your `@username/name` when writing resource files.

---

## 4. Metadata — Describing Your Resource

Every resource has a `metadata` block. This is the information shown in the marketplace and in the DockBar settings
panel.

```yaml
metadata:
  # Name shown in the UI (required)
  displayName: My Dark Theme

  # Short description (required)
  description: A clean dark theme for all widgets.

  # Keywords that help users find your resource
  tags:
    - dark
    - minimal
    - toolbar

  # Square preview image — 1:1 aspect ratio (optional)
  # portrait: uploaded via the DockBar website (see note below)

  # Wide promotional banner — 21:9 aspect ratio (optional)
  # banner: uploaded via the DockBar website (see note below)

  # Screenshots — 16:9 aspect ratio, as many as you want (optional)
  # screenshots: uploaded via the DockBar website (see note below)

  # Oldest DockBar version your resource is compatible with (optional)
  appTargetVersion: [1, 11, 0]
```

The only fields that are truly required are `displayName` and `description`. Everything else is optional but filling
them in makes your resource look much better in the marketplace.

> **Images (portrait, banner, screenshots):** these are uploaded directly through the DockBar website when you publish
> or edit your resource — you do not write URLs in `metadata.yml` yourself. Only images hosted on `dockbar.io` are
> accepted; any other URL will be ignored and DockBar will display the default resource icon as the portrait instead.

---

## 5. Supporting Multiple Languages

`displayName`, `description`, and any user-visible text in your resource support multiple languages out of the box.

**Simple — English only:**

```yaml
displayName: My Dark Theme
```

**Localized — multiple languages:**

```yaml
displayName:
  en: My Dark Theme
  es: Mi Tema Oscuro
  de: Mein Dunkles Theme
  fr: Mon Thème Sombre
  pt-BR: Meu Tema Escuro
```

When a user's language is not in your list, DockBar automatically falls back to English. **If you provide a language
map, you must always include `en`.**

Use whatever languages you are comfortable with. The more the better, but English alone is perfectly fine.

> **Automatic translation:** DockBar ships a `dockbar-cli` CLI command that fills in all the supported languages in a
> single translation file. Write your source language entry first, then run:
>
> ```bash
> dockbar-cli resource translate <path/to/file.yml>
> ```
>
> For example, given `i18n/display_name.yml` with only:
>
> ```yaml
> en: My Dark Theme
> ```
>
> Running the command on that file will complete it with every other language DockBar supports. If your source text is
> not in English, pass the language code explicitly:
>
> ```bash
> dockbar-cli resource translate i18n/display_name.yml es
> ```
>
> Run the command once per translation file. Always review the results — machine translations can be inaccurate.
>
> `displayName`/`description` are just two examples of a `ResourceText` field — the same type and the same
> `slu
> resource translate` workflow apply to every user-facing label, description, and tooltip across widgets, themes,
> and shortcuts. See [ResourceText](./resource-text) for the full reference.

---

## 6. Extended YAML — Splitting Your Files

Resource files are written in YAML. DockBar extends standard YAML with two special tags that let you pull in content
from other files. This keeps your main `metadata.yml` clean and each piece of content in its own focused file.

### `!include` — embed a file as text

Reads a file and inserts its content as a text string.

```yaml
key: !include path/to/file.ext
```

- The path is relative to the folder where your `metadata.yml` lives.
- **`.scss` and `.sass` files are automatically compiled to CSS** before being inserted.
- Any other file is inserted as plain text.

Typical uses:

```yaml
# A theme inserting its stylesheets
styles:
  "@dockbar/fancy-toolbar": !include styles/toolbar.scss
  "@dockbar/dock": !include styles/dock.css
```

### `!extend` — embed a YAML file

Reads another YAML file and inserts its contents as a YAML value (a map, a list, anything).

```yaml
key: !extend path/to/file.yml
```

The most common use is splitting translations into their own files:

```yaml
# metadata.yml
metadata:
  displayName: !extend i18n/display_name.yml
  description: !extend i18n/description.yml
```

```yaml
# i18n/display_name.yml
en: My Dark Theme
es: Mi Tema Oscuro
de: Mein Dunkles Theme
```

After loading, the result is exactly the same as if you had written the translations inline. `!extend` files are
themselves parsed with the same rules, so you can use `!include` and `!extend` inside them too.

**When to use each:**

| Tag        | Use it for                                        |
| ---------- | ------------------------------------------------- |
| `!include` | CSS, SCSS, JavaScript, or any text file.          |
| `!extend`  | YAML content — translations, settings lists, etc. |

---

## 7. Folder Structure

A resource can be a single `metadata.yml` file, but for anything real you will want a folder. DockBar looks for an
entrypoint file inside the folder by checking these names in order: `metadata.yml`, `metadata.yaml`, `index.yml`,
`mod.yml`, `main.yml` (and their `.json` variants).

**Always name your entrypoint `metadata.yml`.** It is the clearest and most expected name.

A typical resource folder looks like this:

```
my-dark-theme/
├── metadata.yml          ← entrypoint: id, metadata, kind-specific fields
├── i18n/
│   ├── display_name.yml  ← translations for the name
│   └── description.yml   ← translations for the description
└── styles/
    ├── toolbar.scss       ← styles referenced with !include
    └── dock.scss
```

There are no other required files or folder names — only `metadata.yml` is mandatory. The rest is entirely up to you,
though following a consistent layout (like the one above) makes maintenance easier.

---

## 8. Loading and Unloading Resources

While developing a resource you can load it into a running DockBar instance directly from any folder on your machine
using the `dockbar-cli` CLI — no need to copy files anywhere.

### Load

```bash
dockbar-cli resource load <kind> <path>
```

- `<kind>` — one of: `theme`, `widget`, `icon-pack`, `wallpaper`
- `<path>` — absolute or relative path to the resource file or folder

Examples:

```bash
dockbar-cli resource load theme ./my-dark-theme
dockbar-cli resource load widget C:\Users\me\projects\my-clock
```

The resource is registered immediately and available in DockBar settings without restarting the app. **DockBar must be
running** for this command to work.

### Unload

```bash
dockbar-cli resource unload <kind> <path>
```

Removes the resource from the registry using the same path you used to load it.

```bash
dockbar-cli resource unload theme ./my-dark-theme
```

> Loaded resources are registered for the current session. After restarting DockBar you will need to load them again, or
> install them permanently through the settings panel.

---

## 9. Bundling for Publication

If you plan to share your resource on the DockBar marketplace, you need to bundle it into a single distributable file
first. Bundling resolves all `!include` and `!extend` references, compiles SCSS, and produces a self-contained `.yaml`
file ready to upload to the website.

If your resource is for personal use only, bundling is not required — the `load` / `unload` workflow is enough.

### Bundle

```bash
dockbar-cli resource bundle <kind> <path>
```

- `<kind>` — one of: `theme`, `widget`, `icon-pack`, `wallpaper`
- `<path>` — absolute or relative path to the resource file or folder

Examples:

```bash
dockbar-cli resource bundle theme ./my-dark-theme
dockbar-cli resource bundle widget C:\Users\me\projects\my-clock
```

The output file is written next to the resource folder (or file) with the `.yaml` extension. Upload that file through
the DockBar website to publish or update your resource in the marketplace.
