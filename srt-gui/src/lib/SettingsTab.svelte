<script lang="ts">
  import { onMount } from "svelte";
  import {
    providers,
    providerOrder,
    getModelsForProvider,
    saveCustomModel,
    deleteCustomModel,
    getCustomModels,
    formatContextWindow,
    loadAndValidateApiKeys,
    type ModelInfo,
    type CustomModel,
    type ApiKeyConfig,
  } from "./models";
  import { locale, currentLanguage, availableUILanguages, setLanguage } from "./i18n";

  // State
  let apiKeys = $state<ApiKeyConfig[]>([]);
  let selectedProviderType = $state<string>("local"); // "local" or "openrouter"
  let selectedFamily = $state<string | null>(null);
  
  let showAddKey = $state(false);
  let showAddModel = $state(false); // Legacy/Unused for now but kept for safety
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);

  // Reactive translation
  let t = $derived($locale);

  // New key form
  let newKeyName = $state("");
  let newKeyType = $state<ApiKeyConfig["apiType"]>("openrouter");
  let newKeyValue = $state("");
  let newKeyUrl = $state("");
  let newKeyModelName = $state("");

  // Computed
  let currentProviderModels = $derived(getModelsForProvider(selectedProviderType));
  
  // Group models by family
  let families = $derived.by(() => {
    const fams = new Set<string>();
    currentProviderModels.forEach(m => {
      if (m.family) fams.add(m.family);
    });
    return Array.from(fams).sort();
  });

  // Select first family by default if none selected or invalid
  $effect(() => {
    if (families.length > 0 && (!selectedFamily || !families.includes(selectedFamily))) {
      selectedFamily = families[0];
    }
  });

  let filteredModels = $derived(
    selectedFamily 
      ? currentProviderModels.filter(m => m.family === selectedFamily)
      : []
  );

  // Count keys per provider
  let keysPerProvider = $derived(() => {
    const counts: Record<string, number> = {};
    providerOrder.forEach((p) => {
      counts[p] = apiKeys.filter((k) => k.apiType === p).length;
    });
    return counts;
  });

  onMount(() => {
    loadApiKeys();
    
    // Global ESC key handler for closing modals
    const handleKeydown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (deleteConfirmId) {
          cancelDelete();
        } else if (showAddKey) {
          showAddKey = false;
        }
      }
    };
    
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });

  function loadApiKeys() {
    apiKeys = loadAndValidateApiKeys();
  }

  function saveApiKeys() {
    localStorage.setItem("srt-tools-api-keys", JSON.stringify(apiKeys));
  }

  function generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }

  function openAddKeyModal(providerId?: string, modelName?: string) {
    if (providerId) {
      newKeyType = providerId as ApiKeyConfig["apiType"];
      newKeyName = providers[providerId]?.name || "";
    }
    if (modelName) {
      newKeyModelName = modelName;
    }
    showAddKey = true;
  }

  function addApiKey() {
    if (!newKeyName.trim()) {
      error = t("settings.errorNameRequired");
      return;
    }

    // Per provider locali, la chiave può essere vuota
    if (newKeyType !== "local" && !newKeyValue.trim()) {
      error = t("settings.errorKeyRequired");
      return;
    }

    const newKey: ApiKeyConfig = {
      id: generateId(),
      name: newKeyName.trim(),
      apiType: newKeyType,
      apiKey: newKeyValue.trim(),
      apiUrl: newKeyUrl.trim() || undefined,
      modelName: newKeyModelName.trim() || undefined,
      isDefault: apiKeys.filter((k) => k.apiType === newKeyType).length === 0,
    };

    apiKeys = [...apiKeys, newKey];
    saveApiKeys();

    // Reset form
    newKeyName = "";
    newKeyValue = "";
    newKeyUrl = "";
    newKeyModelName = "";
    showAddKey = false;

    success = t("settings.keyAdded");
    setTimeout(() => (success = null), 3000);
  }

  // Delete confirmation state
  let deleteConfirmId = $state<string | null>(null);
  let deleteConfirmName = $state<string>("");

  function askDeleteApiKey(id: string) {
    const key = apiKeys.find((k) => k.id === id);
    if (!key) return;
    deleteConfirmId = id;
    deleteConfirmName = key.name;
  }

  function cancelDelete() {
    deleteConfirmId = null;
    deleteConfirmName = "";
  }

  function confirmDeleteApiKey() {
    if (!deleteConfirmId) return;
    
    const key = apiKeys.find((k) => k.id === deleteConfirmId);
    if (!key) {
      cancelDelete();
      return;
    }

    const wasDefault = key.isDefault;
    const keyType = key.apiType;
    apiKeys = apiKeys.filter((k) => k.id !== deleteConfirmId);

    // Set new default if needed
    if (wasDefault) {
      const sameTypeKeys = apiKeys.filter((k) => k.apiType === keyType);
      if (sameTypeKeys.length > 0) {
        sameTypeKeys[0].isDefault = true;
      }
    }

    saveApiKeys();
    success = t("settings.keyDeleted");
    setTimeout(() => (success = null), 3000);
    cancelDelete();
  }

  function setDefaultKey(id: string) {
    const key = apiKeys.find((k) => k.id === id);
    if (!key) return;

    // Remove default from same type
    apiKeys = apiKeys.map((k) => ({
      ...k,
      isDefault: k.apiType === key.apiType ? k.id === id : k.isDefault,
    }));
    saveApiKeys();
  }

  // Visibility toggle for API keys
  let visibleKeyIds = $state<Set<string>>(new Set());

  function toggleKeyVisibility(keyId: string) {
    const newSet = new Set(visibleKeyIds);
    if (newSet.has(keyId)) {
      newSet.delete(keyId);
    } else {
      newSet.add(keyId);
    }
    visibleKeyIds = newSet;
  }

  function maskApiKey(key: string): string {
    if (!key || key.length <= 8) return "••••••••";
    return key.substring(0, 4) + "••••" + key.substring(key.length - 4);
  }

  function formatApiKeyForDisplay(key: string, isVisible: boolean): string {
    if (!key) return "—";
    if (isVisible) {
      // Show full key with special character indicators
      return key.split('').map(char => {
        if (char === ' ') return '␣'; // Space indicator
        if (char === '\t') return '→'; // Tab indicator
        if (char === '\n') return '↵'; // Newline indicator
        return char;
      }).join('');
    }
    return maskApiKey(key);
  }

  function hasSpecialChars(key: string): boolean {
    return /[\s\t\n]/.test(key);
  }

  function onModelClick(model: ModelInfo) {
    if (model.provider === 'openrouter') {
        newKeyUrl = "https://openrouter.ai/api/v1";
    }
    // Usa l'ID del modello, non il nome display
    openAddKeyModal(model.provider, model.id);
  }
</script>

<div class="h-full flex flex-col p-6 overflow-hidden bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950">
  <!-- Header & Language -->
  <div class="mb-6 flex flex-col gap-4">
    <div>
      <h2 class="text-3xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
        {t("settings.title")}
      </h2>
      <p class="text-gray-400 mt-1">
        {t("settings.subtitle")}
      </p>
    </div>

    <!-- Language Selector Bar -->
    <div class="glass-card p-3 flex items-center gap-4 overflow-x-auto">
      <span class="text-xs font-bold text-gray-500 uppercase tracking-wide whitespace-nowrap px-2">
        {t("settings.language")}
      </span>
      <div class="flex gap-2">
        {#each availableUILanguages as lang}
          <button
            onclick={() => setLanguage(lang.code)}
            class="flex items-center gap-2 px-3 py-1.5 rounded-lg transition-all duration-200 border
              {$currentLanguage === lang.code 
                ? 'bg-gradient-to-r from-indigo-500/20 to-purple-500/20 border-indigo-500/50 text-white shadow-sm' 
                : 'bg-white/5 hover:bg-white/10 text-gray-400 hover:text-gray-200 border-transparent hover:border-white/10'}"
          >
            <span class="text-base">{lang.flag}</span>
            <span class="text-xs font-medium uppercase">{lang.code}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>

  <!-- Notifications -->
  {#if error}
    <div class="mb-4 p-4 bg-red-500/10 border border-red-500/30 rounded-xl flex items-center gap-3 animate-fade-in shrink-0">
      <span class="text-red-300 flex-1">{error}</span>
      <button onclick={() => (error = null)} class="text-red-400 hover:text-red-300">✕</button>
    </div>
  {/if}

  {#if success}
    <div class="mb-4 p-4 bg-green-500/10 border border-green-500/30 rounded-xl flex items-center gap-3 animate-fade-in shrink-0">
      <span class="text-green-300">{success}</span>
    </div>
  {/if}

  <!-- Main Content 3-Column Grid -->
  <div class="grid grid-cols-12 gap-6 flex-1 min-h-0">
    
    <!-- COL 1: Provider & Families (3 cols wide) -->
    <div class="col-span-3 flex flex-col gap-4">
      <!-- Add Custom Key Button -->
      <button
        onclick={() => openAddKeyModal(selectedProviderType)}
        class="btn-primary w-full py-3 flex items-center justify-center gap-2 shadow-lg shadow-indigo-500/20"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        <span>{t("settings.addCustomApiKey")}</span>
      </button>

      <div class="glass-card flex-1 flex flex-col min-h-0">
        <!-- Provider Toggle -->
        <div class="p-4 border-b border-white/5">
          <div class="grid grid-cols-2 gap-1 p-1 bg-black/20 rounded-xl">
            {#each providerOrder as pid}
              <button
                onclick={() => selectedProviderType = pid}
                class="py-2 px-3 rounded-lg text-xs font-medium transition-all duration-200
                  {selectedProviderType === pid 
                    ? 'bg-white/10 text-white shadow-sm' 
                    : 'text-gray-500 hover:text-gray-300'}"
              >
                {providers[pid].name}
              </button>
            {/each}
          </div>
        </div>

        <!-- Family List -->
        <div class="flex-1 overflow-y-auto p-2 space-y-1">
          <h3 class="text-xs font-bold text-gray-500 uppercase tracking-wide px-3 py-2">
            {t("settings.modelFamilies")}
          </h3>
          {#each families as family}
            <button
              onclick={() => selectedFamily = family}
              class="w-full text-left px-4 py-3 rounded-lg text-sm transition-all duration-200 flex items-center justify-between group
                {selectedFamily === family 
                  ? 'bg-indigo-500/10 text-indigo-300 border border-indigo-500/20' 
                  : 'text-gray-400 hover:bg-white/5 hover:text-gray-200 border border-transparent'}"
            >
              <span>{family}</span>
              <svg class="w-4 h-4 opacity-0 group-hover:opacity-100 transition-opacity {selectedFamily === family ? 'opacity-100' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- COL 2: Models List (5 cols wide) -->
    <div class="col-span-5 flex flex-col min-h-0">
      <div class="glass-card flex-1 flex flex-col min-h-0">
        <div class="p-4 border-b border-white/5 bg-white/5 backdrop-blur-sm sticky top-0 z-10">
          <h3 class="text-sm font-semibold text-white flex items-center gap-2">
            <span class="text-gray-400">Models /</span> {selectedFamily}
          </h3>
        </div>
        
        <div class="flex-1 overflow-y-auto p-2 space-y-2">
          {#each filteredModels as model}
            <button
              onclick={() => onModelClick(model)}
              class="w-full text-left p-3 rounded-xl transition-all duration-200 group
                 bg-white/5 hover:bg-white/10 border border-white/5 hover:border-white/10"
            >
              <div class="flex justify-between items-start mb-1">
                <span class="font-medium text-gray-200 group-hover:text-white transition-colors">
                  {model.name}
                </span>
                {#if model.recommended}
                  <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-green-500/20 text-green-400 border border-green-500/20">
                    {t("settings.recommended")}
                  </span>
                {/if}
              </div>
              
              <div class="flex items-center gap-2 text-[10px] text-gray-600 font-mono">
                <span class="bg-black/30 px-1.5 py-0.5 rounded">
                  {model.id}
                </span>
                {#if model.contextWindow}
                  <span class="text-gray-500">
                    {formatContextWindow(model.contextWindow)} {t("settings.ctx")}
                  </span>
                {/if}
              </div>
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- COL 3: Configured Keys (4 cols wide) -->
    <div class="col-span-4 flex flex-col min-h-0">
      <div class="glass-card flex-1 flex flex-col min-h-0 border-l border-white/5">
        <div class="p-4 border-b border-white/5">
          <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wide">
            {t("settings.apiKeys")}
          </h3>
        </div>

        <div class="flex-1 overflow-y-auto p-2 space-y-2">
          {#each apiKeys as key}
            <div
              class="p-3 bg-white/5 rounded-xl border border-white/10 hover:border-white/20 transition-all group
                {key.isDefault ? 'ring-1 ring-indigo-500/50 bg-indigo-500/5' : ''}"
            >
              <div class="flex items-start gap-3">
                <div class="w-8 h-8 rounded-lg bg-gradient-to-br {providers[key.apiType]?.color || 'from-gray-500 to-gray-600'} flex items-center justify-center flex-shrink-0 text-white text-xs shadow-lg">
                  {providers[key.apiType]?.icon || "?"}
                </div>

                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-0.5">
                    <span class="font-medium text-gray-200 text-sm truncate">{key.name}</span>
                    {#if key.isDefault}
                      <!-- Pin icon for default -->
                      <svg class="w-3.5 h-3.5 text-indigo-400" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M16 4c.55 0 1 .45 1 1v4.38l1.71 1.71c.18.18.29.43.29.7V14c0 .55-.45 1-1 1h-5v5l-1 1-1-1v-5H6c-.55 0-1-.45-1-1v-2.21c0-.27.11-.52.29-.71L7 9.38V5c0-.55.45-1 1-1h8zm-1 2H9v3.62l-2 2V13h10v-1.38l-2-2V6z"/>
                      </svg>
                    {/if}
                  </div>
                  <div class="flex items-center gap-1.5">
                    <button
                      onclick={() => toggleKeyVisibility(key.id)}
                      class="text-[10px] text-gray-500 font-mono truncate hover:text-gray-300 transition-colors flex items-center gap-1"
                      title={t("settings.toggleVisibility")}
                    >
                      {#if visibleKeyIds.has(key.id)}
                        <svg class="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                        </svg>
                      {:else}
                        <svg class="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                        </svg>
                      {/if}
                      <span class="truncate">{formatApiKeyForDisplay(key.apiKey, visibleKeyIds.has(key.id))}</span>
                    </button>
                    {#if hasSpecialChars(key.apiKey)}
                      <span class="text-[9px] bg-amber-500/20 text-amber-400 px-1 py-0.5 rounded flex-shrink-0" title={t("settings.hasSpecialChars")}>
                        ⚠
                      </span>
                    {/if}
                  </div>
                  {#if key.modelName}
                    <div class="text-[10px] text-indigo-400 mt-1 truncate">{t("settings.model")}: {key.modelName}</div>
                  {/if}
                </div>

                <div class="flex flex-col gap-1 opacity-100 sm:opacity-0 sm:group-hover:opacity-100 transition-opacity">
                  {#if !key.isDefault}
                    <button
                      onclick={() => setDefaultKey(key.id)}
                      class="p-1.5 text-gray-500 hover:text-indigo-400 hover:bg-white/10 rounded transition-colors"
                      title={t("settings.setAsDefault")}
                    >
                      <!-- Pin icon -->
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v4.38l1.71 1.71c.18.18.29.43.29.7V14a2 2 0 01-2 2h-5v4l-2 2-2-2v-4H5a2 2 0 01-2-2v-2.21c0-.27.11-.52.29-.71L5 9.38V5z" />
                      </svg>
                    </button>
                  {/if}
                  <button
                    onclick={() => askDeleteApiKey(key.id)}
                    class="p-1.5 text-gray-500 hover:text-red-400 hover:bg-white/10 rounded transition-colors"
                    title={t("settings.delete")}
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          {/each}

          {#if apiKeys.length === 0}
            <div class="flex-1 flex flex-col items-center justify-center text-gray-500 p-8 text-center opacity-50">
              <svg class="w-10 h-10 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
              </svg>
              <p class="text-xs">{t("settings.noApiKeys")}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>

  <!-- Modal: Add API Key -->
  {#if showAddKey}
    <div 
      class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4" 
      onmousedown={(e) => { if (e.target === e.currentTarget) showAddKey = false; }}
    >
      <div class="w-full max-w-lg overflow-hidden animate-fade-in shadow-2xl border border-white/20 bg-gray-900/98 backdrop-blur-xl rounded-xl" onmousedown={(e) => e.stopPropagation()}>
        <div class="p-6 border-b border-white/5 bg-white/5">
          <h3 class="text-xl font-bold text-white flex items-center gap-2">
             {t("settings.modal.addCustomApiKey")}
          </h3>
        </div>

        <div class="p-6 space-y-5">
          <!-- Protocol Info -->
          <div class="p-3 bg-indigo-500/10 border border-indigo-500/20 rounded-lg flex gap-3">
             <div class="text-lg">💡</div>
             <p class="text-xs text-indigo-200 leading-relaxed">
               {t("settings.modal.protocolInfo")}
             </p>
          </div>

          <!-- Types Grid -->
          <div>
            <label class="block text-xs font-bold text-gray-500 uppercase tracking-wide mb-2">{t("settings.modal.provider")}</label>
            <div class="grid grid-cols-2 gap-3">
              {#each providerOrder as providerId}
                {@const provider = providers[providerId]}
                <button
                  type="button"
                  onclick={() => { 
                    newKeyType = providerId as ApiKeyConfig["apiType"]; 
                    newKeyName = provider.name; 
                    // Auto-fill endpoint
                    if (provider.defaultApiUrl) {
                      newKeyUrl = provider.defaultApiUrl;
                    }
                  }}
                  class="flex items-center gap-3 p-3 rounded-lg transition-all duration-200 border text-left
                    {newKeyType === providerId 
                      ? 'bg-indigo-500/20 border-indigo-500/50 text-white' 
                      : 'bg-white/5 hover:bg-white/10 border-transparent text-gray-400'}"
                >
                  <span class="text-2xl">{provider.icon}</span>
                  <div class="flex flex-col">
                    <span class="text-sm font-bold">
                      {providerId === 'local' ? t("settings.modal.localServer") : t("settings.modal.cloudApi")}
                    </span>
                    <span class="text-[10px] opacity-70 leading-tight">
                      {providerId === 'local' ? t("settings.modal.localServerDesc") : t("settings.modal.cloudApiDesc")}
                    </span>
                  </div>
                </button>
              {/each}
            </div>
          </div>

          <div class="space-y-4">
            <div>
              <label for="key-name" class="block text-xs font-bold text-gray-500 uppercase tracking-wide mb-1.5">{t("settings.modal.configName")}</label>
              <input
                id="key-name"
                type="text"
                bind:value={newKeyName}
                placeholder={t("settings.modal.configNamePlaceholder")}
                class="w-full bg-black/40 border border-white/10 rounded-lg px-4 py-2.5 text-sm text-white focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 outline-none transition-all placeholder-gray-600"
              />
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="col-span-2">
                <label for="api-url" class="block text-xs font-bold text-gray-500 uppercase tracking-wide mb-1.5">
                  {t("settings.modal.apiEndpoint")}
                </label>
                <input
                  id="api-url"
                  type="text"
                  bind:value={newKeyUrl}
                  placeholder={providers[newKeyType]?.defaultApiUrl || "https://..."}
                  class="w-full bg-black/40 border border-white/10 rounded-lg px-4 py-2.5 text-sm text-white focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 outline-none transition-all placeholder-gray-600 font-mono"
                />
              </div>

              <div class="col-span-2">
                <label for="api-key" class="block text-xs font-bold text-gray-500 uppercase tracking-wide mb-1.5">{t("settings.modal.apiKey")}</label>
                <input
                  id="api-key"
                  type="password"
                  bind:value={newKeyValue}
                  disabled={newKeyType === "local"}
                  placeholder={newKeyType === "local" ? t("settings.modal.notRequiredForLocal") : "sk-..."}
                  class="w-full bg-black/40 border border-white/10 rounded-lg px-4 py-2.5 text-sm text-white focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 outline-none transition-all placeholder-gray-600 font-mono disabled:opacity-50 disabled:cursor-not-allowed"
                />
              </div>

              <!-- Model ID - sempre visibile per entrambi i provider -->
              <div class="col-span-2 animate-fade-in">
                <label for="model-id" class="block text-xs font-bold text-gray-500 uppercase tracking-wide mb-1.5">
                  {t("settings.modal.defaultModel")}
                  {#if newKeyType === "openrouter"}
                    <span class="text-indigo-400 font-normal ml-1">(OpenRouter Model ID)</span>
                  {/if}
                </label>
                <input
                  id="model-id"
                  type="text"
                  bind:value={newKeyModelName}
                  placeholder={newKeyType === "local" ? "e.g. llama3.2, gemma3:27b..." : "e.g. google/gemini-2.5-flash, anthropic/claude-3.5-sonnet..."}
                  class="w-full bg-black/40 border border-white/10 rounded-lg px-4 py-2.5 text-sm text-white focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 outline-none transition-all placeholder-gray-600 font-mono"
                />
                {#if newKeyType === "openrouter"}
                  <p class="text-[10px] text-gray-500 mt-1.5 leading-relaxed">
                    💡 {t("settings.modal.openrouterModelHint")} <a href="https://openrouter.ai/models" target="_blank" class="text-indigo-400 hover:text-indigo-300 underline">openrouter.ai/models</a>
                  </p>
                {/if}
              </div>
            </div>
          </div>

          <div class="flex gap-3 pt-4 border-t border-white/5">
            <button onclick={() => (showAddKey = false)} class="flex-1 py-2.5 rounded-lg border border-white/10 text-gray-400 hover:bg-white/5 hover:text-white transition-all text-sm font-medium">
              {t("settings.modal.cancel")}
            </button>
            <button onclick={addApiKey} class="flex-1 py-2.5 rounded-lg bg-indigo-500 hover:bg-indigo-400 text-white shadow-lg shadow-indigo-500/20 transition-all text-sm font-bold">
              {t("settings.modal.save")}
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Modal: Delete Confirmation -->
  {#if deleteConfirmId}
    <div 
      class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4" 
      onmousedown={(e) => { if (e.target === e.currentTarget) cancelDelete(); }}
    >
      <div class="w-full max-w-sm overflow-hidden animate-fade-in shadow-2xl border border-white/20 bg-gray-900/98 backdrop-blur-xl rounded-xl" onmousedown={(e) => e.stopPropagation()}>
        <div class="p-6 border-b border-white/5 bg-white/5">
          <h3 class="text-xl font-bold text-white">{t("app.title")}</h3>
        </div>
        
        <div class="p-6 space-y-4">
          <p class="text-gray-300">
            {t("settings.confirmDeleteKey", { name: deleteConfirmName })}
          </p>
          
          <div class="flex gap-3 pt-2">
            <button onclick={cancelDelete} class="flex-1 py-2.5 rounded-lg border border-white/10 text-gray-400 hover:bg-white/5 hover:text-white transition-all text-sm font-medium">
              {t("settings.modal.cancel")}
            </button>
            <button onclick={confirmDeleteApiKey} class="flex-1 py-2.5 rounded-lg bg-red-500 hover:bg-red-400 text-white shadow-lg shadow-red-500/20 transition-all text-sm font-bold">
              {t("settings.confirmDelete")}
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Custom Scrollbar for inner elements */
  ::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }
  ::-webkit-scrollbar-track {
    background: transparent;
  }
  ::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }
  ::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }
</style>
