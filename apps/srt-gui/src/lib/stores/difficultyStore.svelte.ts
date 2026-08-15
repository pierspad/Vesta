import * as vestaConfig from "$lib/config/vestaConfig";

export interface CustomScheme {
  id: string;
  name: string;
  filePath: string;
  tagPrefix?: string;
}

const EXPERIMENTAL_DIFFICULTY_TAGGING_KEY = "vesta-experimental-difficulty-tagging";
const CUSTOM_SCHEMES_STORAGE_KEY = "vesta-custom-difficulty-schemes";

const RESERVED_NAMES = ["cefr", "hsk", "jlpt", "custom"];

class DifficultyStore {
  enabled = $state(false);
  customSchemes = $state<CustomScheme[]>([]);

  constructor() {
    this.enabled = vestaConfig.getItem(EXPERIMENTAL_DIFFICULTY_TAGGING_KEY) === "true";
    this.loadCustomSchemes();
  }

  private loadCustomSchemes() {
    try {
      const raw = vestaConfig.getItem(CUSTOM_SCHEMES_STORAGE_KEY);
      if (raw) {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) {
          this.customSchemes = parsed.filter(
            (s) => s && typeof s.id === "string" && typeof s.name === "string" && typeof s.filePath === "string"
          );
          return;
        }
      }
    } catch (e) {
      console.error("[DifficultyStore] Failed to load custom difficulty schemes", e);
    }
    this.customSchemes = [];
  }

  private saveCustomSchemes() {
    try {
      vestaConfig.setItem(CUSTOM_SCHEMES_STORAGE_KEY, JSON.stringify(this.customSchemes));
    } catch (e) {
      console.error("[DifficultyStore] Failed to save custom difficulty schemes", e);
    }
  }

  toggleFeature(): boolean {
    this.enabled = !this.enabled;
    vestaConfig.setItem(EXPERIMENTAL_DIFFICULTY_TAGGING_KEY, String(this.enabled));
    return this.enabled;
  }

  setFeatureEnabled(val: boolean) {
    this.enabled = val;
    vestaConfig.setItem(EXPERIMENTAL_DIFFICULTY_TAGGING_KEY, String(this.enabled));
  }

  isNameAvailable(name: string, excludeId?: string): boolean {
    const trimmed = name.trim().toLowerCase();
    if (!trimmed) return false;
    if (RESERVED_NAMES.includes(trimmed)) return false;
    return !this.customSchemes.some(
      (s) => s.id !== excludeId && s.name.trim().toLowerCase() === trimmed
    );
  }

  addCustomScheme(data: { name: string; filePath: string; tagPrefix?: string }): { success: boolean; scheme?: CustomScheme; error?: string } {
    const trimmedName = data.name.trim();
    const trimmedPath = data.filePath.trim();
    const trimmedPrefix = data.tagPrefix?.trim();

    if (!trimmedName) {
      return { success: false, error: "Name cannot be empty" };
    }
    if (!trimmedPath) {
      return { success: false, error: "File path cannot be empty" };
    }
    if (!this.isNameAvailable(trimmedName)) {
      return { success: false, error: "Scheme name already exists or is reserved" };
    }

    const newScheme: CustomScheme = {
      id: "custom_" + Date.now().toString(36) + "_" + Math.random().toString(36).substring(2, 7),
      name: trimmedName,
      filePath: trimmedPath,
      tagPrefix: trimmedPrefix || trimmedName,
    };

    this.customSchemes = [...this.customSchemes, newScheme];
    this.saveCustomSchemes();
    return { success: true, scheme: newScheme };
  }

  removeCustomScheme(id: string): boolean {
    const prevLen = this.customSchemes.length;
    this.customSchemes = this.customSchemes.filter((s) => s.id !== id);
    if (this.customSchemes.length !== prevLen) {
      this.saveCustomSchemes();
      return true;
    }
    return false;
  }

  getSchemeById(id: string): CustomScheme | undefined {
    return this.customSchemes.find((s) => s.id === id);
  }
}

export const difficultyStore = new DifficultyStore();
