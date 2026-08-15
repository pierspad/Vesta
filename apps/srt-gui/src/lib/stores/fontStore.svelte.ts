import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { snackbar } from "$lib/stores/snackbarStore.svelte";

export interface FontStatusInfo {
  id: string;
  name: string;
  language_name: string;
  target_languages: string[];
  filename: string;
  approx_size: string;
  downloaded: boolean;
}

interface FontDownloadProgressEvent {
  font_id: string;
  percentage: number;
}

class FontStore {
  fonts = $state<FontStatusInfo[]>([]);
  downloadingFontId = $state<string | null>(null);
  downloadProgress = $state<number>(0);
  isDownloading = $derived(this.downloadingFontId !== null);

  private unlistenProgress: UnlistenFn | null = null;

  constructor() {
    this.init();
  }

  async init() {
    await this.loadFonts();
    this.setupListener();
  }

  async setupListener() {
    if (this.unlistenProgress) return;
    try {
      this.unlistenProgress = await listen<FontDownloadProgressEvent>(
        "font-download-progress",
        (event) => {
          if (this.downloadingFontId === event.payload.font_id) {
            this.downloadProgress = event.payload.percentage;
          }
        }
      );
    } catch {
      // Ignore if running in test / non-Tauri environment
    }
  }

  async loadFonts(): Promise<void> {
    try {
      this.fonts = await invoke<FontStatusInfo[]>("flashcard_list_fonts");
    } catch (err) {
      console.warn("Failed to load fonts list:", err);
    }
  }

  getFontForLanguage(lang: string): FontStatusInfo | undefined {
    if (!lang || !lang.trim()) return undefined;
    const lower = lang.trim().toLowerCase();
    if (lower === "zh-tw" || lower === "zh-hk" || lower === "zh-hant" || lower === "cht") {
      return this.fonts.find((f) => f.id === "noto-sans-tc");
    }
    const primary = lower.split(/[-_]/)[0];
    return this.fonts.find((f) => f.target_languages.includes(primary));
  }

  isFontDownloadedForLanguage(lang: string): boolean {
    const font = this.getFontForLanguage(lang);
    return font ? font.downloaded : false;
  }

  needsSpecialFont(lang: string): boolean {
    return this.getFontForLanguage(lang) !== undefined;
  }

  async downloadFont(fontId: string): Promise<boolean> {
    if (this.downloadingFontId) return false;
    this.downloadingFontId = fontId;
    this.downloadProgress = 0;

    try {
      await invoke<boolean>("flashcard_download_font", { fontId });
      await this.loadFonts();
      snackbar.show("Font scaricato con successo!", "success", 2000);
      return true;
    } catch (err) {
      console.error("Font download failed:", err);
      snackbar.show(`Errore download font: ${err}`, "error", 3000);
      return false;
    } finally {
      this.downloadingFontId = null;
      this.downloadProgress = 0;
    }
  }

  async deleteFont(fontId: string): Promise<boolean> {
    try {
      await invoke<boolean>("flashcard_delete_font", { fontId });
      await this.loadFonts();
      snackbar.show("Font rimosso dalla cache", "success", 1500);
      return true;
    } catch (err) {
      console.error("Font delete failed:", err);
      snackbar.show(`Errore rimozione font: ${err}`, "error", 2500);
      return false;
    }
  }
}

export const fontStore = new FontStore();
