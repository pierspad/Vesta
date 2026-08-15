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

export const CEFR_TEMPLATE_TSV = `# CEFR Vocabulary Level Database (Common European Framework of Reference)
# Format: word<TAB>level_number (1=A1, 2=A2, 3=B1, 4=B2, 5=C1, 6=C2)
# Lines starting with # are ignored.
the	1
a	1
an	1
is	1
are	1
was	1
were	1
have	1
has	1
had	1
do	1
does	1
did	1
say	1
said	1
go	1
good	1
day	1
man	1
woman	1
child	1
book	1
friend	1
hello	1
house	1
water	1
food	1
work	1
time	1
year	1
journey	2
arrive	2
departure	2
receive	2
explain	2
decision	2
future	2
possible	2
analyze	3
analyse	3
hypothesis	4
paradigm	5
ubiquitous	6
`;

export const HSK_TEMPLATE_TSV = `# HSK Vocabulary Level Database (Chinese Proficiency Test)
# Format: word<TAB>level_number (1=HSK1, 2=HSK2, 3=HSK3, 4=HSK4, 5=HSK5, 6=HSK6)
# Lines starting with # are ignored.
你	1
好	1
我	1
是	1
中国	1
人	1
谢谢	1
不	1
喜欢	1
水	1
吃	1
喝	1
看	1
听	1
去	1
来	1
想	1
小	1
大	1
好	1
学习	2
帮助	2
简单	2
问题	2
工作	2
开始	2
因为	2
所以	2
但是	2
已经	2
经常	3
环境	3
解决	3
提高	3
选择	3
几乎	3
关系	3
影响	3
偶尔	4
频繁	4
判断	4
趋势	4
极其	4
抽象	5
震撼	5
涵盖	5
`;

export const JLPT_TEMPLATE_TSV = `# JLPT Vocabulary Level Database (Japanese Language Proficiency Test)
# Format: word<TAB>level_number (1=N5, 2=N4, 3=N3, 4=N2, 5=N1)
# Lines starting with # are ignored.
私	1
本	1
人	1
日本	1
食べる	1
飲む	1
行く	1
来る	1
見る	1
聞く	1
大きい	1
小さい	1
勉強	2
簡単	2
問題	2
仕事	2
始まる	2
選ぶ	3
環境	3
関係	3
影响	3
複雑	4
概念	4
判断	4
顕著	5
`;

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
    langCoverage: "English, Italian, Spanish, French, German, Portuguese, Russian, Dutch, Polish, etc.",
    levels: "A1 (1) · A2 (2) · B1 (3) · B2 (4) · C1 (5) · C2 (6)",
    description: "Common European Framework of Reference for Languages based on frequency and lexical complexity.",
    templateContent: CEFR_TEMPLATE_TSV,
    defaultFilename: "cefr-vocabulary-template.tsv",
  },
  {
    id: "hsk",
    name: "HSK (1 - 6)",
    langCoverage: "Chinese (Mandarin / Simplified & Traditional)",
    levels: "HSK 1 (1) · HSK 2 (2) · HSK 3 (3) · HSK 4 (4) · HSK 5 (5) · HSK 6 (6)",
    description: "Hànyǔ Shuǐpíng Kǎoshì (Chinese Proficiency Test - 汉语水平考试) official standard vocabulary lists.",
    templateContent: HSK_TEMPLATE_TSV,
    defaultFilename: "hsk-vocabulary-template.tsv",
  },
  {
    id: "jlpt",
    name: "JLPT (N5 - N1)",
    langCoverage: "Japanese",
    levels: "N5 (1) · N4 (2) · N3 (3) · N2 (4) · N1 (5)",
    description: "Japanese-Language Proficiency Test (JLPT - 日本語能力試験 / Nihongo Nōryoku Shiken) official standard vocabulary lists.",
    templateContent: JLPT_TEMPLATE_TSV,
    defaultFilename: "jlpt-vocabulary-template.tsv",
  },
];

export async function exportSchemeTsv(schemeId: string) {
  let content = CUSTOM_TEMPLATE_TSV;
  let filename = "custom-vocabulary-template.tsv";

  const found = SCHEMES.find((s) => s.id === schemeId);
  if (found) {
    content = found.templateContent;
    filename = found.defaultFilename;
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
  if (!language) return "cefr";
  const primary = language.toLowerCase().split(/[-_]/)[0];
  if (primary === "zh" || primary === "cmn" || primary === "chinese") {
    return "hsk";
  }
  if (primary === "ja" || primary === "jpn" || primary === "japanese") {
    return "jlpt";
  }
  return "cefr";
}
