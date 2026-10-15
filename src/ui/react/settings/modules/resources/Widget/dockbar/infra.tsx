import { DockBarMode, DockBarSide, HideMode, WegAttentionReveal, WegMiddleClickAction } from "@dockbar/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { $is_touch_primary } from "libs/ui/react/utils/signals";
import { Button, InputNumber, message, Select, Slider, Switch, Tooltip } from "antd";
import { useTranslation } from "react-i18next";

import { OptionsFromEnum } from "../../../shared/utils/app.ts";
import { getWegConfig, importFromWindowsTaskbar, patchWegConfig } from "./application.ts";
import { getDevTools } from "../../../developer/application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../../components/SettingsBox/index.tsx";
import Compact from "antd/es/space/Compact";

export const DockBarSettings = () => {
  const settings = getWegConfig();
  const isTouchPrimary = $is_touch_primary.value;
  const devTools = getDevTools();

  const { t } = useTranslation();

  const handleImportTaskbarItems = async () => {
    try {
      const count = await importFromWindowsTaskbar();
      if (count === 0) {
        message.info(t("dock.import_no_items"));
      } else {
        message.success(t("dock.import_success", { count }));
      }
    } catch (error) {
      message.error(t("dock.import_error"));
      console.error("Failed to import taskbar items:", error);
    }
  };

  return (
    <>
      <SettingsGroup>
        <SettingsSubGroup label={t("dock.label")}>
          <SettingsOption>
            <div>{t("dock.width")}</div>
            <Select
              style={{ width: "120px" }}
              value={settings.mode}
              options={OptionsFromEnum(t, DockBarMode, "dock.mode")}
              onChange={(value) => patchWegConfig({ mode: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.dock_side")}</div>
            <Compact>
              {Object.values(DockBarSide).map((side) => (
                <Button
                  key={side}
                  type={side === settings.position ? "primary" : "default"}
                  onClick={() => patchWegConfig({ position: side })}
                >
                  <Icon iconName={`CgToolbar${side}`} size={18} />
                </Button>
              ))}
            </Compact>
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.margin")}</div>
            <InputNumber
              value={settings.margin}
              onChange={(value) => patchWegConfig({ margin: value || 0 })}
              min={0}
              max={40}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.padding")}</div>
            <InputNumber
              value={settings.padding}
              onChange={(value) => patchWegConfig({ padding: value || 0 })}
              min={0}
              max={40}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption>
              <b>{t("dock.auto_hide")}</b>
              {/* disabled on touch devices: autohide requires hover/pointer events that touchscreens don't fire */}
              <Tooltip title={isTouchPrimary ? t("dock.auto_hide_touch_disabled") : undefined}>
                <Select
                  style={{ width: "120px" }}
                  value={settings.hideMode}
                  options={OptionsFromEnum(t, HideMode, "dock.hide_mode")}
                  onChange={(value) => patchWegConfig({ hideMode: value })}
                  disabled={isTouchPrimary}
                />
              </Tooltip>
            </SettingsOption>
          }
        >
          <SettingsOption>
            <span>{t("dock.delay_to_show")} (ms)</span>
            <InputNumber
              value={settings.delayToShow}
              min={0}
              max={10000}
              disabled={settings.hideMode === HideMode.Never || isTouchPrimary}
              onChange={(value) => patchWegConfig({ delayToShow: value || 0 })}
            />
          </SettingsOption>
          <SettingsOption>
            <span>{t("dock.delay_to_hide")} (ms)</span>
            <InputNumber
              value={settings.delayToHide}
              min={0}
              max={10000}
              disabled={settings.hideMode === HideMode.Never || isTouchPrimary}
              onChange={(value) => patchWegConfig({ delayToHide: value || 0 })}
            />
          </SettingsOption>
          <SettingsOption>
            <span>{t("dock.reveal_on_attention.label")}</span>
            <Select
              style={{ width: "160px" }}
              value={settings.revealOnAttention}
              options={OptionsFromEnum(t, WegAttentionReveal, "dock.reveal_on_attention")}
              disabled={settings.hideMode === HideMode.Never || isTouchPrimary}
              onChange={(value) => patchWegConfig({ revealOnAttention: value })}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("dock.filtering")}>
          <SettingsOption>
            <div>{t("dock.items.temporal_visibility.label")}</div>
            <Select
              style={{ width: "120px" }}
              value={settings.temporalItemsVisibility}
              options={[
                { value: "All", label: t("dock.items.temporal_visibility.all") },
                {
                  value: "OnMonitor",
                  label: t("dock.items.temporal_visibility.on_monitor"),
                },
              ]}
              onChange={(value) => patchWegConfig({ temporalItemsVisibility: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.items.pinned_visibility.label")}</div>
            <Select
              style={{ width: "120px" }}
              value={settings.pinnedItemsVisibility}
              options={[
                {
                  value: "Always",
                  label: t("dock.items.pinned_visibility.always"),
                },
                {
                  value: "WhenPrimary",
                  label: t("dock.items.pinned_visibility.when_primary"),
                },
              ]}
              onChange={(value) => patchWegConfig({ pinnedItemsVisibility: value })}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("dock.items.label")}>
          <SettingsOption>
            <div>{t("dock.items.size")}</div>
            <InputNumber
              value={settings.size}
              onChange={(value) => patchWegConfig({ size: value || 0 })}
              min={16}
              max={128}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.items.gap")}</div>
            <InputNumber
              value={settings.spaceBetweenItems}
              onChange={(value) => patchWegConfig({ spaceBetweenItems: value || 0 })}
              min={0}
              max={40}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.items.show_window_title")}</div>
            <Switch
              checked={settings.showWindowTitle}
              onChange={(value) => patchWegConfig({ showWindowTitle: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.items.show_instance_counter")}</div>
            <Switch
              checked={settings.showInstanceCounter}
              onChange={(value) => patchWegConfig({ showInstanceCounter: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.items.hover_show_app_name")}</div>
            <Switch
              checked={settings.hoverShowAppName}
              onChange={(value) => patchWegConfig({ hoverShowAppName: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.items.icon_magnification")}</div>
            <Slider
              style={{ width: "200px" }}
              min={0}
              max={100}
              step={5}
              value={settings.iconMagnification}
              onChange={(value) => patchWegConfig({ iconMagnification: value })}
            />
          </SettingsOption>
          <SettingsOption>
            <div>{t("dock.items.split_windows")}</div>
            <Switch
              checked={settings.splitWindows}
              onChange={(value) => patchWegConfig({ splitWindows: value })}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("dock.items.middle_click_action.label")}
          action={
            <Select
              style={{ width: "160px" }}
              value={settings.middleClickAction}
              options={[
                {
                  value: WegMiddleClickAction.CloseApp,
                  label: t("dock.items.middle_click_action.close_app"),
                },
                {
                  value: WegMiddleClickAction.OpenNewInstance,
                  label: t("dock.items.middle_click_action.open_new_instance"),
                },
              ]}
              onChange={(value) => patchWegConfig({ middleClickAction: value })}
            />
          }
        />
      </SettingsGroup>

      {devTools && (
        <SettingsGroup>
          <SettingsOption>
            <b>{t("dock.show_end_task")}</b>
            <Switch
              checked={settings.showEndTask}
              onChange={(value) => patchWegConfig({ showEndTask: value })}
            />
          </SettingsOption>
        </SettingsGroup>
      )}

      <SettingsGroup>
        <SettingsOption>
          <div>{t("dock.import_from_taskbar.label")}</div>
          <Button onClick={handleImportTaskbarItems}>
            {t("dock.import_from_taskbar.button")}
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </>
  );
};
