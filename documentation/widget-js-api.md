# Widget JS API — `@dockbar/lib`

This page covers the runtime JS/TS API a widget actually calls once it's loaded — how it initializes itself, how it
calls backend commands, and how it listens for backend events. Read [widget guidelines](./widget-guidelines) first for
how a widget is declared and packaged; this page is about what your widget's code does once it's running.

---

## 1. Lifecycle — `Widget.self`, `init()`, `ready()`

Every widget runs inside a webview that DockBar injects a runtime handle into. Access it with:

```ts
import { Widget } from "@dockbar/lib";

const widget = Widget.self;
```

This throws if the code is not actually running inside a DockBar widget webview.

> `Widget.getCurrent()` still exists but is deprecated — use the `Widget.self` getter instead.

### `init()`

Call this **before any other action**. It applies the widget's `preset` behavior (sizing, positioning, theming) but does
**not** show the widget yet — the widget stays in a `pending` state.

```ts
await widget.init({ ...options });
```

All options are optional — omit anything you want the preset's default behavior for. The full `InitWidgetOptions` type,
its fields, and their defaults are documented via doc comments in `libs/core/src/state/widget/interfaces.ts` — read that
file directly rather than relying on a copy of the field list here.

### `ready()`

Call this once your widget has actually mounted and is visually ready. It marks the widget `ready`, runs the auto-sizer,
and — unless the widget is `lazy` or you pass `show: false` — shows the window. Calling `ready()` before `init()`
throws.

```ts
await widget.ready(); // show: !widget.lazy by default
```

`ready()` is also what flushes a pending **trigger** event — if something tried to open this widget (e.g. another widget
calling `trigger(widgetId)`) before the widget finished loading, that trigger is queued and delivered once `ready()`
runs.

### Typical bootstrap

```ts
import { Widget } from "@dockbar/lib";
import { mount } from "./app"; // your framework's mount function

const widget = Widget.self;
await widget.init();
mount(document.getElementById("root")!);
await widget.ready();
```

Both methods, their full option types (`InitWidgetOptions`, `ReadyWidgetOptions`), and their doc comments live in
`libs/core/src/state/widget/mod.ts` and `libs/core/src/state/widget/interfaces.ts` — read those directly if you need
behavior not summarized above.

---

## 2. Calling the backend — `invoke`

```ts
import { DockBarCommand, invoke } from "@dockbar/lib";

const workspaces = await invoke(DockBarCommand.StateGetVirtualDesktops);
await invoke(DockBarCommand.SwitchWorkspace, { workspaceId });
```

`invoke` is a thin, strongly-typed wrapper around Tauri's own `invoke` — the command name and argument/return types are
all inferred from the `DockBarCommand` enum value you pass, so passing the wrong argument shape is a compile-time error,
not a runtime one.

**Do not look for a list of commands in this doc.** Every backend command DockBar exposes is declared in one place, and
that declaration is the only source of truth (names, arguments, and return types all live together, and the
`DockBarCommand` enum plus the TS argument/return types are generated straight from it):

```
libs/core/src/handlers/commands.rs
```

Read the `slu_commands_declaration! { ... }` block there. After a change to that file, `DockBarCommand` and its typed
argument/return maps are regenerated into `libs/core/src/handlers/commands.ts` — never edit that generated file by hand.

---

## 3. Listening for backend events — `subscribe`

```ts
import { DockBarEvent, subscribe } from "@dockbar/lib";

const unsubscribe = await subscribe(DockBarEvent.VirtualDesktopsChanged, (event) => {
  console.log(event.payload); // typed as VirtualDesktops
});

// later, if needed
unsubscribe();
```

`subscribe` is a thin, strongly-typed wrapper around Tauri's `listen` — the payload type is inferred from the
`DockBarEvent` value you pass.

**Do not look for a list of events in this doc**, for the same reason as commands — the single source of truth is:

```
libs/core/src/handlers/events.rs
```

Read the `slu_events_declaration! { ... }` block there (`EventName(PayloadType) as "wire-event-name"`). The
`DockBarEvent` enum and its typed payload map are regenerated from this file into `libs/core/src/handlers/events.ts` —
never edit that generated file by hand.

---

## 4. Regenerating bindings after touching either file

If you add or change a command or event in the Rust files above, regenerate the TS bindings before using the new name
from a widget:

```bash
cd libs/core && deno task build:rs
```

Both `invoke` and `subscribe` themselves — the wrapper functions shown in sections 2 and 3 — are defined in
`libs/core/src/handlers/mod.ts`, which also re-exports the generated `DockBarCommand`/`DockBarEvent` enums. That file is
the right place to look if you need to understand exactly how the typing or the Tauri call is wired, beyond what's shown
here.
