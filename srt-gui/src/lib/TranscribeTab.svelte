<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount, onDestroy } from "svelte";
  import { locale } from "./i18n";

  // Reactive translation
  let t = $derived($locale);

  // State
  let inputPath = $state("");
  let outputPath = $state("");
  let selectedModel = $state("base");
  let selectedLanguage = $state("auto");
  let translateToEnglish = $state(false);
  let wordTimestamps = $state(true);
  let maxSegmentLength = $state(30);
  
  let isTranscribing = $state(false);
  let progress = $state(0);
  let progressMessage = $state("");
  let logs = $state<string[]>([]);
  let error = $state<string | null>(null);
  let result = $state<{ success: boolean; message: string } | null>(null);

  // Whisper models
  const whisperModels = [
    { id: "tiny", name: "Tiny", size: "~75MB", speed: "~32x" },
    { id: "base", name: "Base", size: "~150MB", speed: "~16x" },
    { id: "small", name: "Small", size: "~500MB", speed: "~6x" },
    { id: "medium", name: "Medium", size: "~1.5GB", speed: "~2x" },
    { id: "large", name: "Large", size: "~3GB", speed: "~1x" },
  ];

  // Languages for transcription
  const transcriptionLanguages = [
    { code: "auto", name: "Auto-detect" },
    { code: "en", name: "English" },
    { code: "it", name: "Italiano" },
    { code: "es", name: "Español" },
    { code: "fr", name: "Français" },
    { code: "de", name: "Deutsch" },
    { code: "pt", name: "Português" },
    { code: "ru", name: "Русский" },
    { code: "zh", name: "中文" },
    { code: "ja", name: "日本語" },
    { code: "ko", name: "한국어" },
    { code: "ar", name: "العربية" },
  ];

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    logs = [...logs, `[${timestamp}] ${message}`];
  }

  async function selectInputFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: t("transcribe.audioVideoFiles"), extensions: ["mp4", "mkv", "avi", "webm", "mp3", "wav", "m4a", "flac", "ogg"] }
        ],
      });

      if (selected) {
        inputPath = selected as string;
        
        // Auto-generate output path
        if (!outputPath) {
          const basePath = inputPath.replace(/\.[^/.]+$/, "");
          outputPath = `${basePath}.srt`;
        }
        addLog(`📁 ${t("transcribe.fileSelected")}: ${inputPath.split('/').pop()}`);
      }
    } catch (e) {
      error = `${t("transcribe.errorSelectingFile")}: ${e}`;
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
      error = `${t("transcribe.errorSelectingFile")}: ${e}`;
    }
  }

  async function startTranscription() {
    if (!inputPath || !outputPath) {
      error = t("transcribe.selectFilesFirst");
      return;
    }

    error = null;
    result = null;
    progress = 0;
    isTranscribing = true;
    addLog(`🚀 ${t("transcribe.starting")} (${selectedModel})`);

    // TODO: Implement actual Whisper transcription backend
    // Simulated progress for now
    const interval = setInterval(() => {
      if (progress < 100) {
        progress += 5;
        progressMessage = `${t("transcribe.processing")}... ${progress}%`;
      } else {
        clearInterval(interval);
        isTranscribing = false;
        result = { success: true, message: t("transcribe.completed") };
        addLog(`✅ ${t("transcribe.completed")}`);
      }
    }, 500);
  }

  async function cancelTranscription() {
    isTranscribing = false;
    progress = 0;
    progressMessage = "";
    addLog(`⚠️ ${t("transcribe.cancelled")}`);
  }

  function clearLogs() {
    logs = [];
  }
</script>

<div class="h-full flex flex-col p-6 overflow-hidden bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950">
  <!-- Header -->
  <div class="mb-6 shrink-0">
    <div class="flex items-center gap-3">
      <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center shadow-lg">
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
        </svg>
      </div>
      <div>
        <h2 class="text-3xl font-bold bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent">
          {t("transcribe.title")}
        </h2>
        <p class="text-gray-400 mt-1">
          {t("transcribe.subtitle")}
        </p>
      </div>
    </div>
  </div>

  <!-- Main Grid -->
  <div class="grid grid-cols-2 gap-6 shrink-0">
    <!-- Left Column: Options -->
    <div class="space-y-4">
      <!-- Whisper Model Selection -->
      <div class="glass-card p-5">
        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
          <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
          {t("transcribe.whisperModel")}
        </h3>

        <div class="grid grid-cols-5 gap-2">
          {#each whisperModels as model}
            <button
              onclick={() => selectedModel = model.id}
              class="p-3 rounded-lg text-center transition-all duration-200 border
                {selectedModel === model.id 
                  ? 'bg-cyan-500/20 border-cyan-500/50 text-white' 
                  : 'bg-white/5 hover:bg-white/10 border-transparent text-gray-400 hover:text-white'}"
            >
              <div class="font-bold text-sm">{model.name}</div>
              <div class="text-[10px] text-gray-500 mt-1">{model.size}</div>
            </button>
          {/each}
        </div>
        
        <div class="mt-3 p-3 bg-cyan-500/10 rounded-lg border border-cyan-500/20">
          <div class="flex items-center gap-2">
            <span class="text-cyan-400">💡</span>
            <span class="text-sm text-cyan-200">
              {t("transcribe.speed")}: {whisperModels.find(m => m.id === selectedModel)?.speed || ""}
            </span>
          </div>
        </div>
      </div>

      <!-- Transcription Options -->
      <div class="glass-card p-5">
        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
          <svg class="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          {t("transcribe.options")}
        </h3>

        <div class="space-y-4">
          <!-- Language Selection -->
          <div>
            <label class="block text-sm text-gray-400 mb-1">{t("transcribe.sourceLanguage")}</label>
            <div class="relative">
              <select bind:value={selectedLanguage} class="select-modern w-full text-sm appearance-none pr-10">
                {#each transcriptionLanguages as lang}
                  <option value={lang.code}>{lang.name}</option>
                {/each}
              </select>
              <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-gray-400">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </div>
            </div>
          </div>

          <!-- Word Timestamps Toggle -->
          <div class="flex items-center justify-between p-3 bg-white/5 rounded-lg">
            <div>
              <span class="text-gray-200 text-sm">{t("transcribe.wordTimestamps")}</span>
              <p class="text-xs text-gray-500">{t("transcribe.wordTimestampsDesc")}</p>
            </div>
            <button
              onclick={() => wordTimestamps = !wordTimestamps}
              class="w-12 h-6 rounded-full transition-all duration-200 relative
                {wordTimestamps ? 'bg-cyan-500' : 'bg-gray-600'}"
            >
              <div class="absolute w-5 h-5 bg-white rounded-full top-0.5 transition-all duration-200
                {wordTimestamps ? 'left-6' : 'left-0.5'}"></div>
            </button>
          </div>

          <!-- Translate to English Toggle -->
          <div class="flex items-center justify-between p-3 bg-white/5 rounded-lg">
            <div>
              <span class="text-gray-200 text-sm">{t("transcribe.translateToEnglish")}</span>
              <p class="text-xs text-gray-500">{t("transcribe.translateToEnglishDesc")}</p>
            </div>
            <button
              onclick={() => translateToEnglish = !translateToEnglish}
              class="w-12 h-6 rounded-full transition-all duration-200 relative
                {translateToEnglish ? 'bg-cyan-500' : 'bg-gray-600'}"
            >
              <div class="absolute w-5 h-5 bg-white rounded-full top-0.5 transition-all duration-200
                {translateToEnglish ? 'left-6' : 'left-0.5'}"></div>
            </button>
          </div>

          <!-- Max Segment Length -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <label class="text-sm text-gray-400">{t("transcribe.maxSegmentLength")}</label>
              <span class="text-white font-mono bg-white/10 px-2 py-0.5 rounded text-sm">{maxSegmentLength}s</span>
            </div>
            <input
              type="range"
              bind:value={maxSegmentLength}
              min="10"
              max="60"
              step="5"
              class="w-full"
            />
            <div class="flex justify-between text-xs text-gray-500 mt-1">
              <span>10s</span>
              <span>60s</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Right Column: File Selection + Action -->
    <div class="space-y-4">
      <!-- File Input Card -->
      <div class="glass-card p-5">
        <h3 class="text-lg font-semibold mb-4 flex items-center gap-2">
          <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
          </svg>
          {t("transcribe.files")}
        </h3>

        <div class="space-y-3">
          <div>
            <label class="block text-sm text-gray-400 mb-1">{t("transcribe.inputFile")}</label>
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={inputPath}
                placeholder={t("transcribe.selectAudioVideo")}
                class="input-modern flex-1 text-sm"
                readonly
              />
              <button 
                onclick={selectInputFile} 
                class="btn-primary py-2 px-3"
                title={t("transcribe.selectFile")}
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                </svg>
              </button>
            </div>
          </div>

          <div>
            <label class="block text-sm text-gray-400 mb-1">{t("transcribe.outputFile")}</label>
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={outputPath}
                placeholder={t("transcribe.selectDestination")}
                class="input-modern flex-1 text-sm"
              />
              <button 
                onclick={selectOutputFile} 
                class="btn-secondary py-2 px-3"
                title={t("transcribe.selectDestination")}
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                </svg>
              </button>
            </div>
          </div>

          {#if inputPath}
            <div class="p-3 bg-cyan-500/10 border border-cyan-500/30 rounded-lg">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-cyan-500/20 flex items-center justify-center">
                  <span class="text-xl">🎬</span>
                </div>
                <div>
                  <p class="font-medium text-white">{inputPath.split('/').pop()}</p>
                  <p class="text-sm text-gray-400">{t("transcribe.readyToProcess")}</p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Action Button -->
      <div class="flex gap-3">
        {#if isTranscribing}
          <button onclick={cancelTranscription} class="btn-danger flex-1 py-4 text-lg">
            <svg class="w-5 h-5 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
            {t("transcribe.cancel")}
          </button>
        {:else}
          <button
            onclick={startTranscription}
            disabled={!inputPath || !outputPath}
            class="btn-success flex-1 py-4 text-lg disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <svg class="w-5 h-5 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            {t("transcribe.startTranscription")}
          </button>
        {/if}
      </div>

      <!-- Model Download Info -->
      <div class="p-3 bg-amber-500/10 border border-amber-500/30 rounded-lg">
        <div class="flex items-start gap-2">
          <span class="text-amber-400">⚠️</span>
          <p class="text-xs text-amber-200">
            {t("transcribe.modelDownloadNote")}
          </p>
        </div>
      </div>
    </div>
  </div>

  <!-- Bottom Section: Progress + Logs -->
  <div class="mt-6 flex-1 flex flex-col space-y-4 min-h-0">
    <!-- Progress -->
    {#if isTranscribing || progress > 0}
      <div class="glass-card p-4 shrink-0 {isTranscribing ? 'animate-pulse-glow' : ''}">
        <div class="flex items-center gap-6">
          <div class="flex-1">
            <div class="progress-modern h-2">
              <div class="progress-modern-bar" style="width: {progress}%"></div>
            </div>
          </div>
          <span class="text-xl font-bold bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent">
            {progress}%
          </span>
        </div>
        {#if progressMessage}
          <p class="text-gray-400 text-sm mt-2">{progressMessage}</p>
        {/if}
      </div>
    {/if}

    <!-- Result -->
    {#if result}
      <div class="glass-card p-4 shrink-0 border-l-4 {result.success ? 'border-green-500 bg-green-500/5' : 'border-red-500 bg-red-500/5'}">
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
          <p class="{result.success ? 'text-green-400' : 'text-red-400'} font-medium">{result.message}</p>
        </div>
      </div>
    {/if}

    <!-- Error -->
    {#if error}
      <div class="glass-card p-4 shrink-0 border border-red-500/30 bg-red-500/10">
        <div class="flex items-center gap-3">
          <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-red-300 flex-1 text-sm">{error}</p>
          <button onclick={() => (error = null)} class="text-red-400 hover:text-red-300">✕</button>
        </div>
      </div>
    {/if}

    <!-- Logs -->
    <div class="glass-card p-5 flex-1 flex flex-col min-h-0">
      <div class="flex items-center justify-between mb-4 shrink-0">
        <h3 class="text-lg font-semibold flex items-center gap-2">
          <svg class="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          {t("transcribe.log")}
        </h3>
        {#if logs.length > 0}
          <button onclick={clearLogs} class="text-xs text-gray-500 hover:text-gray-400">
            {t("transcribe.clearLog")}
          </button>
        {/if}
      </div>

      <div class="flex-1 min-h-0 overflow-y-auto bg-black/20 rounded-lg p-3">
        {#if logs.length > 0}
          <div class="space-y-1">
            {#each logs as log}
              <p class="text-gray-400 text-xs font-mono">{log}</p>
            {/each}
          </div>
        {:else}
          <p class="text-gray-600 text-xs">{t("transcribe.noLog")}</p>
        {/if}
      </div>
    </div>
  </div>
</div>
