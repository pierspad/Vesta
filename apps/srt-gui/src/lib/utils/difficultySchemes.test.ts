import { describe, it, expect } from "vitest";
import {
  SCHEMES,
  CEFR_TEMPLATE_TSV,
  HSK_TEMPLATE_TSV,
  JLPT_TEMPLATE_TSV,
  CUSTOM_TEMPLATE_TSV,
  inferSchemeForLanguage,
} from "./difficultySchemes";

describe("difficultySchemes", () => {
  describe("inferSchemeForLanguage", () => {
    it("infers HSK for Simplified Chinese and TOCFL for Traditional Chinese", () => {
      expect(inferSchemeForLanguage("zh")).toBe("hsk");
      expect(inferSchemeForLanguage("zh-CN")).toBe("hsk");
      expect(inferSchemeForLanguage("zh-TW")).toBe("tocfl");
      expect(inferSchemeForLanguage("zh-HK")).toBe("tocfl");
      expect(inferSchemeForLanguage("cmn")).toBe("hsk");
      expect(inferSchemeForLanguage("chinese")).toBe("hsk");
      expect(inferSchemeForLanguage("ZH_HANS")).toBe("hsk");
      expect(inferSchemeForLanguage("ZH_HANT")).toBe("tocfl");
    });

    it("infers JLPT for Japanese", () => {
      expect(inferSchemeForLanguage("ja")).toBe("jlpt");
      expect(inferSchemeForLanguage("ja-JP")).toBe("jlpt");
      expect(inferSchemeForLanguage("jpn")).toBe("jlpt");
      expect(inferSchemeForLanguage("japanese")).toBe("jlpt");
    });

    it("infers TOPIK for Korean", () => {
      expect(inferSchemeForLanguage("ko")).toBe("topik");
      expect(inferSchemeForLanguage("ko-KR")).toBe("topik");
      expect(inferSchemeForLanguage("kor")).toBe("topik");
      expect(inferSchemeForLanguage("korean")).toBe("topik");
    });

    it("infers language-specific CEFR for European and other languages", () => {
      expect(inferSchemeForLanguage("en")).toBe("cefr_en");
      expect(inferSchemeForLanguage("it")).toBe("cefr_it");
      expect(inferSchemeForLanguage("es")).toBe("cefr_es");
      expect(inferSchemeForLanguage("fr")).toBe("cefr_fr");
      expect(inferSchemeForLanguage("de")).toBe("cefr_de");
      expect(inferSchemeForLanguage("ru")).toBe("cefr_ru");
      expect(inferSchemeForLanguage("pt")).toBe("cefr_pt");
      expect(inferSchemeForLanguage("ar")).toBe("cefr_en");
      expect(inferSchemeForLanguage("")).toBe("cefr_en");
      expect(inferSchemeForLanguage(undefined as unknown as string)).toBe("cefr_en");
    });
  });

  describe("TSV Templates structure & formatting", () => {
    const templates = [
      { name: "CEFR", tsv: CEFR_TEMPLATE_TSV, minLevel: 1, maxLevel: 6 },
      { name: "HSK", tsv: HSK_TEMPLATE_TSV, minLevel: 1, maxLevel: 6 },
      { name: "JLPT", tsv: JLPT_TEMPLATE_TSV, minLevel: 1, maxLevel: 5 },
      { name: "Custom", tsv: CUSTOM_TEMPLATE_TSV, minLevel: 1, maxLevel: 5 },
    ];

    for (const { name, tsv, minLevel, maxLevel } of templates) {
      it(`validates ${name} template has tab-separated format and correct levels`, () => {
        const lines = tsv.trim().split("\n");
        let dataRowCount = 0;
        const errors: string[] = [];

        for (let i = 0; i < lines.length; i++) {
          const trimmed = lines[i].trim();
          if (!trimmed || trimmed.startsWith("#")) continue;

          dataRowCount++;
          const parts = trimmed.split("\t");
          if (parts.length !== 2) {
            errors.push(`Line ${i + 1}: expected 2 tab-separated columns, got ${parts.length}`);
            continue;
          }

          const [word, levelStr] = parts;
          if (word.length === 0) {
            errors.push(`Line ${i + 1}: empty word`);
          }

          const level = Number.parseInt(levelStr, 10);
          if (!Number.isFinite(level) || level < minLevel || level > maxLevel) {
            errors.push(`Line ${i + 1}: invalid level '${levelStr}' (expected ${minLevel}-${maxLevel})`);
          }
        }

        expect(errors).toEqual([]);
        expect(dataRowCount).toBeGreaterThan(0);
      });
    }
  });

  describe("SCHEMES catalog integrity", () => {
    it("contains all standard built-in schemes", () => {
      const ids = SCHEMES.map((s) => s.id);
      expect(ids).toEqual(["cefr", "hsk", "tocfl", "jlpt", "topik"]);
    });

    it("every scheme has a valid defaultFilename with .tsv extension", () => {
      for (const scheme of SCHEMES) {
        expect(scheme.defaultFilename.endsWith(".tsv")).toBe(true);
        expect(scheme.name.length).toBeGreaterThan(0);
        expect(scheme.description.length).toBeGreaterThan(0);
      }
    });
  });
});
