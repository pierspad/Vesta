<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount, onDestroy } from "svelte";
  import { languages, getModelsForProvider, providers } from "./models";

  // Props
  interface Props {
    onGoToSettings?: () => void;
  }

  let { onGoToSettings }: Props = $props();

  // Tipi
  interface SrtFileInfo {
    path: string;
    subtitle_count: number;
    first_subtitle: string;
    last_subtitle: string;
  }

  interface TranslateConfig {
    input_path: string;
    output_path: string;
    target_lang: string;
    api_key: string;
    api_type: string;
    batch_size: number;
    title_context: string | null;
    api_url: string | null;
    model: string | null;
  }

  interface TranslateProgressEvent {
    message: string;
    current_batch: number;
    total_batches: number;
    percentage: number;
    eta_seconds: number | null;
  }

  interface TranslateResult {
    success: boolean;
    message: string;
    output_path: string | null;
    translated_count: number;
  }

  interface ApiKeyConfig {
    id: string;
    name: string;
    apiType: "gemini" | "openai" | "local" | "anthropic" | "openrouter";
    apiKey: string;
    apiUrl?: string;
    isDefault: boolean;
  }

  interface ModelInfo {
    id: string;
    name: string;
    provider: string;
  }

  // State
  let inputPath = $state("");
  let outputPath = $state("");
  let targetLang = $state("it");
  let selectedKeyId = $state("");
  let batchSize = $state(10);
  let titleContext = $state("");
  let selectedModel = $state("");

  let fileInfo = $state<SrtFileInfo | null>(null);
  let isTranslating = $state(false);
  let progress = $state<TranslateProgressEvent | null>(null);
  let logs = $state<string[]>([]);
  let error = $state<string | null>(null);
  let result = $state<TranslateResult | null>(null);

  let apiKeys = $state<ApiKeyConfig[]>([]);
  let availableModels = $state<ModelInfo[]>([]);

  let unlistenProgress: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;

  // Derived: selected API key
  let selectedKey = $derived(apiKeys.find((k) => k.id === selectedKeyId) || null);
  let hasApiKey = $derived(apiKeys.length > 0);

  // Update models when key changes
  $effect(() => {
    if (selectedKey) {
      availableModels = getModelsForProvider(selectedKey.apiType);
      if (!selectedModel && availableModels.length > 0) {
        selectedModel = availableModels[0].id;
      }
    }
  });

  onMount(async () => {
    loadApiKeys();

    // Listen for progress events
    unlistenProgress = await listen<TranslateProgressEvent>(
      "translate-progress",
      (event) => {
        progress = event.payload;
        addLog(event.payload.message);
      }
    );

    unlistenComplete = await listen<TranslateResult>(
      "translate-complete",
      (event) => {
        result = event.payload;
        isTranslating = false;
        addLog(`✅ ${event.payload.message}`);
      }
    );
  });

  onDestroy(() => {
    unlistenProgress?.();
    unlistenComplete?.();
  });

  function loadApiKeys() {
    const saved = localStorage.getItem("srt-tools-api-keys");
    if (saved) {
      try {
        apiKeys = JSON.parse(saved);
        // Select default key
        const defaultKey = apiKeys.find((k) => k.isDefault);
        if (defaultKey) {
          selectedKeyId = defaultKey.id;
        } else if (apiKeys.length > 0) {
          selectedKeyId = apiKeys[0].id;
        }
      } catch {
        apiKeys = [];
      }
    }
  }

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    logs = [...logs, `[${timestamp}] ${message}`];
  }

  async function selectInputFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "SRT Files", extensions: ["srt"] }],
      });

      if (selected) {
        inputPath = selected as string;
        await loadFileInfo();

        // Auto-generate output path
        if (!outputPath) {
          outputPath = inputPath.replace(".srt", `.${targetLang}.srt`);
        }
      }
    } catch (e) {
      error = `Errore selezione file: ${e}`;
    }
  }

  async function selectOutputFile() {
    try {
      const selected = await save({
        filters: [{ name: "SRT Files", extensions: ["srt"] }],
        defaultPath: outputPath || undefined,
      });

      if (selected) {
        outputPath = selected;
      }
    } catch (e) {
      error = `Errore selezione file: ${e}`;
    }
  }

  async function loadFileInfo() {
    if (!inputPath) return;

    try {
      fileInfo = await invoke<SrtFileInfo>("load_srt_for_translate", {
        path: inputPath,
      });
      addLog(`📄 Caricato file con ${fileInfo.subtitle_count} sottotitoli`);
    } catch (e) {
      error = `Errore caricamento file: ${e}`;
      fileInfo = null;
    }
  }

  async function startTranslation() {
    if (!inputPath || !outputPath || !selectedKey) {
      error = "Seleziona un file e una chiave API";
      return;
    }

    error = null;
    result = null;
    progress = null;
    isTranslating = true;
    addLog("🚀 Avvio traduzione...");

    const config: TranslateConfig = {
      input_path: inputPath,
      output_path: outputPath,
      target_lang: targetLang,
      api_key: selectedKey.apiKey,
      api_type: selectedKey.apiType,
      batch_size: batchSize,
      title_context: titleContext || null,
      api_url: selectedKey.apiUrl || null,
      model: selectedModel || null,
    };

    try {
      const res = await invoke<TranslateResult>("start_translation", {
        config,
      });
      result = res;
      isTranslating = false;
    } catch (e) {
      error = `Errore traduzione: ${e}`;
      isTranslating = false;
      addLog(`❌ Errore: ${e}`);
    }
  }

  async function cancelTranslation() {
    try {
      await invoke("cancel_translation");
      isTranslating = false;
      addLog("⚠️ Traduzione annullata");
    } catch (e) {
      error = `Errore annullamento: ${e}`;
    }
  }

  function formatEta(seconds: number | null): string {
    if (seconds === null) return "...";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}m ${secs}s`;
  }

  function clearLogs() {
    logs = [];
  }

  function handleGoToSettings() {
    if (onGoToSettings) {
      onGoToSettings();
    }
  }
</script>

<div class="h-full flex flex-col p-6 overflow-auto bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950">
  <!-- Header -->
  <div class="mb-4">
    <h2 class="text-3xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
      Traduzione Sottotitoli
    </h2>
    <p class="text-gray-400 mt-1">
      Traduci file SRT usando AI (Gemini, OpenAI, Claude, Local LLM)
    </p>
  </div>

  <!-- API Warning Banner (shows when no API configured, disappears when configured) -->
  {#if !hasApiKey}
    <div class="mb-4 p-4 bg-amber-500/10 border border-amber-500/30 rounded-xl animate-fade-in">
      <p class="text-amber-300">
        ⚠️ Nessuna chiave API configurata. 
        <button 
          onclick={handleGoToSettings}
          class="underline hover:text-amber-200 font-medium transition-colors"
        >
          Vai alle Impostazioni
        </button> 
        per aggiungerne una.
      </p>
    </div>
  {/if}

  <!-- Main Grid: 2 columns -->
  <div class="flex-1 grid grid-cols-2 gap-6">
    <!-- Left Column: File Selection -->
    <div class="space-y-4">
      <!-- File Input Card -->
      <div class="glass-card p-5">
        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
          <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
          </svg>
          File
        </h3>

        <div class="space-y-3">
          <div>
            <label for="input-path" class="block text-sm text-gray-400 mb-1">File SRT Input</label>
            <div class="flex gap-2">
              <input
                id="input-path"
                type="text"
                bind:value={inputPath}
                placeholder="Seleziona file..."
                class="input-modern flex-1 text-sm"
                readonly
              />
              <button onclick={selectInputFile} class="btn-primary py-2 px-3">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                </svg>
              </button>
            </div>
          </div>

          <div>
            <label for="output-path" class="block text-sm text-gray-400 mb-1">File Output</label>
            <div class="flex gap-2">
              <input
                id="output-path"
                type="text"
                bind:value={outputPath}
                placeholder="Seleziona destinazione..."
                class="input-modern flex-1 text-sm"
              />
              <button onclick={selectOutputFile} class="btn-secondary py-2 px-3">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                </svg>
              </button>
            </div>
          </div>

          {#if fileInfo}
            <div class="p-3 bg-indigo-500/10 border border-indigo-500/30 rounded-lg">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-indigo-500/20 flex items-center justify-center">
                  <span class="text-xl">📄</span>
                </div>
                <div>
                  <p class="font-medium text-white">{fileInfo.subtitle_count} sottotitoli</p>
                  <p class="text-sm text-gray-400 truncate max-w-xs">"{fileInfo.first_subtitle}"</p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Log Panel -->
      <div class="glass-card p-5 flex-1 flex flex-col">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-lg font-semibold flex items-center gap-2">
            <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" />
            </svg>
            Log
          </h3>
          {#if logs.length > 0}
            <button onclick={clearLogs} class="text-xs text-gray-500 hover:text-gray-400">
              Cancella
            </button>
          {/if}
        </div>

        <div class="log-viewer flex-1 min-h-[200px] overflow-y-auto p-3 space-y-1">
          {#each logs as log}
            <p class="text-gray-400 text-xs">{log}</p>
          {/each}
          {#if logs.length === 0}
            <p class="text-gray-600 text-center py-8 text-sm">Nessun log...</p>
          {/if}
        </div>
      </div>
    </div>

    <!-- Right Column: Options + Progress -->
    <div class="space-y-4">
      <!-- Translation Options Card -->
      <div class="glass-card p-5">
        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
          <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          Opzioni Traduzione
        </h3>

        <div class="space-y-4">
          <!-- API Key Selection -->
          {#if hasApiKey}
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm text-gray-400 mb-1">Chiave API</label>
                <select bind:value={selectedKeyId} class="select-modern w-full text-sm">
                  {#each apiKeys as key}
                    <option value={key.id}>
                      {key.name} ({providers[key.apiType]?.name.split(' ')[0] || key.apiType})
                      {key.isDefault ? "⭐" : ""}
                    </option>
                  {/each}
                </select>
              </div>

              <div>
                <label class="block text-sm text-gray-400 mb-1">Modello</label>
                <select bind:value={selectedModel} class="select-modern w-full text-sm">
                  {#each availableModels as model}
                    <option value={model.id}>{model.name}</option>
                  {/each}
                </select>
              </div>
            </div>
          {/if}

          <!-- Language Selection - Custom styled -->
          <div>
            <label class="block text-sm text-gray-400 mb-1">Lingua Target</label>
            <div class="relative">
              <select
                bind:value={targetLang}
                class="select-modern-styled w-full text-sm appearance-none cursor-pointer"
              >
                {#each languages as lang}
                  <option value={lang.code}>{lang.flag} {lang.name}</option>
                {/each}
              </select>
              <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-gray-400">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </div>
            </div>
          </div>

          <!-- Batch Size -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <label class="text-sm text-gray-400 flex items-center gap-2">
                Batch Size
                <span 
                  class="tooltip cursor-help"
                  data-tooltip="Numero di sottotitoli per richiesta API. Valori alti = meno chiamate ma più token."
                >
                  <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </span>
              </label>
              <span class="text-white font-mono bg-white/10 px-2 py-0.5 rounded text-sm">{batchSize}</span>
            </div>
            <input
              type="range"
              bind:value={batchSize}
              min="5"
              max="50"
              step="5"
              class="w-full"
            />
            <div class="flex justify-between text-xs text-gray-500 mt-1">
              <span>5 (lento)</span>
              <span>50 (veloce)</span>
            </div>
          </div>

          <!-- Context -->
          <div>
            <label class="block text-sm text-gray-400 mb-1">
              Contesto <span class="text-gray-500">(opzionale)</span>
            </label>
            <input
              type="text"
              bind:value={titleContext}
              placeholder="es: Pulp Fiction, Breaking Bad..."
              class="input-modern w-full text-sm"
            />
          </div>
        </div>
      </div>

      <!-- Progress Card -->
      {#if isTranslating || progress}
        <div class="glass-card p-5 animate-fade-in {isTranslating ? 'animate-pulse-glow' : ''}">
          <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
            <svg class="w-5 h-5 text-blue-400 {isTranslating ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            Progresso
          </h3>

          <div class="space-y-3">
            <div class="progress-modern h-3">
              <div class="progress-modern-bar" style="width: {progress?.percentage || 0}%"></div>
            </div>

            <div class="flex justify-between items-center">
              <span class="text-gray-400 text-sm">
                Batch {progress?.current_batch || 0} / {progress?.total_batches || 0}
              </span>
              <span class="text-2xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
                {Math.round(progress?.percentage || 0)}%
              </span>
            </div>

            {#if progress?.eta_seconds}
              <div class="flex items-center gap-2 text-gray-400 text-sm">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Tempo rimanente: {formatEta(progress.eta_seconds)}</span>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Result Card -->
      {#if result}
        <div
          class="glass-card p-5 border-l-4 animate-fade-in {result.success
            ? 'border-green-500 bg-green-500/5'
            : 'border-red-500 bg-red-500/5'}"
        >
          <div class="flex items-start gap-4">
            <div class="w-10 h-10 rounded-xl flex items-center justify-center {result.success ? 'bg-green-500/20' : 'bg-red-500/20'}">
              {#if result.success}
                <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              {:else}
                <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              {/if}
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-semibold {result.success ? 'text-green-400' : 'text-red-400'}">
                {result.success ? "Traduzione Completata" : "Errore"}
              </h3>
              <p class="text-gray-300 text-sm mt-1">{result.message}</p>
              {#if result.output_path}
                <p class="text-xs text-gray-500 mt-2 font-mono truncate">📁 {result.output_path}</p>
              {/if}
            </div>
          </div>
        </div>
      {/if}

      <!-- Error Card -->
      {#if error}
        <div class="glass-card p-4 border border-red-500/30 bg-red-500/10 animate-fade-in">
          <div class="flex items-center gap-3">
            <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="text-red-300 flex-1 text-sm">{error}</p>
            <button onclick={() => (error = null)} class="text-red-400 hover:text-red-300">✕</button>
          </div>
        </div>
      {/if}

      <!-- Action Button -->
      <div class="flex gap-3">
        {#if isTranslating}
          <button onclick={cancelTranslation} class="btn-danger flex-1 py-4 text-lg">
            <svg class="w-5 h-5 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
            Annulla
          </button>
        {:else}
          <button
            onclick={startTranslation}
            disabled={!inputPath || !outputPath || !selectedKey}
            class="btn-success flex-1 py-4 text-lg disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
          >
            <svg class="w-5 h-5 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            Avvia Traduzione
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .select-modern-styled {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 12px 40px 12px 16px;
    color: white;
    transition: all 0.3s ease;
  }

  .select-modern-styled:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2);
  }

  .select-modern-styled option {
    background: #1a1a2e;
    color: white;
    padding: 10px;
  }
</style>
