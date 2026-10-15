import { DockBarCommand, DockBarEvent, invoke, subscribe } from "@dockbar/lib";
import { batch, computed, effect, signal } from "@preact/signals";
import { Modal } from "antd";
import { monitors } from "./system";
import { cloneDeep } from "lodash";
import i18n from "../i18n";
import { hasPermissionsChanges, restorePermissionsToLastSaved, savePermissions } from "./permissions";

export const DEFAULT_SETTINGS = await invoke(DockBarCommand.StateGetDefaultSettings);

export const settings = signal(await invoke(DockBarCommand.StateGetSettings, { path: null }));
const initialSettings = signal(JSON.stringify(settings.value));
subscribe(DockBarEvent.StateSettingsChanged, ({ payload }) => {
  settings.value = payload;
  initialSettings.value = JSON.stringify(payload);
});

export const language = computed(() => settings.value.language);

const hasSettingsChanges = computed(() => initialSettings.value !== JSON.stringify(settings.value));
export const hasChanges = computed(() => hasSettingsChanges.value || hasPermissionsChanges.value);
export const needRestart = signal(false);

export async function saveSettings() {
  try {
    if (hasPermissionsChanges.value) {
      await savePermissions();
    }
  } catch (error) {
    Modal.error({
      title: "Error on Save",
      content: String(error),
      centered: true,
    });
  }

  if (!hasSettingsChanges.value) {
    return;
  }

  const s = settings.value;

  try {
    initialSettings.value = JSON.stringify(s);
    await invoke(DockBarCommand.StateWriteSettings, {
      settings: s,
    });
  } catch (error) {
    Modal.error({
      title: "Error on Save",
      content: String(error),
      centered: true,
    });
  }
}

const defaultMonitorConfig = await invoke(DockBarCommand.StateGetDefaultMonitorSettings);
effect(() => {
  const sanitized = settings.peek();
  for (const monitor of monitors.value) {
    if (!sanitized.monitorsV3[monitor.id]) {
      sanitized.monitorsV3[monitor.id] = cloneDeep(defaultMonitorConfig);
    }
  }
  batch(() => {
    settings.value = sanitized;
    initialSettings.value = JSON.stringify(sanitized);
  });
});

effect(() => {
  i18n.changeLanguage(language.value);
});

export function restoreToLastSaved() {
  settings.value = JSON.parse(initialSettings.value);
  restorePermissionsToLastSaved();
}
