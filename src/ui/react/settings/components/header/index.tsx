import { DockBarCommand, invoke } from "@dockbar/lib";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Button } from "antd";
import React from "react";
import { useTranslation } from "react-i18next";
import { NavLink, useLocation, useSearchParams } from "react-router";

import cs from "./index.module.css";
import { widgets as widgetList } from "../../state/resources.ts";
import { hasChanges, needRestart, restoreToLastSaved, saveSettings } from "../../state/mod.ts";

export const Header = () => {
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const { t } = useTranslation();

  const SaveOrQuit = async () => {
    if (hasChanges.value) {
      await saveSettings();
      if (needRestart.value) {
        await invoke(DockBarCommand.AppRestart);
      }
    } else {
      await getCurrentWebviewWindow().close();
    }
  };

  const saveBtnLabel = needRestart.value ? t("save_and_restart") : t("save");

  let label: React.ReactNode = <span>null!?</span>;
  let parts = location.pathname.split("/").filter(Boolean);

  if (parts[0] === "widget") {
    const widgetId = searchParams.get("id");
    const widget = widgetList.value.find((w) => w.id === widgetId);
    label = widget ? <ResourceText text={widget.metadata.displayName} /> : <span>{widgetId}</span>;
  } else {
    label = parts.map((part, idx) => (
      <React.Fragment key={part}>
        {idx !== parts.length - 1
          ? (
            <NavLink to={`/${parts.slice(0, idx + 1).join("/")}`} className={cs.part}>
              {t(`header.labels.${part}`)}
            </NavLink>
          )
          : <span className={cs.part}>{t(`header.labels.${part}`)}</span>}
        {++idx < parts.length ? ">" : ""}
      </React.Fragment>
    ));
  }

  return (
    <div className={cs.header}>
      <div className={cs.title}>
        {label}
      </div>
      <div className={cs.actions}>
        <Button
          style={{ minWidth: 60 }}
          type="default"
          danger
          disabled={!hasChanges.value}
          onClick={restoreToLastSaved}
        >
          {t("cancel")}
        </Button>
        <Button
          style={{ minWidth: 60 }}
          type="primary"
          danger={!hasChanges.value}
          onClick={SaveOrQuit}
        >
          {hasChanges.value ? saveBtnLabel : t("close")}
        </Button>
      </div>
    </div>
  );
};
