import { guardedSave } from "$lib/utils/dialogGuard";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { snackbar } from "$lib/stores/snackbarStore.svelte";
import { t } from "$lib/i18n";

export interface SchemeInfo {
  id: string;
  name: string;
  langCoverage: string;
  levels: string;
  description: string;
  templateContent: string;
  defaultFilename: string;
}

import rawCefrEn from "$lib/data/difficulty/cefr_en.tsv?raw";
import rawCefrIt from "$lib/data/difficulty/cefr_it.tsv?raw";
import rawCefrEs from "$lib/data/difficulty/cefr_es.tsv?raw";
import rawCefrFr from "$lib/data/difficulty/cefr_fr.tsv?raw";
import rawCefrDe from "$lib/data/difficulty/cefr_de.tsv?raw";
import rawCefrRu from "$lib/data/difficulty/cefr_ru.tsv?raw";
import rawCefrPt from "$lib/data/difficulty/cefr_pt.tsv?raw";
import rawHsk from "$lib/data/difficulty/hsk.tsv?raw";
import rawTocfl from "$lib/data/difficulty/tocfl.tsv?raw";
import rawJlpt from "$lib/data/difficulty/jlpt.tsv?raw";
import rawTopik from "$lib/data/difficulty/topik.tsv?raw";

export const CEFR_LANG_MAP: Record<string, { label: string; content: string; filename: string }> = {
  en: { label: "English", content: rawCefrEn, filename: "cefr-english-vocabulary.tsv" },
  it: { label: "Italiano", content: rawCefrIt, filename: "cefr-italian-vocabulary.tsv" },
  es: { label: "Español", content: rawCefrEs, filename: "cefr-spanish-vocabulary.tsv" },
  fr: { label: "Français", content: rawCefrFr, filename: "cefr-french-vocabulary.tsv" },
  de: { label: "Deutsch", content: rawCefrDe, filename: "cefr-german-vocabulary.tsv" },
  ru: { label: "Русский", content: rawCefrRu, filename: "cefr-russian-vocabulary.tsv" },
  pt: { label: "Português", content: rawCefrPt, filename: "cefr-portuguese-vocabulary.tsv" },
};

export const CEFR_TEMPLATE_TSV = rawCefrEn;
export const HSK_TEMPLATE_TSV = rawHsk;
export const TOCFL_TEMPLATE_TSV = rawTocfl;
export const JLPT_TEMPLATE_TSV = rawJlpt;
export const TOPIK_TEMPLATE_TSV = rawTopik;

export const CUSTOM_TEMPLATE_TSV = `# Custom User Vocabulary Level Database
# Format: word<TAB>level_number or word,level_number
# Example:
apple	1
book	1
car	1
computer	2
algorithm	3
architecture	4
ephemeral	5
`;

export const SCHEMES: SchemeInfo[] = [
  {
    id: "cefr",
    name: "CEFR (A1 - C2)",
    langCoverage: "English, Italiano, Español, Français, Deutsch, Русский, Português",
    levels: "A1 (1) · A2 (2) · B1 (3) · B2 (4) · C1 (5) · C2 (6)",
    description: "Common European Framework of Reference for Languages",
    templateContent: CEFR_TEMPLATE_TSV,
    defaultFilename: "cefr-english-vocabulary.tsv",
  },
  {
    id: "hsk",
    name: "HSK (1 - 6)",
    langCoverage: "Chinese (Mandarin / Simplified & Traditional)",
    levels: "HSK 1 (1) · HSK 2 (2) · HSK 3 (3) · HSK 4 (4) · HSK 5 (5) · HSK 6 (6)",
    description: "Hànyǔ Shuǐpíng Kǎoshì (Chinese Proficiency Test - 汉语水平考试)",
    templateContent: HSK_TEMPLATE_TSV,
    defaultFilename: "hsk-vocabulary-database.tsv",
  },
  {
    id: "tocfl",
    name: "TOCFL (1 - 6)",
    langCoverage: "Chinese Traditional (Taiwan / Hong Kong / 繁體中文)",
    levels: "Novice/A1 (1) · A2 (2) · B1 (3) · B2 (4) · C1 (5) · C2 (6)",
    description: "Test of Chinese as a Foreign Language (華語文能力測驗)",
    templateContent: TOCFL_TEMPLATE_TSV,
    defaultFilename: "tocfl-vocabulary-database.tsv",
  },
  {
    id: "jlpt",
    name: "JLPT (N5 - N1)",
    langCoverage: "Japanese (日本語)",
    levels: "N5 (1) · N4 (2) · N3 (3) · N2 (4) · N1 (5)",
    description: "Japanese-Language Proficiency Test (JLPT - 日本語能力試験 / Nihongo Nōryoku Shiken)",
    templateContent: JLPT_TEMPLATE_TSV,
    defaultFilename: "jlpt-vocabulary-database.tsv",
  },
  {
    id: "topik",
    name: "TOPIK (1 - 6)",
    langCoverage: "Korean (한국어)",
    levels: "Level 1 (1) · Level 2 (2) · Level 3 (3) · Level 4 (4) · Level 5 (5) · Level 6 (6)",
    description: "Test of Proficiency in Korean (TOPIK - 한국어능력시험)",
    templateContent: TOPIK_TEMPLATE_TSV,
    defaultFilename: "topik-vocabulary-database.tsv",
  },
];

export async function exportSchemeTsv(schemeId: string, langCode?: string) {
  let content = CUSTOM_TEMPLATE_TSV;
  let filename = "custom-vocabulary-template.tsv";

  if (schemeId === "cefr" && langCode && CEFR_LANG_MAP[langCode]) {
    content = CEFR_LANG_MAP[langCode].content;
    filename = CEFR_LANG_MAP[langCode].filename;
  } else {
    const found = SCHEMES.find((s) => s.id === schemeId);
    if (found) {
      content = found.templateContent;
      filename = found.defaultFilename;
    }
  }

  try {
    const savePath = await guardedSave({
      defaultPath: filename,
      filters: [
        { name: "TSV / Tab-Separated Values", extensions: ["tsv", "txt"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });

    if (savePath) {
      await writeTextFile(savePath, content);
      snackbar.show(t("settings.difficulty.exportSuccess") || `Exported ${filename} successfully`, "success", 2500);
      return true;
    }
  } catch (err: any) {
    // Fallback for browser download
    try {
      const blob = new Blob([content], { type: "text/tab-separated-values;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      a.click();
      URL.revokeObjectURL(url);
      snackbar.show(t("settings.difficulty.exportSuccess") || `Exported ${filename} successfully`, "success", 2500);
      return true;
    } catch (e: any) {
      snackbar.show(`Export error: ${err?.message || err}`, "error", 3500);
    }
  }
  return false;
}

export function inferSchemeForLanguage(language: string): string {
  if (!language) return "cefr_en";
  const full = language.toLowerCase().trim();
  const primary = full.split(/[-_]/)[0];

  if (full.includes("tw") || full.includes("hk") || full.includes("hant") || full.includes("traditional") || primary === "tocfl") {
    return "tocfl";
  }
  if (primary === "zh" || primary === "cmn" || primary === "chinese" || primary === "zho" || primary === "chi") {
    return "hsk";
  }
  if (primary === "ja" || primary === "jpn" || primary === "japanese" || primary === "jap") {
    return "jlpt";
  }
  if (primary === "ko" || primary === "kor" || primary === "korean") {
    return "topik";
  }
  if (primary === "it" || primary === "ita" || primary === "italian") {
    return "cefr_it";
  }
  if (primary === "es" || primary === "spa" || primary === "spanish") {
    return "cefr_es";
  }
  if (primary === "fr" || primary === "fra" || primary === "fre" || primary === "french") {
    return "cefr_fr";
  }
  if (primary === "de" || primary === "deu" || primary === "ger" || primary === "german") {
    return "cefr_de";
  }
  if (primary === "ru" || primary === "rus" || primary === "russian") {
    return "cefr_ru";
  }
  if (primary === "pt" || primary === "por" || primary === "portuguese") {
    return "cefr_pt";
  }
  if (primary === "en" || primary === "eng" || primary === "english") {
    return "cefr_en";
  }
  return "cefr_en";
}
