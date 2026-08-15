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
    it("infers HSK for Chinese languages and dialects", () => {
      expect(inferSchemeForLanguage("zh")).toBe("hsk");
      expect(inferSchemeForLanguage("zh-CN")).toBe("hsk");
      expect(inferSchemeForLanguage("zh-TW")).toBe("hsk");
      expect(inferSchemeForLanguage("cmn")).toBe("hsk");
      expect(inferSchemeForLanguage("chinese")).toBe("hsk");
      expect(inferSchemeForLanguage("ZH_HANS")).toBe("hsk");
    });

    it("infers JLPT for Japanese", () => {
      expect(inferSchemeForLanguage("ja")).toBe("jlpt");
      expect(inferSchemeForLanguage("ja-JP")).toBe("jlpt");
      expect(inferSchemeForLanguage("jpn")).toBe("jlpt");
      expect(inferSchemeForLanguage("japanese")).toBe("jlpt");
    });

    it("defaults to CEFR for European and other languages", () => {
      expect(inferSchemeForLanguage("en")).toBe("cefr");
      expect(inferSchemeForLanguage("it")).toBe("cefr");
      expect(inferSchemeForLanguage("es")).toBe("cefr");
      expect(inferSchemeForLanguage("fr")).toBe("cefr");
      expect(inferSchemeForLanguage("de")).toBe("cefr");
      expect(inferSchemeForLanguage("ru")).toBe("cefr");
      expect(inferSchemeForLanguage("ar")).toBe("cefr");
      expect(inferSchemeForLanguage("")).toBe("cefr");
      expect(inferSchemeForLanguage(undefined as unknown as string)).toBe("cefr");
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

        for (const line of lines) {
          const trimmed = line.trim();
          if (!trimmed || trimmed.startsWith("#")) continue;

          dataRowCount++;
          const parts = trimmed.split("\t");
          expect(parts.length).toBe(2);

          const [word, levelStr] = parts;
          expect(word.length).toBeGreaterThan(0);

          const level = Number.parseInt(levelStr, 10);
          expect(Number.isFinite(level)).toBe(true);
          expect(level).toBeGreaterThanOrEqual(minLevel);
          expect(level).toBeLessThanOrEqual(maxLevel);
        }

        expect(dataRowCount).toBeGreaterThan(0);
      });
    }
  });

  describe("SCHEMES catalog integrity", () => {
    it("contains exactly the three standard built-in schemes", () => {
      const ids = SCHEMES.map((s) => s.id);
      expect(ids).toEqual(["cefr", "hsk", "jlpt"]);
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
