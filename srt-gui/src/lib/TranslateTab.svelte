<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount, onDestroy } from "svelte";
  import { languages, getModelsForProvider, providers, loadAndValidateApiKeys, type ApiKeyConfig } from "./models";
  import { locale } from "./i18n";

  // Props
  interface Props {
    onGoToSettings?: () => void;
  }

  let { onGoToSettings }: Props = $props();

  // Reactive translation
  let t = $derived($locale);

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

  // Simplified to only 2 providers: local and openrouter


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
  
  // Live subtitle preview
  let currentSubtitleOriginal = $state<string>("");
  let currentSubtitleTranslated = $state<string>("");

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
    apiKeys = loadAndValidateApiKeys();
    
    // Select default key
    const defaultKey = apiKeys.find((k) => k.isDefault);
    if (defaultKey) {
      selectedKeyId = defaultKey.id;
    } else if (apiKeys.length > 0) {
      selectedKeyId = apiKeys[0].id;
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
      error = `${t("translate.errorSelectingFile")} ${e}`;
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
      error = `${t("translate.errorSelectingFile")} ${e}`;
    }
  }

  async function loadFileInfo() {
    if (!inputPath) return;

    try {
      fileInfo = await invoke<SrtFileInfo>("load_srt_for_translate", {
        path: inputPath,
      });
      addLog(`📄 ${t("translate.loadedFile", { count: fileInfo.subtitle_count })}`);
    } catch (e) {
      error = `${t("translate.errorLoading")} ${e}`;
      fileInfo = null;
    }
  }

  async function startTranslation() {
    if (!inputPath || !outputPath || !selectedKey) {
      error = t("translate.selectFileAndKey");
      return;
    }

    error = null;
    result = null;
    progress = null;
    isTranslating = true;
    addLog(`🚀 ${t("translate.starting")}`);

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
      error = `${t("translate.errorTranslating")} ${e}`;
      isTranslating = false;
      addLog(`❌ ${t("translate.error")}: ${e}`);
    }
  }

  async function cancelTranslation() {
    try {
      await invoke("cancel_translation");
      isTranslating = false;
      addLog(`⚠️ ${t("translate.cancelled")}`);
    } catch (e) {
      error = `${t("translate.errorCancelling")} ${e}`;
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

<div class="h-full flex flex-col p-6 overflow-hidden bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950">
  <!-- Header -->
  <div class="mb-4 shrink-0">
    <h2 class="text-3xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
      {t("translate.title")}
    </h2>
    <p class="text-gray-400 mt-1">
      {t("translate.subtitle")}
    </p>
  </div>

  <!-- API Warning Banner (shows when no API configured, disappears when configured) -->
  {#if !hasApiKey}
    <div class="mb-4 p-4 bg-amber-500/10 border border-amber-500/30 rounded-xl animate-fade-in">
      <p class="text-amber-300">
        ⚠️ {t("translate.noApiWarning")} 
        <button 
          onclick={handleGoToSettings}
          class="underline hover:text-amber-200 font-medium transition-colors"
        >
          {t("translate.goToSettings")}
        </button> 
        {t("translate.toAddOne")}
      </p>
    </div>
  {/if}

  <!-- Main Grid: 2 columns - Options left, File right -->
  <div class="grid grid-cols-2 gap-6">
    <!-- Left Column: Translation Options -->
    <div class="space-y-4">
      <!-- Translation Options Card -->
      <div class="glass-card p-5">
        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
          <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          {t("translate.options")}
        </h3>

        <div class="space-y-4">
          <!-- API Key Selection -->
          {#if hasApiKey}
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label for="api-key-select" class="block text-sm text-gray-400 mb-1">{t("translate.apiKey")}</label>
                <div class="relative">
                  <select id="api-key-select" bind:value={selectedKeyId} class="select-modern w-full text-sm appearance-none pr-10">
                    {#each apiKeys as key}
                      <option value={key.id}>
                        {key.name} {key.isDefault ? "⭐" : ""}
                      </option>
                    {/each}
                  </select>
                  <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-gray-400">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                  </div>
                </div>
              </div>

              <div>
                <label for="model-select" class="block text-sm text-gray-400 mb-1">{t("translate.model")}</label>
                <div class="relative">
                  <select id="model-select" bind:value={selectedModel} class="select-modern w-full text-sm appearance-none pr-10">
                    {#each availableModels as model}
                      <option value={model.id}>{model.name}</option>
                    {/each}
                  </select>
                  <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-gray-400">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          {/if}

          <!-- Language Selection -->
          <div>
            <label for="target-lang" class="block text-sm text-gray-400 mb-1">{t("translate.targetLang")}</label>
            <div class="relative">
              <select
                id="target-lang"
                bind:value={targetLang}
                class="select-modern w-full text-sm appearance-none cursor-pointer pr-10"
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
                {t("translate.batchSize")}
                <span class="cursor-help" title={t("translate.batchSizeTooltip")}>
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
              <span>5 ({t("translate.slow")})</span>
              <span>50 ({t("translate.fast")})</span>
            </div>
          </div>

          <!-- Context -->
          <div>
            <label for="context-input" class="block text-sm text-gray-400 mb-1">
              {t("translate.context")} <span class="text-gray-500">{t("translate.contextOptional")}</span>
            </label>
            <input
              id="context-input"
              type="text"
              bind:value={titleContext}
              placeholder={t("translate.contextPlaceholder")}
              class="input-modern w-full text-sm"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Right Column: File + Start Button -->
    <div class="space-y-4">
      <!-- File Input Card -->
      <div class="glass-card p-5">
        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
          <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
          </svg>
          {t("translate.file")}
        </h3>

        <div class="space-y-3">
          <div>
            <label for="input-path" class="block text-sm text-gray-400 mb-1">{t("translate.inputFile")}</label>
            <div class="flex gap-2">
              <input
                id="input-path"
                type="text"
                bind:value={inputPath}
                placeholder={t("translate.selectFile")}
                class="input-modern flex-1 text-sm"
                readonly
              />
              <button 
                onclick={selectInputFile} 
                class="btn-primary py-2 px-3"
                title={t("translate.tooltip.upload")}
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                </svg>
              </button>
            </div>
          </div>

          <div>
            <label for="output-path" class="block text-sm text-gray-400 mb-1">{t("translate.outputFile")}</label>
            <div class="flex gap-2">
              <input
                id="output-path"
                type="text"
                bind:value={outputPath}
                placeholder={t("translate.selectDestination")}
                class="input-modern flex-1 text-sm"
              />
              <button 
                onclick={selectOutputFile} 
                class="btn-secondary py-2 px-3"
                title={t("translate.tooltip.save")}
              >
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
                  <p class="font-medium text-white">{fileInfo.subtitle_count} {t("translate.subtitles")}</p>
                  <p class="text-sm text-gray-400 truncate max-w-xs">"{fileInfo.first_subtitle}"</p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Action Button -->
      <div class="flex gap-3">
        {#if isTranslating}
          <button onclick={cancelTranslation} class="btn-danger flex-1 py-4 text-lg">
            <svg class="w-5 h-5 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
            {t("translate.cancel")}
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
            {t("translate.start")}
          </button>
        {/if}
      </div>
    </div>
  </div>

  <!-- Bottom Section: Progress + Live Preview -->
  <div class="mt-6 flex-1 flex flex-col space-y-4 min-h-0">
    <!-- Progress/Result/Error Cards -->
    {#if isTranslating || progress}
      <div class="glass-card p-4 animate-fade-in {isTranslating ? 'animate-pulse-glow' : ''}">
        <div class="flex items-center gap-6">
          <div class="flex-1">
            <div class="progress-modern h-2">
              <div class="progress-modern-bar" style="width: {progress?.percentage || 0}%"></div>
            </div>
          </div>
          <span class="text-gray-400 text-sm whitespace-nowrap">
            {t("translate.batch")} {progress?.current_batch || 0}/{progress?.total_batches || 0}
          </span>
          <span class="text-xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
            {Math.round(progress?.percentage || 0)}%
          </span>
          {#if progress?.eta_seconds}
            <span class="text-gray-500 text-sm flex items-center gap-1">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              {formatEta(progress.eta_seconds)}
            </span>
          {/if}
        </div>
      </div>
    {/if}

    {#if result}
      <div class="glass-card p-4 border-l-4 animate-fade-in {result.success ? 'border-green-500 bg-green-500/5' : 'border-red-500 bg-red-500/5'}">
        <div class="flex items-center gap-3">
          {#if result.success}
            <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          {:else}
            <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          {/if}
          <div class="flex-1">
            <p class="{result.success ? 'text-green-400' : 'text-red-400'} font-medium">{result.message}</p>
            {#if result.output_path}
              <p class="text-xs text-gray-500 mt-1 font-mono truncate">📁 {result.output_path}</p>
            {/if}
          </div>
        </div>
      </div>
    {/if}

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

    <!-- Live Translation Preview -->
    <div class="glass-card p-5 flex-1 flex flex-col min-h-0">
      <div class="flex items-center justify-between mb-4 shrink-0">
        <h3 class="text-lg font-semibold flex items-center gap-2">
          <svg class="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          {t("translate.livePreview")}
        </h3>
        {#if logs.length > 0}
          <button onclick={clearLogs} class="text-xs text-gray-500 hover:text-gray-400">
            {t("translate.clearLog")}
          </button>
        {/if}
      </div>

      <div class="grid grid-cols-2 gap-4 h-24 shrink-0">
        <!-- Original Subtitle -->
        <div class="bg-white/5 rounded-xl p-4 overflow-y-auto">
          <p class="text-xs text-gray-500 uppercase tracking-wide mb-2">{t("translate.original")}</p>
          {#if progress && currentSubtitleOriginal}
            <p class="text-gray-300">{currentSubtitleOriginal}</p>
          {:else}
            <p class="text-gray-600 text-sm">{t("translate.waitingForTranslation")}</p>
          {/if}
        </div>
        
        <!-- Translated Subtitle -->
        <div class="bg-white/5 rounded-xl p-4 overflow-y-auto">
          <p class="text-xs text-gray-500 uppercase tracking-wide mb-2">{t("translate.translated")}</p>
          {#if progress && currentSubtitleTranslated}
            <p class="text-green-300">{currentSubtitleTranslated}</p>
          {:else}
            <p class="text-gray-600 text-sm">{t("translate.waitingForTranslation")}</p>
          {/if}
        </div>
      </div>

      <!-- Log messages (scrollable, fills remaining space) -->
      <div class="mt-4 pt-4 border-t border-white/10 flex-1 min-h-0 overflow-y-auto">
        {#if logs.length > 0}
          <div class="space-y-1">
            {#each logs as log}
              <p class="text-gray-500 text-xs font-mono">{log}</p>
            {/each}
          </div>
        {:else}
          <p class="text-gray-600 text-xs">{t("translate.noLog")}</p>
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
