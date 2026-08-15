import { describe, it, expect, beforeEach, vi } from "vitest";

const { storage } = vi.hoisted(() => {
  return { storage: new Map<string, string>() };
});

vi.mock("$lib/config/vestaConfig", () => ({
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, val: string) => {
    storage.set(key, val);
  },
  removeItem: (key: string) => {
    storage.delete(key);
  },
}));

import { difficultyStore } from "$lib/stores/difficultyStore.svelte";

describe("difficultyStore", () => {
  beforeEach(() => {
    storage.clear();
    difficultyStore.setFeatureEnabled(false);
    while (difficultyStore.customSchemes.length > 0) {
      difficultyStore.removeCustomScheme(difficultyStore.customSchemes[0].id);
    }
  });

  it("is disabled by default and can be toggled", () => {
    expect(difficultyStore.enabled).toBe(false);
    difficultyStore.toggleFeature();
    expect(difficultyStore.enabled).toBe(true);
    expect(storage.get("vesta-experimental-difficulty-tagging")).toBe("true");

    difficultyStore.toggleFeature();
    expect(difficultyStore.enabled).toBe(false);
    expect(storage.get("vesta-experimental-difficulty-tagging")).toBe("false");
  });

  it("validates reserved and duplicate scheme names", () => {
    expect(difficultyStore.isNameAvailable("cefr")).toBe(false);
    expect(difficultyStore.isNameAvailable("HSK")).toBe(false);
    expect(difficultyStore.isNameAvailable("jlpt")).toBe(false);
    expect(difficultyStore.isNameAvailable("custom")).toBe(false);
    expect(difficultyStore.isNameAvailable("")).toBe(false);
    expect(difficultyStore.isNameAvailable("   ")).toBe(false);
    expect(difficultyStore.isNameAvailable("My Vocab")).toBe(true);
  });

  it("adds and removes custom schemes", () => {
    const res = difficultyStore.addCustomScheme({
      name: "Oxford 3000",
      filePath: "/path/to/oxford.tsv",
      tagPrefix: "Oxford",
    });

    expect(res.success).toBe(true);
    expect(res.scheme).toBeDefined();
    expect(res.scheme?.name).toBe("Oxford 3000");
    expect(res.scheme?.filePath).toBe("/path/to/oxford.tsv");
    expect(res.scheme?.tagPrefix).toBe("Oxford");
    expect(difficultyStore.customSchemes).toHaveLength(1);

    // Cannot add duplicate
    const duplicateRes = difficultyStore.addCustomScheme({
      name: "oxford 3000",
      filePath: "/path/to/other.tsv",
    });
    expect(duplicateRes.success).toBe(false);
    expect(difficultyStore.customSchemes).toHaveLength(1);

    // Can remove
    const schemeId = res.scheme!.id;
    const removed = difficultyStore.removeCustomScheme(schemeId);
    expect(removed).toBe(true);
    expect(difficultyStore.customSchemes).toHaveLength(0);
  });
});
