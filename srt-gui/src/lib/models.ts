/**
 * Definizione modelli AI disponibili per la traduzione
 * 
 * ARCHITETTURA SEMPLIFICATA:
 * - Local: per Ollama, LM Studio e altri server locali (API OpenAI-compatible)
 * - OpenRouter: gateway unificato per tutti i modelli cloud (Gemini, GPT, Claude, Mistral, etc.)
 * 
 * Tutti usano lo stesso formato API (OpenAI-compatible), semplificando enormemente il backend.
 */

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  family: string; // New field for grouping
  contextWindow?: number;
  description?: string;
  recommended?: boolean;
}

export interface ProviderInfo {
  id: string;
  name: string;
  icon: string;
  color: string;
  description: string;
  requiresApiKey: boolean;
  requiresApiUrl: boolean;
  defaultApiUrl?: string;
}

// Solo 2 provider: Local e OpenRouter
export const providers: Record<string, ProviderInfo> = {
  local: {
    id: "local",
    name: "Local LLM",
    icon: "🏠",
    color: "from-purple-500 to-pink-500",
    description: "Ollama, LM Studio, modelli locali",
    requiresApiKey: false,
    requiresApiUrl: true,
    defaultApiUrl: "http://localhost:11434/v1",
  },
  openrouter: {
    id: "openrouter",
    name: "OpenRouter",
    icon: "🌐",
    color: "from-indigo-500 to-purple-600",
    description: "Accesso unificato a GPT, Claude, Gemini, Mistral e altri",
    requiresApiKey: true,
    requiresApiUrl: false,
    defaultApiUrl: "https://openrouter.ai/api/v1",
  },
};

// Solo 2 provider
export const providerOrder = ["local", "openrouter"];

// Modelli per provider
export const modelsByProvider: Record<string, ModelInfo[]> = {
  local: [
    {
      id: "llama3.2",
      name: "Llama 3.2",
      provider: "local",
      family: "Meta",
      description: "Meta open-weight latest local model",
      recommended: true,
    },
    {
      id: "llama3.1",
      name: "Llama 3.1",
      provider: "local",
      family: "Meta",
      description: "Versione precedente utile per traduzioni",
    },
    {
      id: "mistral-small-3.1",
      name: "Mistral Small 3.1",
      provider: "local",
      family: "Mistral",
      description: "Mistral Small 3.1 locale",
    },
    {
      id: "mistral-medium-3",
      name: "Mistral Medium 3",
      provider: "local",
      family: "Mistral",
      description: "Mistral Medium 3 locale",
    },
    {
      id: "mixtral-8x7b",
      name: "Mixtral 8x7B",
      provider: "local",
      family: "Mistral",
      description: "Mixtral MoE per uso locale",
    },
    {
      id: "qwen2.5",
      name: "Qwen 2.5",
      provider: "local",
      family: "Qwen",
      description: "Ottimo per multilingue locale",
    },
    {
      id: "gemma3-27b-local",
      name: "Gemma 3 27B Local",
      provider: "local",
      family: "Google",
      description: "Gemma 3 27B (open)",
    },
    {
      id: "phi3",
      name: "Phi-3",
      provider: "local",
      family: "Microsoft",
      description: "Microsoft compatto locale",
    },
  ],

  openrouter: [
    // 🅶 Google Gemini 3 family
    {
      id: "google/gemini-3-pro-preview-20251117",
      name: "🔷 Gemini 3 Pro Preview",
      provider: "openrouter",
      family: "Google",
      contextWindow: 1048576,
      description: "Google Gemini 3 Pro (Preview), capacità top",
      recommended: true,
    },
    {
      id: "google/gemini-3-flash-preview-20251217",
      name: "🔷 Gemini 3 Flash Preview",
      provider: "openrouter",
      family: "Google",
      contextWindow: 1048576,
      description: "Google Gemini 3 Flash (Preview), rapido e potente",
    },

    // 🅶 Google Gemini 2.5 family (ancora utili)
    {
      id: "google/gemini-2.5-flash",
      name: "🔷 Gemini 2.5 Flash",
      provider: "openrouter",
      family: "Google",
      contextWindow: 1048576,
      description: "Google - versione efficiente e veloce",
    },
    {
      id: "google/gemini-2.5-pro",
      name: "🔷 Gemini 2.5 Pro",
      provider: "openrouter",
      family: "Google",
      contextWindow: 1048576,
      description: "Google - versione Pro con reasoning migliorato",
    },

    // 🅶 Gemma 3 family (open)
    {
      id: "google/gemma-3-12b-it",
      name: "🔶 Gemma 3 12B",
      provider: "openrouter",
      family: "Google",
      contextWindow: 128000,
      description: "Gemma 3 12B open model",
    },
    {
      id: "google/gemma-3-27b-it",
      name: "🔶 Gemma 3 27B",
      provider: "openrouter",
      family: "Google",
      contextWindow: 128000,
      description: "Gemma 3 27B open model",
    },
    {
      id: "google/gemma-3-4b-it",
      name: "🔶 Gemma 3 4B",
      provider: "openrouter",
      family: "Google",
      contextWindow: 128000,
      description: "Gemma 3 4B open model",
    },
    {
      id: "google/gemma-3n-e4b-it",
      name: "🔶 Gemma 3n E4B",
      provider: "openrouter",
      family: "Google",
      contextWindow: 8192,
      description: "Gemma 3n E4B, efficienza ottimizzata",
    },
    {
      id: "google/gemma-3n-e2b-it",
      name: "🔶 Gemma 3n E2B",
      provider: "openrouter",
      family: "Google",
      contextWindow: 8192,
      description: "Gemma 3n E2B, efficienza massima",
    },

    // 🅾️ OpenAI GPT
    {
      id: "openai/gpt-4o",
      name: "🟢 GPT-4o",
      provider: "openrouter",
      family: "OpenAI",
      contextWindow: 128000,
      description: "OpenAI - flagship multimodale",
    },
    {
      id: "openai/gpt-4o-mini",
      name: "🟢 GPT-4o Mini",
      provider: "openrouter",
      family: "OpenAI",
      contextWindow: 128000,
      description: "OpenAI - versione economica",
    },
    {
      id: "openai/gpt-5.1",
      name: "🟢 GPT-5.1",
      provider: "openrouter",
      family: "OpenAI",
      contextWindow: 128000,
      description: "OpenAI - mod. avanzato (se disponibile)",
    },

    // 🅰️ Anthropic Claude (aggiornati)
    {
      id: "anthropic/claude-4.5-opus-20251124",
      name: "🟠 Claude 4.5 Opus",
      provider: "openrouter",
      family: "Anthropic",
      contextWindow: 200000,
      description: "Anthropic Claude 4.5 Opus",
    },
    {
      id: "anthropic/claude-4.5-sonnet-20250929",
      name: "🟠 Claude 4.5 Sonnet",
      provider: "openrouter",
      family: "Anthropic",
      contextWindow: 200000,
      description: "Anthropic Claude 4.5 Sonnet",
    },

    // 🟡 Mistral
    {
      id: "z-ai/glm-4.5",
      name: "🟡 GLM 4.5",
      provider: "openrouter",
      family: "GLM",
      contextWindow: 128000,
      description: "GLM 4.5 full",
    },
    {
      id: "z-ai/glm-4.5-air",
      name: "🟡 GLM 4.5 Air",
      provider: "openrouter",
      family: "GLM",
      contextWindow: 128000,
      description: "GLM 4.5 Air versione economica",
    },

    // 🌀 Altri
    {
      id: "deepseek/deepseek-chat",
      name: "🔮 DeepSeek Chat",
      provider: "openrouter",
      family: "DeepSeek",
      contextWindow: 64000,
      description: "DeepSeek - buona qualità/rapporto prezzo",
    },
    {
      id: "qwen/qwen-2.5-72b-instruct",
      name: "🌙 Qwen 2.5 72B",
      provider: "openrouter",
      family: "Qwen",
      contextWindow: 128000,
      description: "Qwen 2.5 72B - eccellente multilingue",
    },
  ],
};

// Modelli personalizzati (salvati in localStorage)
export interface CustomModel {
  id: string;
  name: string;
  provider: string;
  apiModelId: string; // ID usato nelle chiamate API
  description?: string;
}

export interface ApiKeyConfig {
  id: string;
  name: string;
  apiType: "local" | "openrouter";
  apiKey: string;
  apiUrl?: string;
  modelName?: string;  // Nome modello preferito
  isDefault: boolean;
}

// Funzione per ottenere tutti i modelli di un provider (inclusi custom)
export function getModelsForProvider(providerId: string): ModelInfo[] {
  const defaultModels = modelsByProvider[providerId] || [];

  // Carica modelli personalizzati
  const customModelsJson = localStorage.getItem("srt-tools-custom-models");
  if (customModelsJson) {
    try {
      const customModels: CustomModel[] = JSON.parse(customModelsJson);
      const providerCustomModels = customModels
        .filter((m) => m.provider === providerId)
        .map((m) => ({
          id: m.apiModelId,
          name: m.name,
          provider: m.provider,
          family: "Custom", // Default family for custom models
          description: m.description || "Modello personalizzato",
        }));
      return [...defaultModels, ...providerCustomModels];
    } catch {
      return defaultModels;
    }
  }

  return defaultModels;
}

// Funzione per salvare un modello personalizzato
export function saveCustomModel(model: CustomModel): void {
  const customModelsJson = localStorage.getItem("srt-tools-custom-models");
  let customModels: CustomModel[] = [];

  if (customModelsJson) {
    try {
      customModels = JSON.parse(customModelsJson);
    } catch {
      customModels = [];
    }
  }

  // Controlla se esiste già
  const existingIndex = customModels.findIndex((m) => m.id === model.id);
  if (existingIndex >= 0) {
    customModels[existingIndex] = model;
  } else {
    customModels.push(model);
  }

  localStorage.setItem("srt-tools-custom-models", JSON.stringify(customModels));
}

// Funzione per eliminare un modello personalizzato
export function deleteCustomModel(modelId: string): void {
  const customModelsJson = localStorage.getItem("srt-tools-custom-models");
  if (!customModelsJson) return;

  try {
    let customModels: CustomModel[] = JSON.parse(customModelsJson);
    customModels = customModels.filter((m) => m.id !== modelId);
    localStorage.setItem("srt-tools-custom-models", JSON.stringify(customModels));
  } catch {
    // Ignora errori
  }
}

// Funzione per ottenere tutti i modelli personalizzati
export function getCustomModels(): CustomModel[] {
  const customModelsJson = localStorage.getItem("srt-tools-custom-models");
  if (!customModelsJson) return [];

  try {
    return JSON.parse(customModelsJson);
  } catch {
    return [];
  }
}

// Lista lingue disponibili per la traduzione
export const languages = [
  { code: "it", name: "Italiano", flag: "🇮🇹" },
  { code: "en", name: "English", flag: "🇬🇧" },
  { code: "es", name: "Español", flag: "🇪🇸" },
  { code: "fr", name: "Français", flag: "🇫🇷" },
  { code: "de", name: "Deutsch", flag: "🇩🇪" },
  { code: "pt", name: "Português", flag: "🇵🇹" },
  { code: "pt-br", name: "Português (Brasil)", flag: "🇧🇷" },
  { code: "ja", name: "日本語", flag: "🇯🇵" },
  { code: "ko", name: "한국어", flag: "🇰🇷" },
  { code: "zh", name: "中文 (简体)", flag: "🇨🇳" },
  { code: "zh-tw", name: "中文 (繁體)", flag: "🇹🇼" },
  { code: "ru", name: "Русский", flag: "🇷🇺" },
  { code: "ar", name: "العربية", flag: "🇸🇦" },
  { code: "hi", name: "हिंदी", flag: "🇮🇳" },
  { code: "tr", name: "Türkçe", flag: "🇹🇷" },
  { code: "pl", name: "Polski", flag: "🇵🇱" },
  { code: "nl", name: "Nederlands", flag: "🇳🇱" },
  { code: "sv", name: "Svenska", flag: "🇸🇪" },
  { code: "da", name: "Dansk", flag: "🇩🇰" },
  { code: "no", name: "Norsk", flag: "🇳🇴" },
  { code: "fi", name: "Suomi", flag: "🇫🇮" },
  { code: "cs", name: "Čeština", flag: "🇨🇿" },
  { code: "el", name: "Ελληνικά", flag: "🇬🇷" },
  { code: "he", name: "עברית", flag: "🇮🇱" },
  { code: "th", name: "ไทย", flag: "🇹🇭" },
  { code: "vi", name: "Tiếng Việt", flag: "🇻🇳" },
  { code: "id", name: "Bahasa Indonesia", flag: "🇮🇩" },
  { code: "ms", name: "Bahasa Melayu", flag: "🇲🇾" },
  { code: "uk", name: "Українська", flag: "🇺🇦" },
  { code: "ro", name: "Română", flag: "🇷🇴" },
  { code: "hu", name: "Magyar", flag: "🇭🇺" },
];

// Shortcut predefinite
export interface ShortcutDefinition {
  id: string;
  action: string;
  description: string;
  defaultKey: string;
  category: "global" | "translate" | "sync" | "settings";
}

export const defaultShortcuts: ShortcutDefinition[] = [
  // Globali (include anche Aggiungi chiave API)
  { id: "tab-translate", action: "switchToTranslate", description: "Vai a Traduzione", defaultKey: "Alt+1", category: "global" },
  { id: "tab-sync", action: "switchToSync", description: "Vai a Sincronizzazione", defaultKey: "Alt+2", category: "global" },
  { id: "tab-settings", action: "switchToSettings", description: "Vai a Impostazioni", defaultKey: "Alt+3", category: "global" },
  { id: "tab-shortcuts", action: "switchToShortcuts", description: "Vai a Shortcuts", defaultKey: "Alt+4", category: "global" },
  { id: "settings-add-key", action: "addApiKey", description: "Aggiungi chiave API", defaultKey: "Ctrl+N", category: "global" },

  // Traduzione
  { id: "translate-open-file", action: "openInputFile", description: "Apri file SRT", defaultKey: "Ctrl+O", category: "translate" },
  { id: "translate-start", action: "startTranslation", description: "Avvia traduzione", defaultKey: "Ctrl+Enter", category: "translate" },
  { id: "translate-cancel", action: "cancelTranslation", description: "Annulla traduzione", defaultKey: "Escape", category: "translate" },
  { id: "translate-clear-logs", action: "clearLogs", description: "Cancella log", defaultKey: "Ctrl+L", category: "translate" },

  // Sincronizzazione (divise in 2 gruppi nella visualizzazione)
  { id: "sync-play-pause", action: "playPause", description: "Play/Pausa video", defaultKey: "Space", category: "sync" },
  { id: "sync-seek-back", action: "seekBack", description: "Indietro 0.1s", defaultKey: "ArrowLeft", category: "sync" },
  { id: "sync-seek-forward", action: "seekForward", description: "Avanti 0.1s", defaultKey: "ArrowRight", category: "sync" },
  { id: "sync-seek-back-fast", action: "seekBackFast", description: "Indietro 1s", defaultKey: "Shift+ArrowLeft", category: "sync" },
  { id: "sync-seek-forward-fast", action: "seekForwardFast", description: "Avanti 1s", defaultKey: "Shift+ArrowRight", category: "sync" },
  { id: "sync-offset-up", action: "offsetUp", description: "Offset +100ms", defaultKey: "ArrowUp", category: "sync" },
  { id: "sync-offset-down", action: "offsetDown", description: "Offset -100ms", defaultKey: "ArrowDown", category: "sync" },
  { id: "sync-offset-up-fast", action: "offsetUpFast", description: "Offset +500ms", defaultKey: "Shift+ArrowUp", category: "sync" },
  { id: "sync-offset-down-fast", action: "offsetDownFast", description: "Offset -500ms", defaultKey: "Shift+ArrowDown", category: "sync" },
  { id: "sync-confirm", action: "confirmAnchor", description: "Conferma ancora", defaultKey: "Enter", category: "sync" },
  { id: "sync-next-sub", action: "nextSubtitle", description: "Sottotitolo successivo", defaultKey: "Tab", category: "sync" },
  { id: "sync-prev-sub", action: "prevSubtitle", description: "Sottotitolo precedente", defaultKey: "Shift+Tab", category: "sync" },
  { id: "sync-prev-anchor", action: "prevAnchor", description: "Ancora precedente", defaultKey: "Ctrl+ArrowUp", category: "sync" },
  { id: "sync-next-anchor", action: "nextAnchor", description: "Ancora successiva", defaultKey: "Ctrl+ArrowDown", category: "sync" },
  { id: "sync-go-suggested", action: "goToSuggested", description: "Vai al suggerito", defaultKey: "Ctrl+G", category: "sync" },
  { id: "sync-save", action: "saveFile", description: "Salva SRT", defaultKey: "Ctrl+S", category: "sync" },
];

// Funzione per ottenere le shortcut (con override utente)
export function getShortcuts(): ShortcutDefinition[] {
  const overridesJson = localStorage.getItem("srt-tools-shortcut-overrides");
  if (!overridesJson) return defaultShortcuts;

  try {
    const overrides: Record<string, string> = JSON.parse(overridesJson);
    return defaultShortcuts.map((shortcut) => ({
      ...shortcut,
      defaultKey: overrides[shortcut.id] || shortcut.defaultKey,
    }));
  } catch {
    return defaultShortcuts;
  }
}

// Funzione per salvare override shortcut
export function saveShortcutOverride(shortcutId: string, newKey: string): void {
  const overridesJson = localStorage.getItem("srt-tools-shortcut-overrides");
  let overrides: Record<string, string> = {};

  if (overridesJson) {
    try {
      overrides = JSON.parse(overridesJson);
    } catch {
      overrides = {};
    }
  }

  overrides[shortcutId] = newKey;
  localStorage.setItem("srt-tools-shortcut-overrides", JSON.stringify(overrides));
}

// Funzione per resettare le shortcut
export function resetShortcuts(): void {
  localStorage.removeItem("srt-tools-shortcut-overrides");
}

// Funzione per formattare context window
export function formatContextWindow(tokens: number): string {
  if (tokens >= 1000000) {
    return `${(tokens / 1000000).toFixed(1)}M`;
  }
  return `${(tokens / 1000).toFixed(0)}K`;
}

// Helper per caricare e validare le chiavi API
export function loadAndValidateApiKeys(): ApiKeyConfig[] {
  const saved = localStorage.getItem("srt-tools-api-keys");
  if (!saved) return [];

  try {
    const parsed = JSON.parse(saved);
    if (!Array.isArray(parsed)) return [];

    // Converti chiavi con tipi legacy a "openrouter" e filtra quelle non valide
    const converted = parsed.map((k: any) => {
      // Se già ha un tipo valido, mantienilo
      if (k.apiType === "local" || k.apiType === "openrouter") {
        return k;
      }
      // Converti tipi legacy (gemini, openai, anthropic, etc.) a openrouter
      if (k.apiType && k.apiType !== "local") {
        return {
          ...k,
          apiType: "openrouter" as const,
          // Se manca l'URL, aggiungi quello di default per OpenRouter
          apiUrl: k.apiUrl || "https://openrouter.ai/api/v1"
        };
      }
      return null;
    }).filter((k: any) => k !== null);

    return converted as ApiKeyConfig[];
  } catch {
    return [];
  }
}
