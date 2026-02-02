/**
 * Definizione modelli AI disponibili per la traduzione
 * 
 * Questo file contiene tutti i modelli disponibili per ogni provider.
 * È facile da mantenere: basta aggiungere o rimuovere modelli dagli array.
 * 
 * Struttura:
 * - id: identificativo API del modello (usato nelle chiamate)
 * - name: nome visualizzato nell'interfaccia
 * - provider: tipo di provider
 * - contextWindow: finestra di contesto in token (opzionale)
 * - description: descrizione breve (opzionale)
 * - recommended: modello consigliato per il provider (opzionale)
 */

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  contextWindow?: number;
  description?: string;
  recommended?: boolean;
}

export interface ProviderInfo {
  id: string;
  name: string;
  icon: string; // SVG path o emoji
  color: string; // Tailwind gradient classes
  description: string;
  requiresApiKey: boolean;
  requiresApiUrl: boolean;
  defaultApiUrl?: string;
}

// Definizione provider
export const providers: Record<string, ProviderInfo> = {
  gemini: {
    id: "gemini",
    name: "Google Gemini",
    icon: "✦",
    color: "from-blue-500 to-cyan-500",
    description: "Modelli Google AI, ottimi per traduzioni",
    requiresApiKey: true,
    requiresApiUrl: false,
  },
  openai: {
    id: "openai",
    name: "OpenAI",
    icon: "◎",
    color: "from-green-500 to-emerald-500",
    description: "GPT-4 e GPT-3.5, modelli versatili",
    requiresApiKey: true,
    requiresApiUrl: false,
  },
  anthropic: {
    id: "anthropic",
    name: "Anthropic Claude",
    icon: "◈",
    color: "from-orange-500 to-amber-500",
    description: "Claude, eccellente per testi lunghi",
    requiresApiKey: true,
    requiresApiUrl: false,
  },
  local: {
    id: "local",
    name: "Local LLM",
    icon: "⬡",
    color: "from-purple-500 to-pink-500",
    description: "Ollama, LM Studio, modelli locali",
    requiresApiKey: false,
    requiresApiUrl: true,
    defaultApiUrl: "http://localhost:11434",
  },
  openrouter: {
    id: "openrouter",
    name: "OpenRouter",
    icon: "◇",
    color: "from-red-500 to-rose-500",
    description: "Accesso unificato a molti modelli",
    requiresApiKey: true,
    requiresApiUrl: false,
  },
};

// Modelli per provider
export const modelsByProvider: Record<string, ModelInfo[]> = {
  gemini: [
    {
      id: "gemini-2.0-flash",
      name: "Gemini 2.0 Flash",
      provider: "gemini",
      contextWindow: 1048576,
      description: "Velocissimo, ottimo per traduzioni",
      recommended: true,
    },
    {
      id: "gemini-2.0-flash-lite",
      name: "Gemini 2.0 Flash Lite",
      provider: "gemini",
      contextWindow: 1048576,
      description: "Versione lite, più economico",
    },
    {
      id: "gemini-1.5-flash",
      name: "Gemini 1.5 Flash",
      provider: "gemini",
      contextWindow: 1048576,
      description: "Veloce, buon rapporto qualità/prezzo",
    },
    {
      id: "gemini-1.5-flash-8b",
      name: "Gemini 1.5 Flash 8B",
      provider: "gemini",
      contextWindow: 1048576,
      description: "Versione 8B, più leggero",
    },
    {
      id: "gemini-1.5-pro",
      name: "Gemini 1.5 Pro",
      provider: "gemini",
      contextWindow: 2097152,
      description: "Più potente, migliore qualità",
    },
  ],

  openai: [
    {
      id: "gpt-4o",
      name: "GPT-4o",
      provider: "openai",
      contextWindow: 128000,
      description: "Il più potente, multimodale",
      recommended: true,
    },
    {
      id: "gpt-4o-mini",
      name: "GPT-4o Mini",
      provider: "openai",
      contextWindow: 128000,
      description: "Economico, buona qualità",
    },
    {
      id: "gpt-4-turbo",
      name: "GPT-4 Turbo",
      provider: "openai",
      contextWindow: 128000,
      description: "Potente con contesto ampio",
    },
    {
      id: "gpt-3.5-turbo",
      name: "GPT-3.5 Turbo",
      provider: "openai",
      contextWindow: 16385,
      description: "Veloce ed economico",
    },
  ],

  anthropic: [
    {
      id: "claude-3-5-sonnet-20241022",
      name: "Claude 3.5 Sonnet",
      provider: "anthropic",
      contextWindow: 200000,
      description: "Ottimo per traduzioni",
      recommended: true,
    },
    {
      id: "claude-3-5-haiku-20241022",
      name: "Claude 3.5 Haiku",
      provider: "anthropic",
      contextWindow: 200000,
      description: "Veloce ed economico",
    },
    {
      id: "claude-3-opus-20240229",
      name: "Claude 3 Opus",
      provider: "anthropic",
      contextWindow: 200000,
      description: "Il più potente",
    },
  ],

  local: [
    {
      id: "llama3.2",
      name: "Llama 3.2",
      provider: "local",
      description: "Meta's latest open model",
      recommended: true,
    },
    {
      id: "llama3.1",
      name: "Llama 3.1",
      provider: "local",
      description: "Molto capace per traduzioni",
    },
    {
      id: "mistral",
      name: "Mistral 7B",
      provider: "local",
      description: "Efficiente e veloce",
    },
    {
      id: "mixtral",
      name: "Mixtral 8x7B",
      provider: "local",
      description: "MoE, alta qualità",
    },
    {
      id: "qwen2.5",
      name: "Qwen 2.5",
      provider: "local",
      description: "Ottimo per multilingue",
    },
    {
      id: "gemma2",
      name: "Gemma 2",
      provider: "local",
      description: "Google's open model",
    },
    {
      id: "phi3",
      name: "Phi-3",
      provider: "local",
      description: "Microsoft, compatto",
    },
  ],

  openrouter: [
    {
      id: "google/gemini-2.0-flash-001",
      name: "Gemini 2.0 Flash",
      provider: "openrouter",
      description: "Via OpenRouter",
      recommended: true,
    },
    {
      id: "anthropic/claude-3.5-sonnet",
      name: "Claude 3.5 Sonnet",
      provider: "openrouter",
      description: "Via OpenRouter",
    },
    {
      id: "openai/gpt-4o",
      name: "GPT-4o",
      provider: "openrouter",
      description: "Via OpenRouter",
    },
    {
      id: "openai/gpt-4o-mini",
      name: "GPT-4o Mini",
      provider: "openrouter",
      description: "Via OpenRouter",
    },
    {
      id: "meta-llama/llama-3.3-70b-instruct",
      name: "Llama 3.3 70B",
      provider: "openrouter",
      description: "Via OpenRouter",
    },
    {
      id: "mistralai/mistral-large-2411",
      name: "Mistral Large",
      provider: "openrouter",
      description: "Via OpenRouter",
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
  // Globali
  { id: "tab-translate", action: "switchToTranslate", description: "Vai a Traduzione", defaultKey: "Alt+1", category: "global" },
  { id: "tab-sync", action: "switchToSync", description: "Vai a Sincronizzazione", defaultKey: "Alt+2", category: "global" },
  { id: "tab-settings", action: "switchToSettings", description: "Vai a Impostazioni", defaultKey: "Alt+3", category: "global" },
  { id: "tab-shortcuts", action: "switchToShortcuts", description: "Vai a Shortcut", defaultKey: "Alt+4", category: "global" },
  
  // Traduzione
  { id: "translate-open-file", action: "openInputFile", description: "Apri file SRT", defaultKey: "Ctrl+O", category: "translate" },
  { id: "translate-start", action: "startTranslation", description: "Avvia traduzione", defaultKey: "Ctrl+Enter", category: "translate" },
  { id: "translate-cancel", action: "cancelTranslation", description: "Annulla traduzione", defaultKey: "Escape", category: "translate" },
  { id: "translate-clear-logs", action: "clearLogs", description: "Cancella log", defaultKey: "Ctrl+L", category: "translate" },
  
  // Sincronizzazione
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
  { id: "sync-go-suggested", action: "goToSuggested", description: "Vai al suggerito", defaultKey: "Ctrl+G", category: "sync" },
  { id: "sync-save", action: "saveFile", description: "Salva SRT", defaultKey: "Ctrl+S", category: "sync" },
  
  // Impostazioni
  { id: "settings-add-key", action: "addApiKey", description: "Aggiungi chiave API", defaultKey: "Ctrl+N", category: "settings" },
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
