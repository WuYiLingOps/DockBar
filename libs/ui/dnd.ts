import type { DragOperation, Modifiers, Plugins, Sensors } from "@dnd-kit/abstract";
import { configure, Modifier } from "@dnd-kit/abstract";
import { DragDropManager, PointerActivationConstraints, PointerSensor } from "@dnd-kit/dom";

function DND_PLUGINS(defaults: Plugins): Plugins {
  return [...defaults];
}

const DND_SENSORS: Sensors = [
  PointerSensor.configure({
    // Allow dragging from buttons and other interactive elements inside items.
    preventActivation: () => false,
    // Prevents unintentional drags on click by requiring a 5px movement before activation.
    activationConstraints: (event, _source) => {
      const { pointerType } = event;
      switch (pointerType) {
        case "mouse":
          return [new PointerActivationConstraints.Distance({ value: 5 })];
        case "touch":
          return [new PointerActivationConstraints.Delay({ value: 250, tolerance: 5 })];
        default:
          return [
            new PointerActivationConstraints.Delay({ value: 200, tolerance: 10 }),
            new PointerActivationConstraints.Distance({ value: 5 }),
          ];
      }
    },
  }),
  /* KeyboardSensor.configure({
    preventActivation: () => true,
  }), */
];

interface RestrictToBoundsOptions {
  /** CSS selector resolving the container the drag must stay within. */
  selector: string;
}

/**
 * Clamps the drag transform so the dragged element never leaves the bounds of
 * the container resolved by `options.selector`. The drag overlay is positioned
 * from the same transform, so without this it follows the pointer anywhere
 * inside the dock's webview — including the empty strip below the dock bar,
 * where the overlay pokes out from behind the dock's rounded corners as a
 * stray white sliver at the bottom of the screen.
 */
class RestrictToBounds extends Modifier<DragDropManager, RestrictToBoundsOptions> {
  private bounds: DOMRect | null = null;

  apply({ transform, source }: DragOperation): { x: number; y: number } {
    // `element` is declared on the @dnd-kit/dom Draggable extension, but
    // DragOperation's `source` is typed with the abstract Draggable
    const element = (source as { element?: Element } | null)?.element;
    if (!element) return transform;

    // cache the bounds on the first apply: the container never moves mid-drag,
    // and re-querying on every pointer move forces layout
    if (!this.bounds) {
      const container = document.querySelector(this.options?.selector ?? "");
      if (!container) return transform;
      this.bounds = container.getBoundingClientRect();
    }
    const bounds = this.bounds;

    const rect = element.getBoundingClientRect();
    const maxX = Math.max(bounds.right - rect.width, bounds.left);
    const maxY = Math.max(bounds.bottom - rect.height, bounds.top);

    return {
      x: Math.min(Math.max(rect.left + transform.x, bounds.left), maxX) - rect.left,
      y: Math.min(Math.max(rect.top + transform.y, bounds.top), maxY) - rect.top,
    };
  }
}

const RestrictToBoundsDescriptor = configure(RestrictToBounds, { selector: ".taskbar" });

export function createDragDropManager(options?: { modifiers?: Modifiers }) {
  return new DragDropManager({
    sensors: DND_SENSORS,
    plugins: DND_PLUGINS,
    modifiers: options?.modifiers,
  });
}

export { RestrictToBounds, RestrictToBoundsDescriptor };
