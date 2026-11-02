pub const SUPPORTED_LANGUAGES: &[SupportedLanguage] = &[
    lang("English", "English", "en"),
    lang("中文 (简体)", "Chinese (Simplified)", "zh-CN"),
    lang("中文 (繁體)", "Chinese (Traditional)", "zh-TW"),
];

pub struct SupportedLanguage {
    pub label: &'static str,
    pub en_label: &'static str,
    pub value: &'static str,
}

const fn lang(
    label: &'static str,
    en_label: &'static str,
    value: &'static str,
) -> SupportedLanguage {
    SupportedLanguage {
        label,
        en_label,
        value,
    }
}
