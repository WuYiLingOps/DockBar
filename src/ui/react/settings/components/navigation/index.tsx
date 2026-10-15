import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { Tooltip } from "antd";
import { memo, useState } from "react";
import { useTranslation } from "react-i18next";
import { NavLink, useLocation } from "react-router";

import { cx } from "../../modules/shared/utils/app.ts";

import { RouteIcons, RoutePath } from "./routes.tsx";
import cs from "./index.module.css";
import { widgets as widgetList } from "../../state/resources.ts";
export const Navigation = memo(() => {
  const [collapsed, setCollapsed] = useState(false);

  const { t } = useTranslation();
  const location = useLocation();

  const Mapper = (route: RoutePath | null) => {
    if (!route) return null;
    return (
      <Item
        key={route}
        route={route}
        isActive={location.pathname.startsWith(route)}
        collapsed={collapsed}
        label={t(`header.labels.${route.replace("/", "")}`)}
        icon={RouteIcons[route]}
      />
    );
  };

  const advanceGroup = [
    RoutePath.SettingsByMonitor,
    RoutePath.Shortcuts,
  ];
  const devGroup = [RoutePath.DevTools];

  return (
    <div
      className={cx(cs.navigation, {
        [cs.collapsed!]: collapsed,
      })}
    >
      <div className={cs.header} onClick={() => setCollapsed(!collapsed)}>
        <img src="./logo.svg" />
        <h1>DockBar</h1>
        <Icon className={cs.chevron} iconName="FaChevronLeft" />
      </div>

      <div className={cs.body}>
        <div className={cs.group}>
          {[RoutePath.General].map(Mapper)}
        </div>

        <div className={cs.separator} />
        <div className={cs.group}>
          {widgetList.value
            .filter((widget) => !widget.hidden)
            .map((widget) => (
              <Item
                key={widget.id}
                route={`/widget?${new URLSearchParams({ id: widget.id })}`}
                isActive={location.pathname === "/widget" &&
                  new URLSearchParams(location.search).get("id") === widget.id}
                collapsed={collapsed}
                label={<ResourceText text={widget.metadata.displayName} />}
                icon={<Icon iconName={(widget.icon as any) || "BiSolidWidget"} />}
              />
            ))}
        </div>

        <div className={cs.separator} />
        <div className={cs.group}>{advanceGroup.map(Mapper)}</div>

        <div className={cs.separator} />
        <div className={cs.group}>{devGroup.map(Mapper)}</div>
      </div>
    </div>
  );
});

interface ItemProps {
  route: string;
  isActive: boolean;
  collapsed: boolean;
  icon?: React.ReactNode;
  label: React.ReactNode;
  badge?: boolean;
}

const Item = ({ route, icon, label, isActive, collapsed, badge }: ItemProps) => {
  return (
    <Tooltip placement="right" title={collapsed ? label : null}>
      <NavLink
        to={route}
        className={cx(cs.item, {
          [cs.active!]: isActive,
        })}
      >
        <div className={cs.iconWrapper}>
          {icon}
          {badge && <span className={cs.badge} />}
        </div>
        <span className={cs.label}>{label}</span>
      </NavLink>
    </Tooltip>
  );
};
