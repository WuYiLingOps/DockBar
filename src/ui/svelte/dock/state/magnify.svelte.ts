// macOS-style magnification: each dock item rides a damped spring toward a
// target derived from the pointer distance. Icons stretch away from the dock
// while squashing slightly along the dock axis (elastic bulge) and shove
// neighbors aside. On pointer leave the targets return to rest and the springs
// keep integrating until everything settles, giving the smooth fall-back.

import { isHorizontalDock, settingsState } from "./settings.svelte.ts";

const REACH = 28; // px beyond an item's edge where the distance falloff ends
const MAX_GROW = 1.0; // scale gain at full strength (strength 100 → 2x, like macOS)
const NEIGHBOR_CAP = 0.15; // neighbors amplify at most this fraction of the gain
const STIFFNESS = 170;
const DAMPING = 20; // underdamped (ζ≈0.77) → elastic overshoot on entry/exit
const AXIS_SQUASH = 0.85; // dock-axis gain vs dock-normal gain (the squash)

interface ItemSpring {
  scale: number;
  scaleVel: number;
  push: number;
  pushVel: number;
}

const springs = new WeakMap<HTMLElement, ItemSpring>();

function springFor(el: HTMLElement): ItemSpring {
  let spring = springs.get(el);
  if (!spring) {
    spring = { scale: 1, scaleVel: 0, push: 0, pushVel: 0 };
    springs.set(el, spring);
  }
  return spring;
}

let raf: number | null = null;
let lastTick = 0;
let active = false;
let pointer: { x: number; y: number } | null = null;
let pointerMoved = false;
let installedScroller: HTMLElement | null = null;

function smoothstep(t: number): number {
  return t * t * (3 - 2 * t);
}

function setVar(el: HTMLElement, name: string, value: string) {
  if (el.style.getPropertyValue(name) !== value) {
    el.style.setProperty(name, value);
  }
}

function tick(now: number) {
  const dt = Math.min(Math.max((now - lastTick) / 1000, 0.001), 0.033);
  lastTick = now;

  const strength = (settingsState.value?.iconMagnification ?? 0) / 100;
  const horizontal = isHorizontalDock();
  const grow = strength * MAX_GROW;

  let settled = !active;

  // The layout positions only matter while the pointer moves (falloff targets
  // change) or while springs are still far from their targets. Reading rects
  // every frame would force layout on a dock that's mostly idle, so skip the
  // read pass entirely once the pointer has been still for a frame.
  const needRects = pointerMoved || (pointer !== null && grow > 0);
  pointerMoved = false;

  // read pass first (rects/sizes), write pass after — avoids layout thrashing
  const items: { el: HTMLElement; spring: ItemSpring; rect: DOMRect | null; half: number }[] = [];
  for (const overlay of Array.from(document.querySelectorAll<HTMLElement>(".dock-item-overlay"))) {
    const el = overlay.firstElementChild as HTMLElement | null;
    if (!el) continue;
    const spring = springFor(el);
    items.push({ el, spring, rect: null, half: 0 });
  }
  if (needRects) {
    for (const it of items) {
      it.rect = it.el.getBoundingClientRect();
      it.half = (horizontal ? it.el.offsetWidth : it.el.offsetHeight) / 2;
    }
  }

  for (const { el, spring, rect, half } of items) {
    let targetScale = 1;
    let targetPush = 0;
    if (pointer && grow > 0 && rect) {
      // undo the current push so targets derive from the layout position,
      // otherwise the shove feeds back into the falloff
      const center = (horizontal ? rect.left + rect.width / 2 : rect.top + rect.height / 2) - spring.push;
      const delta = (horizontal ? pointer.x : pointer.y) - center;
      const distance = Math.abs(delta);
      const falloff = distance <= half ? 1 : Math.max(0, 1 - (distance - half) / REACH);
      const eased = smoothstep(falloff);
      // Only the hovered icon gets the full gain; items past its edge taper
      // fast and are capped so neighbors bulge subtly instead of competing.
      // The shove must still clear the hovered icon's edge growth, so it
      // derives from the hovered icon's gain (approximated via the falloff
      // strength) plus the neighbor's own bulge.
      const neighborEased = Math.min(eased, NEIGHBOR_CAP);
      targetScale = 1 + grow * (distance <= half ? eased : neighborEased);
      const hoveredEdgeGain = (grow * eased * half) / 2; // half of hovered icon's width gain
      targetPush = distance <= half ? 0 : -Math.sign(delta) * (hoveredEdgeGain + half * (targetScale - 1));
    }

    // semi-implicit Euler on a damped spring
    spring.scaleVel += (targetScale - spring.scale) * STIFFNESS * dt;
    spring.scaleVel *= Math.exp(-DAMPING * dt);
    spring.scale += spring.scaleVel * dt;

    spring.pushVel += (targetPush - spring.push) * STIFFNESS * dt;
    spring.pushVel *= Math.exp(-DAMPING * dt);
    spring.push += spring.pushVel * dt;

    if (
      Math.abs(targetScale - spring.scale) > 0.0005 ||
      Math.abs(spring.scaleVel) > 0.001 ||
      Math.abs(targetPush - spring.push) > 0.05 ||
      Math.abs(spring.pushVel) > 0.05
    ) {
      settled = false;
    }

    const gain = Math.max(0, spring.scale - 1);
    // amplified items must paint above their neighbors while overlapping
    // during the spring transit
    const z = Math.round(gain * 100);
    if (el.style.zIndex !== String(z)) {
      el.style.zIndex = String(z);
    }
    if (horizontal) {
      setVar(el, "--my", (1 + gain).toFixed(4));
      setVar(el, "--mx", (1 + gain * AXIS_SQUASH).toFixed(4));
      setVar(el, "--push-x", `${spring.push.toFixed(2)}px`);
      setVar(el, "--push-y", "0px");
    } else {
      setVar(el, "--mx", (1 + gain).toFixed(4));
      setVar(el, "--my", (1 + gain * AXIS_SQUASH).toFixed(4));
      setVar(el, "--push-y", `${spring.push.toFixed(2)}px`);
      setVar(el, "--push-x", "0px");
    }
  }

  if (settled) {
    // Springs at rest: stop the loop entirely. A still pointer keeps its
    // targets, so nothing needs per-frame work until it moves again —
    // onPointerMove restarts the loop.
    raf = null;
    if (!active) {
      pointer = null;
      installedScroller?.removeAttribute("data-magnifying");
      if (installedScroller) installedScroller.style.overflow = "";
      document.body.removeAttribute("data-magnifying");
      document.documentElement.removeAttribute("data-magnifying");
      for (const overlay of Array.from(document.querySelectorAll<HTMLElement>(".dock-item-overlay"))) {
        const el = overlay.firstElementChild as HTMLElement | null;
        if (!el) continue;
        el.style.removeProperty("--mx");
        el.style.removeProperty("--my");
        el.style.removeProperty("--push-x");
        el.style.removeProperty("--push-y");
        el.style.removeProperty("z-index");
      }
    }
    return;
  }

  raf = requestAnimationFrame(tick);
}

function ensureLoop() {
  if (raf !== null) return;
  lastTick = performance.now();
  raf = requestAnimationFrame(tick);
}

function magnificationEnabled(): boolean {
  return (settingsState.value?.iconMagnification ?? 0) > 0;
}

function onPointerEnter(e: PointerEvent) {
  if (!magnificationEnabled()) return;
  active = true;
  pointer = { x: e.clientX, y: e.clientY };
  pointerMoved = true;
  installedScroller?.setAttribute("data-magnifying", "");
  document.body.setAttribute("data-magnifying", "");
  document.documentElement.setAttribute("data-magnifying", "");
  // inline wins over any stylesheet ordering/specificity subtlety
  if (installedScroller) installedScroller.style.overflow = "visible";
  ensureLoop();
}

function onPointerMove(e: PointerEvent) {
  if (!active) return;
  const next = { x: e.clientX, y: e.clientY };
  if (pointer && Math.abs(next.x - pointer.x) < 0.5 && Math.abs(next.y - pointer.y) < 0.5) {
    return;
  }
  pointer = next;
  pointerMoved = true;
  // springs may be sleeping at their targets; wake the loop for the new targets
  ensureLoop();
}

function onPointerLeave() {
  active = false;
  pointer = null;
  pointerMoved = true;
  // the loop keeps integrating until the springs settle at rest
  ensureLoop();
}

/** Starts the magnification listeners once the dock is rendered. */
export function setupMagnification(container: HTMLElement) {
  const scroller = container.closest<HTMLElement>(".dock-items-container") ?? container;
  if (scroller === installedScroller) return;
  installedScroller?.removeEventListener("pointerenter", onPointerEnter);
  installedScroller?.removeEventListener("pointermove", onPointerMove);
  installedScroller?.removeEventListener("pointerleave", onPointerLeave);

  installedScroller = scroller;
  scroller.addEventListener("pointerenter", onPointerEnter);
  scroller.addEventListener("pointermove", onPointerMove);
  scroller.addEventListener("pointerleave", onPointerLeave);
}
