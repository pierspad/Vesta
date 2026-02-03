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
      description: "Meta's latest open model",
      recommended: true,
    },
    {
      id: "llama3.1",
      name: "Llama 3.1",
      provider: "local",
      family: "Meta",
      description: "Molto capace per traduzioni",
    },
    {
      id: "mistral",
      name: "Mistral 7B",
      provider: "local",
      family: "Mistral",
      description: "Efficiente e veloce",
    },
    {
      id: "mixtral",
      name: "Mixtral 8x7B",
      provider: "local",
      family: "Mistral",
      description: "MoE, alta qualità",
    },
    {
      id: "qwen2.5",
      name: "Qwen 2.5",
      provider: "local",
      family: "Qwen",
      description: "Ottimo per multilingue",
    },
    {
      id: "gemma2",
      name: "Gemma 2",
      provider: "local",
      family: "Google",
      description: "Google's open model",
    },
    {
      id: "phi3",
      name: "Phi-3",
      provider: "local",
      family: "Microsoft",
      description: "Microsoft, compatto",
    },
  ],

  openrouter: [
    // Google Gemini (via OpenRouter)
    {
      id: "google/gemini-2.0-flash-001",
      name: "🔷 Gemini 2.0 Flash",
      provider: "openrouter",
      family: "Google",
      contextWindow: 1048576,
      description: "Google - Velocissimo, ottimo per traduzioni",
      recommended: true,
    },
    {
      id: "google/gemini-1.5-flash",
      name: "🔷 Gemini 1.5 Flash",
      provider: "openrouter",
      family: "Google",
      contextWindow: 1048576,
      description: "Google - Veloce, buon rapporto qualità/prezzo",
    },
    {
      id: "google/gemini-1.5-pro",
      name: "🔷 Gemini 1.5 Pro",
      provider: "openrouter",
      family: "Google",
      contextWindow: 2097152,
      description: "Google - Più potente, migliore qualità",
    },
    // OpenAI GPT (via OpenRouter)
    {
      id: "openai/gpt-4o",
      name: "🟢 GPT-4o",
      provider: "openrouter",
      family: "OpenAI",
      contextWindow: 128000,
      description: "OpenAI - Il più potente, multimodale",
    },
    {
      id: "openai/gpt-4o-mini",
      name: "🟢 GPT-4o Mini",
      provider: "openrouter",
      family: "OpenAI",
      contextWindow: 128000,
      description: "OpenAI - Economico, buona qualità",
    },
    {
      id: "openai/gpt-4-turbo",
      name: "🟢 GPT-4 Turbo",
      provider: "openrouter",
      family: "OpenAI",
      contextWindow: 128000,
      description: "OpenAI - Potente con contesto ampio",
    },
    // Anthropic Claude (via OpenRouter)
    {
      id: "anthropic/claude-3.5-sonnet",
      name: "🟠 Claude 3.5 Sonnet",
      provider: "openrouter",
      family: "Anthropic",
      contextWindow: 200000,
      description: "Anthropic - Ottimo per traduzioni",
    },
    {
      id: "anthropic/claude-3.5-haiku",
      name: "🟠 Claude 3.5 Haiku",
      provider: "openrouter",
      family: "Anthropic",
      contextWindow: 200000,
      description: "Anthropic - Veloce ed economico",
    },
    {
      id: "anthropic/claude-3-opus",
      name: "🟠 Claude 3 Opus",
      provider: "openrouter",
      family: "Anthropic",
      contextWindow: 200000,
      description: "Anthropic - Il più potente",
    },
    // Mistral (via OpenRouter)
    {
      id: "mistralai/mistral-large-2411",
      name: "🟡 Mistral Large",
      provider: "openrouter",
      family: "Mistral",
      contextWindow: 128000,
      description: "Mistral - Il più potente, ottimo per traduzioni",
    },
    {
      id: "mistralai/mistral-medium",
      name: "🟡 Mistral Medium",
      provider: "openrouter",
      family: "Mistral",
      contextWindow: 32000,
      description: "Mistral - Bilanciato",
    },
    {
      id: "mistralai/mistral-small-3.1-24b-instruct",
      name: "🟡 Mistral Small",
      provider: "openrouter",
      family: "Mistral",
      contextWindow: 32000,
      description: "Mistral - Veloce ed economico",
    },
    // Meta Llama (via OpenRouter)
    {
      id: "meta-llama/llama-3.3-70b-instruct",
      name: "🦙 Llama 3.3 70B",
      provider: "openrouter",
      family: "Meta",
      contextWindow: 128000,
      description: "Meta - Open source, alta qualità",
    },
    {
      id: "meta-llama/llama-3.1-405b-instruct",
      name: "🦙 Llama 3.1 405B",
      provider: "openrouter",
      family: "Meta",
      contextWindow: 128000,
      description: "Meta - Il più grande open model",
    },
    // DeepSeek (via OpenRouter)
    {
      id: "deepseek/deepseek-chat",
      name: "🔮 DeepSeek Chat",
      provider: "openrouter",
      family: "DeepSeek",
      contextWindow: 64000,
      description: "DeepSeek - Ottimo rapporto qualità/prezzo",
    },
    // Qwen (via OpenRouter)
    {
      id: "qwen/qwen-2.5-72b-instruct",
      name: "🌙 Qwen 2.5 72B",
      provider: "openrouter",
      family: "Qwen",
      contextWindow: 128000,
      description: "Alibaba - Eccellente per multilingue",
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

    // Filtra chiavi con tipo non valido (es. "gemini" legacy)
    return parsed.filter(k =>
      k.apiType === "local" || k.apiType === "openrouter"
    );
  } catch {
    return [];
  }
}
