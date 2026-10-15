import { useDarkMode } from "libs/ui/react/utils/styling.ts";
import { ConfigProvider, theme } from "antd";
import { useEffect } from "react";
import { Color, Widget } from "@dockbar/lib";

import { Routing } from "./router.tsx";
import { WelcomeModal } from "./components/WelcomeModal/infra.tsx";
import { uiColors } from "./state/system.ts";

export function App() {
  const isDarkMode = useDarkMode();

  useEffect(() => {
    Widget.self.ready();
  }, []);

  const color = new Color(isDarkMode ? uiColors.value.accent_light : uiColors.value.accent_dark);

  return (
    <ConfigProvider
      componentSize="small"
      theme={{
        token: {
          colorPrimary: color.toHexString(),
        },
        algorithm: isDarkMode ? theme.darkAlgorithm : theme.defaultAlgorithm,
      }}
    >
      <Routing />
      <WelcomeModal />
    </ConfigProvider>
  );
}
