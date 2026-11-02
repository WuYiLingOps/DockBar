import { GoogleTranslator, ObjectTranslator } from "@seelen/translation-toolkit";
import { SupportedLanguages } from "@dockbar/lib";
import { existsSync, readFileSync, writeFileSync } from "fs";
import yaml from "js-yaml";

const targetLanguages = SupportedLanguages.filter((lang) => lang.value !== "en");

async function completeTranslationsFor(localesDir: string) {
  const translator = new GoogleTranslator({ source: "en" });

  const enPath = `${localesDir}/en.yml`;
  const enHashesPath = `${localesDir}/hash.yml`;

  let cachedHashTable: any = undefined;
  if (existsSync(enHashesPath)) {
    cachedHashTable = new Map(
      Object.entries(yaml.load(readFileSync(enHashesPath, "utf8")) as object),
    );
  }

  const strYaml = readFileSync(enPath, "utf8");
  const fileTranslator = new ObjectTranslator(
    yaml.load(strYaml) as any,
    translator,
    cachedHashTable,
  );

  for (const targetLang of targetLanguages) {
    const filePath = `${localesDir}/${targetLang.value}.yml`;

    let translation: any = {};
    if (existsSync(filePath)) {
      translation = yaml.load(readFileSync(filePath, "utf8"));
    }

    const translated = await fileTranslator.translate_to(targetLang.value, translation);
    writeFileSync(filePath, yaml.dump(translated));
  }

  const { obj, hashTable } = fileTranslator.source();

  writeFileSync(enPath, yaml.dump(obj)); // overwrite sorted
  writeFileSync(`${localesDir}/hash.yml`, yaml.dump(Object.fromEntries(hashTable)));
}

await completeTranslationsFor("src/ui/react/settings/i18n/translations");

await completeTranslationsFor("src/ui/svelte/dock/i18n/translations");
await completeTranslationsFor("src/ui/svelte/icon-editor/i18n/translations");

await completeTranslationsFor("src/background/i18n");
