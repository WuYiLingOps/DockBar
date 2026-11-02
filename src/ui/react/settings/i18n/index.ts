import type { SupportedLanguagesCode } from "@dockbar/lib";
import i18n from "i18next";
import yaml from "js-yaml";
import { initReactI18next } from "react-i18next";

i18n.use(initReactI18next).init(
  {
    lng: "en",
    fallbackLng: "en",
    interpolation: {
      escapeValue: false,
    },
    resources: {},
  },
  undefined,
);

async function loadTranslations() {
  const translations: Record<SupportedLanguagesCode, { default: string }> = {
    en: await import("./translations/en.yml"),
    "zh-CN": await import("./translations/zh-CN.yml"),
    "zh-TW": await import("./translations/zh-TW.yml"),
  };

  for (const [key, value] of Object.entries(translations)) {
    i18n.addResourceBundle(key, "translation", yaml.load(value.default));
  }
}

await loadTranslations();
export default i18n;
