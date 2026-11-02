export type SupportedLanguagesCode = (typeof SupportedLanguages)[number]["value"];
export interface SupportedLanguage {
  label: string;
  enLabel: string;
  /** language code @example 'de' 'es' 'zh' 'en-US' 'en-UK' */
  value: string;
}

export const SupportedLanguages = [
  { label: "English", enLabel: "English", value: "en" },
  { label: "中文 (简体)", enLabel: "Chinese (Simplified)", value: "zh-CN" },
  { label: "中文 (繁體)", enLabel: "Chinese (Traditional)", value: "zh-TW" },
] as const;
