import { DockBarCommand, DockBarEvent, invoke, subscribe } from "@dockbar/lib";
import type { WidgetId, WidgetPerm, WidgetPermState } from "@dockbar/lib/types";
import { computed, signal } from "@preact/signals";

export type WidgetPermissions = { [key in WidgetId]?: { [key in WidgetPerm]?: WidgetPermState } };

export const widgetPermissions = signal<WidgetPermissions>(await invoke(DockBarCommand.GetWidgetPermissions));
const initialPermissions = signal(JSON.stringify(widgetPermissions.value));
subscribe(DockBarEvent.WidgetPermissionsChanged, ({ payload }) => {
  widgetPermissions.value = payload;
  initialPermissions.value = JSON.stringify(payload);
});

export const hasPermissionsChanges = computed(
  () => initialPermissions.value !== JSON.stringify(widgetPermissions.value),
);

export function setWidgetPermission(widgetId: WidgetId, perm: WidgetPerm, state: WidgetPermState) {
  const current = widgetPermissions.value;
  widgetPermissions.value = { ...current, [widgetId]: { ...current[widgetId], [perm]: state } };
}

export async function savePermissions() {
  const permissions = widgetPermissions.value;
  initialPermissions.value = JSON.stringify(permissions);
  await invoke(DockBarCommand.SetWidgetPermissions, { permissions });
}

export function restorePermissionsToLastSaved() {
  widgetPermissions.value = JSON.parse(initialPermissions.value);
}
