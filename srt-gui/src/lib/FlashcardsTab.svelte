<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { locale } from "./i18n";

  // Reactive translation
  let t = $derived($locale);

  // State - Files
  let targetSubsPath = $state("");  // Subtitle in target language (what you're learning)
  let nativeSubsPath = $state("");   // Subtitle in native language (optional)
  let videoPath = $state("");
  let outputDir = $state("");

  // Subtitle Options
  let useTimingsFrom = $state<"target" | "native">("target");
  let spanStart = $state("00:01:30");
  let spanEnd = $state("");
  let timeShiftTarget = $state(0);
  let timeShiftNative = $state(0);

  // Audio Options
  let generateAudio = $state(true);
  let audioBitrate = $state(128);
  let audioPadStart = $state(250);
  let audioPadEnd = $state(250);
  let normalizeAudio = $state(false);

  // Snapshot Options
  let generateSnapshots = $state(true);
  let snapshotWidth = $state(384);
  let snapshotHeight = $state(216);
  let cropBottom = $state(0);

  // Video Clip Options
  let generateVideoClips = $state(false);
  let videoBitrate = $state(800);
  let videoAudioBitrate = $state(128);
  let videoPadStart = $state(250);
  let videoPadEnd = $state(50);

  // Naming
  let deckName = $state("");
  let firstEpisode = $state(1);

  // Processing State
  let isProcessing = $state(false);
  let progress = $state(0);
  let progressMessage = $state("");
  let logs = $state<string[]>([]);
  let error = $state<string | null>(null);
  let result = $state<{ success: boolean; cardsGenerated: number } | null>(null);

  // File Info
  let targetSubsInfo = $state<{ count: number; first: string } | null>(null);
  let nativeSubsInfo = $state<{ count: number; first: string } | null>(null);

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    logs = [...logs, `[${timestamp}] ${message}`];
  }

  async function selectTargetSubs() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: t("flashcards.subtitleFiles"), extensions: ["srt", "ass", "ssa", "vtt"] }],
      });
      if (selected) {
        targetSubsPath = selected as string;
        addLog(`📄 ${t("flashcards.targetSubsLoaded")}: ${targetSubsPath.split('/').pop()}`);
        // TODO: Load and count subtitles
        targetSubsInfo = { count: 2194, first: "Welcome to..." };
        
        // Auto-generate deck name from filename
        if (!deckName) {
          const filename = targetSubsPath.split('/').pop() || "";
          deckName = filename.replace(/\.[^/.]+$/, "").replace(/_/g, " ");
        }
      }
    } catch (e) {
      error = `${t("flashcards.errorSelectingFile")}: ${e}`;
    }
  }

  async function selectNativeSubs() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: t("flashcards.subtitleFiles"), extensions: ["srt", "ass", "ssa", "vtt"] }],
      });
      if (selected) {
        nativeSubsPath = selected as string;
        addLog(`📄 ${t("flashcards.nativeSubsLoaded")}: ${nativeSubsPath.split('/').pop()}`);
        nativeSubsInfo = { count: 2194, first: "Benvenuto a..." };
      }
    } catch (e) {
      error = `${t("flashcards.errorSelectingFile")}: ${e}`;
    }
  }

  async function selectVideo() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: t("flashcards.videoFiles"), extensions: ["mp4", "mkv", "avi", "webm", "mov"] }],
      });
      if (selected) {
        videoPath = selected as string;
        addLog(`🎬 ${t("flashcards.videoLoaded")}: ${videoPath.split('/').pop()}`);
      }
    } catch (e) {
      error = `${t("flashcards.errorSelectingFile")}: ${e}`;
    }
  }

  async function selectOutputDir() {
    try {
      const selected = await open({
        directory: true,
      });
      if (selected) {
        outputDir = selected as string;
        addLog(`📁 ${t("flashcards.outputDirSet")}: ${outputDir}`);
      }
    } catch (e) {
      error = `${t("flashcards.errorSelectingDir")}: ${e}`;
    }
  }

  async function startGeneration() {
    if (!targetSubsPath || !outputDir || !deckName) {
      error = t("flashcards.requiredFieldsMissing");
      return;
    }

    error = null;
    result = null;
    progress = 0;
    isProcessing = true;
    addLog(`🚀 ${t("flashcards.starting")}...`);
    addLog(`📚 ${t("flashcards.deckName")}: ${deckName}`);

    // TODO: Implement actual flashcard generation
    // Simulated progress
    const interval = setInterval(() => {
      if (progress < 100) {
        progress += 2;
        progressMessage = `${t("flashcards.processing")}... ${progress}%`;
      } else {
        clearInterval(interval);
        isProcessing = false;
        result = { success: true, cardsGenerated: targetSubsInfo?.count || 0 };
        addLog(`✅ ${t("flashcards.completed")}: ${result.cardsGenerated} ${t("flashcards.cardsGenerated")}`);
      }
    }, 100);
  }

  async function cancelGeneration() {
    isProcessing = false;
    progress = 0;
    progressMessage = "";
    addLog(`⚠️ ${t("flashcards.cancelled")}`);
  }

  function clearLogs() {
    logs = [];
  }
</script>

<div class="h-full flex flex-col p-6 overflow-hidden bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950">
  <!-- Header -->
  <div class="mb-4 shrink-0">
    <div class="flex items-center gap-3">
      <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-emerald-500 to-teal-600 flex items-center justify-center shadow-lg">
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
      </div>
      <div>
        <h2 class="text-3xl font-bold bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent">
          {t("flashcards.title")}
        </h2>
        <p class="text-gray-400 mt-1">
          {t("flashcards.subtitle")}
        </p>
      </div>
    </div>
  </div>

  <!-- Main Content: 3 Columns -->
  <div class="flex-1 grid grid-cols-12 gap-4 min-h-0 overflow-y-auto">
    
    <!-- Column 1: Files -->
    <div class="col-span-4 space-y-4">
      <div class="glass-card p-4">
        <h3 class="text-sm font-semibold mb-3 flex items-center gap-2 text-emerald-400">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
          </svg>
          {t("flashcards.files")}
        </h3>

        <div class="space-y-3">
          <!-- Target Language Subs -->
          <div>
            <label class="block text-xs text-gray-400 mb-1">
              {t("flashcards.targetLangSubs")} <span class="text-red-400">*</span>
            </label>
            <div class="flex gap-2">
              <button onclick={selectTargetSubs} class="btn-primary py-1.5 px-3 text-xs flex-shrink-0">
                {t("flashcards.subs1")}...
              </button>
              <input type="text" value={targetSubsPath.split('/').pop() || ""} readonly
                class="input-modern flex-1 text-xs" placeholder={t("flashcards.selectFile")} />
            </div>
          </div>

          <!-- Native Language Subs -->
          <div>
            <label class="block text-xs text-gray-400 mb-1">{t("flashcards.nativeLangSubs")}</label>
            <div class="flex gap-2">
              <button onclick={selectNativeSubs} class="btn-secondary py-1.5 px-3 text-xs flex-shrink-0">
                {t("flashcards.subs2")}...
              </button>
              <input type="text" value={nativeSubsPath.split('/').pop() || ""} readonly
                class="input-modern flex-1 text-xs" placeholder={t("flashcards.optional")} />
            </div>
          </div>

          <!-- Video File -->
          <div>
            <label class="block text-xs text-gray-400 mb-1">{t("flashcards.videoFile")}</label>
            <div class="flex gap-2">
              <button onclick={selectVideo} class="btn-secondary py-1.5 px-3 text-xs flex-shrink-0">
                {t("flashcards.video")}...
              </button>
              <input type="text" value={videoPath.split('/').pop() || ""} readonly
                class="input-modern flex-1 text-xs" placeholder={t("flashcards.optionalForMedia")} />
            </div>
          </div>

          <!-- Output Directory -->
          <div>
            <label class="block text-xs text-gray-400 mb-1">
              {t("flashcards.outputDir")} <span class="text-red-400">*</span>
            </label>
            <div class="flex gap-2">
              <button onclick={selectOutputDir} class="btn-primary py-1.5 px-3 text-xs flex-shrink-0">
                {t("flashcards.output")}...
              </button>
              <input type="text" value={outputDir.split('/').pop() || ""} readonly
                class="input-modern flex-1 text-xs" placeholder={t("flashcards.selectDir")} />
            </div>
          </div>
        </div>

        <!-- Subtitle Info -->
        {#if targetSubsInfo}
          <div class="mt-3 p-2 bg-emerald-500/10 border border-emerald-500/20 rounded-lg">
            <p class="text-xs text-emerald-300">
              ✓ {targetSubsInfo.count} {t("flashcards.subtitlesLoaded")}
            </p>
          </div>
        {/if}
      </div>

      <!-- Subtitle Options -->
      <div class="glass-card p-4">
        <h3 class="text-sm font-semibold mb-3 text-gray-300">{t("flashcards.subtitleOptions")}</h3>
        
        <div class="space-y-3">
          <!-- Use Timings From -->
          <div class="flex items-center gap-4">
            <span class="text-xs text-gray-400">{t("flashcards.useTimingsFrom")}:</span>
            <label class="flex items-center gap-1.5">
              <input type="radio" bind:group={useTimingsFrom} value="target" class="text-emerald-500" />
              <span class="text-xs text-gray-300">{t("flashcards.subs1")}</span>
            </label>
            <label class="flex items-center gap-1.5">
              <input type="radio" bind:group={useTimingsFrom} value="native" class="text-emerald-500" 
                disabled={!nativeSubsPath} />
              <span class="text-xs text-gray-300 {!nativeSubsPath ? 'opacity-50' : ''}">{t("flashcards.subs2")}</span>
            </label>
          </div>

          <!-- Span -->
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="block text-xs text-gray-500 mb-1">{t("flashcards.spanStart")}</label>
              <input type="text" bind:value={spanStart} class="input-modern w-full text-xs" placeholder="h:mm:ss" />
            </div>
            <div>
              <label class="block text-xs text-gray-500 mb-1">{t("flashcards.spanEnd")}</label>
              <input type="text" bind:value={spanEnd} class="input-modern w-full text-xs" placeholder="h:mm:ss" />
            </div>
          </div>

          <!-- Time Shift -->
          <div class="grid grid-cols-2 gap-2">
            <div>
              <label class="block text-xs text-gray-500 mb-1">{t("flashcards.timeShift")} {t("flashcards.subs1")}</label>
              <div class="flex items-center gap-1">
                <input type="number" bind:value={timeShiftTarget} class="input-modern w-full text-xs" />
                <span class="text-xs text-gray-500">ms</span>
              </div>
            </div>
            <div>
              <label class="block text-xs text-gray-500 mb-1">{t("flashcards.timeShift")} {t("flashcards.subs2")}</label>
              <div class="flex items-center gap-1">
                <input type="number" bind:value={timeShiftNative} class="input-modern w-full text-xs" 
                  disabled={!nativeSubsPath} />
                <span class="text-xs text-gray-500">ms</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Column 2: Media Options -->
    <div class="col-span-4 space-y-4">
      <!-- Audio Clips -->
      <div class="glass-card p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold flex items-center gap-2 text-cyan-400">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
            </svg>
            {t("flashcards.generateAudioClips")}
          </h3>
          <button onclick={() => generateAudio = !generateAudio}
            class="w-10 h-5 rounded-full transition-all duration-200 relative
              {generateAudio ? 'bg-cyan-500' : 'bg-gray-600'}">
            <div class="absolute w-4 h-4 bg-white rounded-full top-0.5 transition-all duration-200
              {generateAudio ? 'left-5' : 'left-0.5'}"></div>
          </button>
        </div>

        {#if generateAudio}
          <div class="space-y-2 animate-fade-in">
            <div class="grid grid-cols-2 gap-2">
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.bitrate")}</label>
                <div class="relative">
                  <select bind:value={audioBitrate} class="select-modern w-full text-xs appearance-none pr-8">
                    <option value={64}>64 kb/s</option>
                    <option value={128}>128 kb/s</option>
                    <option value={192}>192 kb/s</option>
                    <option value={256}>256 kb/s</option>
                  </select>
                </div>
              </div>
              <div class="flex items-end">
                <label class="flex items-center gap-1.5">
                  <input type="checkbox" bind:checked={normalizeAudio} class="rounded text-cyan-500" />
                  <span class="text-xs text-gray-300">{t("flashcards.normalizeAudio")}</span>
                </label>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-2">
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.padStart")}</label>
                <div class="flex items-center gap-1">
                  <input type="number" bind:value={audioPadStart} class="input-modern w-full text-xs" />
                  <span class="text-xs text-gray-500">ms</span>
                </div>
              </div>
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.padEnd")}</label>
                <div class="flex items-center gap-1">
                  <input type="number" bind:value={audioPadEnd} class="input-modern w-full text-xs" />
                  <span class="text-xs text-gray-500">ms</span>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Snapshots -->
      <div class="glass-card p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold flex items-center gap-2 text-purple-400">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            {t("flashcards.generateSnapshots")}
          </h3>
          <button onclick={() => generateSnapshots = !generateSnapshots}
            class="w-10 h-5 rounded-full transition-all duration-200 relative
              {generateSnapshots ? 'bg-purple-500' : 'bg-gray-600'}">
            <div class="absolute w-4 h-4 bg-white rounded-full top-0.5 transition-all duration-200
              {generateSnapshots ? 'left-5' : 'left-0.5'}"></div>
          </button>
        </div>

        {#if generateSnapshots}
          <div class="space-y-2 animate-fade-in">
            <div class="grid grid-cols-3 gap-2">
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.width")}</label>
                <div class="flex items-center gap-1">
                  <input type="number" bind:value={snapshotWidth} class="input-modern w-full text-xs" />
                  <span class="text-xs text-gray-500">px</span>
                </div>
              </div>
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.height")}</label>
                <div class="flex items-center gap-1">
                  <input type="number" bind:value={snapshotHeight} class="input-modern w-full text-xs" />
                  <span class="text-xs text-gray-500">px</span>
                </div>
              </div>
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.cropBottom")}</label>
                <div class="flex items-center gap-1">
                  <input type="number" bind:value={cropBottom} class="input-modern w-full text-xs" />
                  <span class="text-xs text-gray-500">px</span>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Video Clips -->
      <div class="glass-card p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold flex items-center gap-2 text-rose-400">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
            {t("flashcards.generateVideoClips")}
          </h3>
          <button onclick={() => generateVideoClips = !generateVideoClips}
            class="w-10 h-5 rounded-full transition-all duration-200 relative
              {generateVideoClips ? 'bg-rose-500' : 'bg-gray-600'}">
            <div class="absolute w-4 h-4 bg-white rounded-full top-0.5 transition-all duration-200
              {generateVideoClips ? 'left-5' : 'left-0.5'}"></div>
          </button>
        </div>

        {#if generateVideoClips}
          <div class="space-y-2 animate-fade-in">
            <div class="grid grid-cols-2 gap-2">
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.videoBitrate")}</label>
                <div class="flex items-center gap-1">
                  <input type="number" bind:value={videoBitrate} class="input-modern w-full text-xs" />
                  <span class="text-xs text-gray-500">kb/s</span>
                </div>
              </div>
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.audioBitrate")}</label>
                <div class="relative">
                  <select bind:value={videoAudioBitrate} class="select-modern w-full text-xs appearance-none pr-8">
                    <option value={64}>64 kb/s</option>
                    <option value={128}>128 kb/s</option>
                    <option value={192}>192 kb/s</option>
                  </select>
                </div>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-2">
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.padStart")}</label>
                <div class="flex items-center gap-1">
                  <input type="number" bind:value={videoPadStart} class="input-modern w-full text-xs" />
                  <span class="text-xs text-gray-500">ms</span>
                </div>
              </div>
              <div>
                <label class="block text-xs text-gray-500 mb-1">{t("flashcards.padEnd")}</label>
                <div class="flex items-center gap-1">
                  <input type="number" bind:value={videoPadEnd} class="input-modern w-full text-xs" />
                  <span class="text-xs text-gray-500">ms</span>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Column 3: Naming & Action -->
    <div class="col-span-4 space-y-4">
      <!-- Naming -->
      <div class="glass-card p-4">
        <h3 class="text-sm font-semibold mb-3 flex items-center gap-2 text-amber-400">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
          </svg>
          {t("flashcards.naming")}
        </h3>

        <div class="space-y-3">
          <div>
            <label class="block text-xs text-gray-400 mb-1">
              {t("flashcards.deckNameLabel")} <span class="text-red-400">*</span>
            </label>
            <input type="text" bind:value={deckName} class="input-modern w-full text-sm"
              placeholder={t("flashcards.deckNamePlaceholder")} />
          </div>
          <div>
            <label class="block text-xs text-gray-400 mb-1">{t("flashcards.firstEpisode")}</label>
            <input type="number" bind:value={firstEpisode} min="1" class="input-modern w-20 text-sm" />
          </div>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="space-y-3">
        {#if isProcessing}
          <button onclick={cancelGeneration} class="btn-danger w-full py-4 text-lg">
            <svg class="w-5 h-5 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
            {t("flashcards.cancel")}
          </button>
        {:else}
          <button onclick={startGeneration}
            disabled={!targetSubsPath || !outputDir || !deckName}
            class="btn-success w-full py-4 text-lg disabled:opacity-50 disabled:cursor-not-allowed">
            <svg class="w-5 h-5 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            {t("flashcards.generate")}
          </button>
        {/if}

        <button class="btn-secondary w-full py-2" disabled={!targetSubsPath || !videoPath}>
          <svg class="w-4 h-4 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
          {t("flashcards.preview")}
        </button>
      </div>

      <!-- Progress -->
      {#if isProcessing || progress > 0}
        <div class="glass-card p-4 {isProcessing ? 'animate-pulse-glow' : ''}">
          <div class="flex items-center gap-4">
            <div class="flex-1">
              <div class="progress-modern h-2">
                <div class="progress-modern-bar bg-gradient-to-r from-emerald-500 to-teal-500" style="width: {progress}%"></div>
              </div>
            </div>
            <span class="text-lg font-bold text-emerald-400">{progress}%</span>
          </div>
          {#if progressMessage}
            <p class="text-gray-400 text-xs mt-2">{progressMessage}</p>
          {/if}
        </div>
      {/if}

      <!-- Result -->
      {#if result}
        <div class="glass-card p-4 border-l-4 {result.success ? 'border-green-500 bg-green-500/5' : 'border-red-500 bg-red-500/5'}">
          <div class="flex items-center gap-3">
            <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            <p class="text-green-400 font-medium">
              {result.cardsGenerated} {t("flashcards.cardsGenerated")}
            </p>
          </div>
        </div>
      {/if}

      <!-- Error -->
      {#if error}
        <div class="glass-card p-4 border border-red-500/30 bg-red-500/10">
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
      <div class="glass-card p-4 flex-1 flex flex-col min-h-32">
        <div class="flex items-center justify-between mb-2">
          <h4 class="text-xs font-semibold text-gray-400">{t("flashcards.log")}</h4>
          {#if logs.length > 0}
            <button onclick={clearLogs} class="text-xs text-gray-500 hover:text-gray-400">
              {t("flashcards.clearLog")}
            </button>
          {/if}
        </div>
        <div class="flex-1 overflow-y-auto bg-black/20 rounded-lg p-2 max-h-32">
          {#if logs.length > 0}
            {#each logs as log}
              <p class="text-gray-400 text-xs font-mono">{log}</p>
            {/each}
          {:else}
            <p class="text-gray-600 text-xs">{t("flashcards.noLog")}</p>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>
