<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  // Types
  interface SubtitleInfo {
    id: number;
    start_ms: number;
    end_ms: number;
    text: string;
    synced_start_ms: number;
    synced_end_ms: number;
    offset_ms: number;
    is_anchor: boolean;
  }

  interface SyncStatus {
    is_loaded: boolean;
    srt_path: string | null;
    video_path: string | null;
    total_subtitles: number;
    anchor_count: number;
    checked_count: number;
    completion_percentage: number;
    average_offset_ms: number;
    suggested_next_id: number | null;
  }

  interface AnchorInfo {
    subtitle_id: number;
    original_time_ms: number;
    corrected_time_ms: number;
    offset_ms: number;
  }

  // State
  let videoElement = $state<HTMLVideoElement | null>(null);
  let status = $state<SyncStatus | null>(null);
  let subtitles = $state<SubtitleInfo[]>([]);
  let anchors = $state<AnchorInfo[]>([]);
  let currentSubtitle = $state<SubtitleInfo | null>(null);
  let currentVideoTime = $state(0);
  let isPlaying = $state(false);
  let error = $state<string | null>(null);
  let videoSrc = $state<string | null>(null);

  // Offset adjustment
  let offsetAdjustment = $state(0);

  // Computed: current subtitle based on video time
  let activeSubtitleId = $derived.by(() => {
    const time = currentVideoTime * 1000; // Convert to ms
    for (const sub of subtitles) {
      if (time >= sub.synced_start_ms && time <= sub.synced_end_ms) {
        return sub.id;
      }
    }
    return null;
  });

  // Update current video time
  function onTimeUpdate() {
    if (videoElement) {
      currentVideoTime = videoElement.currentTime;
    }
  }

  async function selectSrtFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "SRT Files", extensions: ["srt"] }],
      });

      if (selected) {
        status = await invoke<SyncStatus>("sync_load_srt", {
          path: selected as string,
        });
        await loadSubtitles();
        await loadAnchors();
      }
    } catch (e) {
      error = `Errore caricamento SRT: ${e}`;
    }
  }

  async function selectVideoFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: "Video Files", extensions: ["mp4", "mkv", "avi", "webm", "mov", "m4v"] },
        ],
      });

      if (selected) {
        const path = selected as string;
        // Use convertFileSrc for proper Tauri asset loading
        videoSrc = convertFileSrc(path);
        status = await invoke<SyncStatus>("sync_set_video", { path });
      }
    } catch (e) {
      error = `Errore caricamento video: ${e}`;
    }
  }

  async function loadSubtitles() {
    try {
      subtitles = await invoke<SubtitleInfo[]>("sync_get_subtitles");
    } catch (e) {
      error = `Errore caricamento sottotitoli: ${e}`;
    }
  }

  async function loadAnchors() {
    try {
      anchors = await invoke<AnchorInfo[]>("sync_get_anchors");
    } catch (e) {
      error = `Errore caricamento ancore: ${e}`;
    }
  }

  async function refreshStatus() {
    try {
      status = await invoke<SyncStatus>("sync_get_status");
      await loadSubtitles();
      await loadAnchors();
    } catch (e) {
      error = `Errore aggiornamento stato: ${e}`;
    }
  }

  async function addAnchor() {
    if (!currentSubtitle) return;

    const correctedTime =
      currentSubtitle.start_ms + offsetAdjustment + currentVideoTime * 1000;

    try {
      status = await invoke<SyncStatus>("sync_add_anchor", {
        subtitleId: currentSubtitle.id,
        correctedTimeMs: Math.round(correctedTime),
      });
      await loadSubtitles();
      await loadAnchors();
      offsetAdjustment = 0;
    } catch (e) {
      error = `Errore aggiunta ancora: ${e}`;
    }
  }

  async function confirmAtCurrentTime() {
    if (activeSubtitleId === null) return;

    const videoTimeMs = currentVideoTime * 1000;

    try {
      status = await invoke<SyncStatus>("sync_add_anchor", {
        subtitleId: activeSubtitleId,
        correctedTimeMs: Math.round(videoTimeMs),
      });
      await loadSubtitles();
      await loadAnchors();
    } catch (e) {
      error = `Errore conferma ancora: ${e}`;
    }
  }

  async function removeAnchor(subtitleId: number) {
    try {
      status = await invoke<SyncStatus>("sync_remove_anchor", {
        subtitleId,
      });
      await loadSubtitles();
      await loadAnchors();
    } catch (e) {
      error = `Errore rimozione ancora: ${e}`;
    }
  }

  async function goToSuggested() {
    if (!status?.suggested_next_id) return;

    const sub = subtitles.find((s) => s.id === status?.suggested_next_id);
    if (sub && videoElement) {
      videoElement.currentTime = sub.synced_start_ms / 1000;
      currentSubtitle = sub;
    }
  }

  async function saveFile() {
    try {
      const selected = await save({
        filters: [{ name: "SRT Files", extensions: ["srt"] }],
        defaultPath: status?.srt_path?.replace(".srt", ".synced.srt"),
      });

      if (selected) {
        await invoke<string>("sync_save_file", { outputPath: selected });
        alert(`File salvato: ${selected}`);
      }
    } catch (e) {
      error = `Errore salvataggio: ${e}`;
    }
  }

  async function saveSession() {
    try {
      const selected = await save({
        filters: [{ name: "Session Files", extensions: ["json"] }],
      });

      if (selected) {
        await invoke<string>("sync_save_session", { sessionPath: selected });
        alert(`Sessione salvata: ${selected}`);
      }
    } catch (e) {
      error = `Errore salvataggio sessione: ${e}`;
    }
  }

  async function loadSession() {
    try {
      const selected = await open({
        filters: [{ name: "Session Files", extensions: ["json"] }],
      });

      if (selected) {
        status = await invoke<SyncStatus>("sync_load_session", {
          sessionPath: selected as string,
        });
        await loadSubtitles();
        await loadAnchors();
      }
    } catch (e) {
      error = `Errore caricamento sessione: ${e}`;
    }
  }

  async function resetSync() {
    if (!confirm("Sei sicuro di voler resettare la sincronizzazione?")) return;

    try {
      status = await invoke<SyncStatus>("sync_reset");
      await loadSubtitles();
      await loadAnchors();
    } catch (e) {
      error = `Errore reset: ${e}`;
    }
  }

  function formatTime(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;
    const millis = Math.floor(ms % 1000);

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}.${millis.toString().padStart(3, "0")}`;
    }
    return `${minutes}:${seconds.toString().padStart(2, "0")}.${millis.toString().padStart(3, "0")}`;
  }

  function formatOffset(ms: number): string {
    const sign = ms >= 0 ? "+" : "";
    return `${sign}${(ms / 1000).toFixed(2)}s`;
  }

  function goToSubtitle(sub: SubtitleInfo) {
    currentSubtitle = sub;
    if (videoElement) {
      videoElement.currentTime = sub.synced_start_ms / 1000;
    }
  }

  // Keyboard shortcuts
  function handleKeydown(e: KeyboardEvent) {
    if (!videoElement) return;

    switch (e.key) {
      case " ":
        e.preventDefault();
        if (isPlaying) {
          videoElement.pause();
        } else {
          videoElement.play();
        }
        break;
      case "ArrowLeft":
        e.preventDefault();
        videoElement.currentTime -= e.shiftKey ? 1 : 0.1;
        break;
      case "ArrowRight":
        e.preventDefault();
        videoElement.currentTime += e.shiftKey ? 1 : 0.1;
        break;
      case "ArrowUp":
        e.preventDefault();
        offsetAdjustment += e.shiftKey ? 500 : 100;
        break;
      case "ArrowDown":
        e.preventDefault();
        offsetAdjustment -= e.shiftKey ? 500 : 100;
        break;
      case "Enter":
        e.preventDefault();
        confirmAtCurrentTime();
        break;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="h-full flex flex-col overflow-hidden bg-gradient-to-br from-gray-900 via-gray-950 to-gray-900">
  <!-- Top Bar -->
  <div class="flex items-center gap-4 p-4 glass-card m-4 mb-0 flex-shrink-0">
    <button
      onclick={selectSrtFile}
      class="btn-primary py-2 px-4 flex items-center gap-2"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      Carica SRT
    </button>
    <button
      onclick={selectVideoFile}
      class="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 py-2 px-4 rounded-xl font-medium flex items-center gap-2 transition-all shadow-lg shadow-purple-500/30"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
      </svg>
      Carica Video
    </button>

    <div class="flex-1"></div>

    <button
      onclick={loadSession}
      class="btn-secondary py-2 px-4 flex items-center gap-2"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
      </svg>
      Carica Sessione
    </button>
    <button
      onclick={saveSession}
      disabled={!status?.is_loaded}
      class="btn-secondary py-2 px-4 flex items-center gap-2 disabled:opacity-50"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
      </svg>
      Salva Sessione
    </button>
    <button
      onclick={saveFile}
      disabled={!status?.is_loaded}
      class="btn-success py-2 px-4 flex items-center gap-2 disabled:opacity-50"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
      </svg>
      Esporta SRT
    </button>
  </div>

  <!-- Main Content -->
  <div class="flex-1 flex overflow-hidden p-4 pt-4 gap-4">
    <!-- Left: Video + Controls -->
    <div class="w-2/3 flex flex-col gap-4">
      <!-- Video Player -->
      <div class="flex-1 glass-card relative flex items-center justify-center overflow-hidden">
        {#if videoSrc}
          <video
            bind:this={videoElement}
            src={videoSrc}
            class="max-w-full max-h-full"
            ontimeupdate={onTimeUpdate}
            onplay={() => (isPlaying = true)}
            onpause={() => (isPlaying = false)}
          >
            <track kind="captions" />
          </video>

          <!-- Subtitle Overlay -->
          {#if activeSubtitleId !== null}
            {@const activeSub = subtitles.find(
              (s) => s.id === activeSubtitleId
            )}
            {#if activeSub}
              <div class="absolute bottom-20 left-0 right-0 text-center px-4">
                <p class="inline-block bg-black/80 backdrop-blur-sm px-6 py-3 rounded-xl text-xl text-white shadow-lg">
                  {activeSub.text}
                </p>
              </div>
            {/if}
          {/if}
        {:else}
          <div class="text-gray-500 text-center">
            <div class="w-20 h-20 mx-auto mb-4 rounded-2xl bg-white/5 flex items-center justify-center">
              <svg class="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </div>
            <p class="text-lg">Carica un video per iniziare</p>
            <p class="text-sm text-gray-600 mt-1">Formati supportati: MP4, MKV, AVI, WebM</p>
          </div>
        {/if}
      </div>

      <!-- Video Controls -->
      <div class="glass-card p-4 space-y-4 flex-shrink-0">
        <!-- Timeline -->
        <div class="flex items-center gap-4">
          <span class="text-sm text-gray-400 font-mono w-24">
            {formatTime(currentVideoTime * 1000)}
          </span>
          <input
            type="range"
            min="0"
            max={videoElement?.duration || 100}
            bind:value={currentVideoTime}
            oninput={() => {
              if (videoElement) videoElement.currentTime = currentVideoTime;
            }}
            class="flex-1"
          />
          <span class="text-sm text-gray-400 font-mono w-24 text-right">
            {videoElement ? formatTime(videoElement.duration * 1000) : "--:--"}
          </span>
        </div>

        <!-- Controls Row -->
        <div class="flex items-center gap-4">
          <button
            onclick={() =>
              videoElement && (isPlaying ? videoElement.pause() : videoElement.play())}
            class="btn-primary py-2 px-6"
          >
            {#if isPlaying}
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/>
              </svg>
            {:else}
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z"/>
              </svg>
            {/if}
          </button>

          <div class="flex-1"></div>

          <!-- Offset Adjustment -->
          <div class="flex items-center gap-3 bg-white/5 rounded-xl px-4 py-2">
            <span class="text-sm text-gray-400">Offset:</span>
            <button
              onclick={() => (offsetAdjustment -= 100)}
              class="w-8 h-8 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg text-sm transition-colors"
            >-</button>
            <span class="text-lg font-mono w-24 text-center font-medium {offsetAdjustment > 0 ? 'text-green-400' : offsetAdjustment < 0 ? 'text-red-400' : 'text-white'}">
              {formatOffset(offsetAdjustment)}
            </span>
            <button
              onclick={() => (offsetAdjustment += 100)}
              class="w-8 h-8 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg text-sm transition-colors"
            >+</button>
          </div>

          <button
            onclick={confirmAtCurrentTime}
            disabled={activeSubtitleId === null}
            class="btn-success py-2 px-4 flex items-center gap-2 disabled:opacity-50"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            Conferma Ancora
          </button>
        </div>

        <!-- Shortcuts Help -->
        <div class="flex flex-wrap gap-3 text-xs text-gray-500">
          <div class="flex items-center gap-1">
            <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400">Spazio</kbd>
            <span>Play/Pausa</span>
          </div>
          <div class="flex items-center gap-1">
            <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400">←/→</kbd>
            <span>Seek ±0.1s</span>
          </div>
          <div class="flex items-center gap-1">
            <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400">↑/↓</kbd>
            <span>Offset ±100ms</span>
          </div>
          <div class="flex items-center gap-1">
            <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400">Enter</kbd>
            <span>Conferma</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Right: Subtitle List + Status -->
    <div class="w-1/3 flex flex-col gap-4">
      <!-- Status Card -->
      {#if status?.is_loaded}
        <div class="glass-card p-4 space-y-3 flex-shrink-0">
          <div class="grid grid-cols-2 gap-3">
            <div class="bg-white/5 rounded-xl p-3 text-center">
              <p class="text-2xl font-bold text-white">{status.total_subtitles}</p>
              <p class="text-xs text-gray-500">Sottotitoli</p>
            </div>
            <div class="bg-white/5 rounded-xl p-3 text-center">
              <p class="text-2xl font-bold text-green-400">{status.anchor_count}</p>
              <p class="text-xs text-gray-500">Ancore</p>
            </div>
          </div>

          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span class="text-gray-400">Offset medio:</span>
              <span class="{status.average_offset_ms > 0 ? 'text-green-400' : status.average_offset_ms < 0 ? 'text-red-400' : 'text-white'}">
                {formatOffset(status.average_offset_ms)}
              </span>
            </div>

            <!-- Progress -->
            <div class="progress-modern h-2">
              <div
                class="progress-modern-bar"
                style="width: {status.completion_percentage}%"
              ></div>
            </div>
            <p class="text-xs text-gray-500 text-center">
              {status.completion_percentage.toFixed(1)}% completato
            </p>
          </div>

          <!-- Suggested Next -->
          {#if status.suggested_next_id}
            <button
              onclick={goToSuggested}
              class="w-full py-2 bg-gradient-to-r from-amber-500 to-orange-500 hover:from-amber-600 hover:to-orange-600 rounded-xl text-sm font-medium flex items-center justify-center gap-2 transition-all shadow-lg shadow-amber-500/30"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              Vai al suggerito: #{status.suggested_next_id}
            </button>
          {/if}

          <button
            onclick={resetSync}
            class="w-full py-2 bg-red-500/10 hover:bg-red-500/20 text-red-400 rounded-xl text-sm transition-colors"
          >
            Reset Sincronizzazione
          </button>
        </div>
      {/if}

      <!-- Anchors List -->
      {#if anchors.length > 0}
        <div class="glass-card p-4 flex-shrink-0">
          <h4 class="text-sm font-semibold text-indigo-400 mb-3 flex items-center gap-2">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
            </svg>
            Ancore ({anchors.length})
          </h4>
          <div class="space-y-2 max-h-32 overflow-y-auto">
            {#each anchors as anchor}
              <div class="flex items-center justify-between text-sm bg-white/5 rounded-lg px-3 py-2">
                <span class="text-gray-400">#{anchor.subtitle_id}</span>
                <span class="{anchor.offset_ms >= 0 ? 'text-green-400' : 'text-red-400'}">
                  {formatOffset(anchor.offset_ms)}
                </span>
                <button
                  onclick={() => removeAnchor(anchor.subtitle_id)}
                  class="text-red-400 hover:text-red-300 p-1 hover:bg-red-500/20 rounded transition-colors"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Subtitle List -->
      <div class="glass-card flex-1 overflow-hidden flex flex-col">
        <div class="p-4 border-b border-white/10 flex-shrink-0">
          <h4 class="text-sm font-semibold text-purple-400 flex items-center gap-2">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" />
            </svg>
            Sottotitoli
          </h4>
        </div>
        
        <div class="flex-1 overflow-y-auto">
          {#each subtitles as sub}
            <button
              onclick={() => goToSubtitle(sub)}
              class="w-full text-left p-3 border-b border-white/5 hover:bg-white/5 transition-colors
                {activeSubtitleId === sub.id ? 'bg-indigo-500/20 border-l-4 border-l-indigo-500' : ''}
                {sub.is_anchor ? 'bg-green-500/5' : ''}"
            >
              <div class="flex items-start gap-2">
                <span class="text-xs text-gray-500 w-8 flex-shrink-0">#{sub.id}</span>
                <div class="flex-1 min-w-0">
                  <p class="text-sm truncate text-gray-200">{sub.text}</p>
                  <div class="flex gap-2 text-xs text-gray-500 mt-1">
                    <span class="font-mono">{formatTime(sub.synced_start_ms)}</span>
                    <span class="text-gray-700">→</span>
                    <span class="font-mono">{formatTime(sub.synced_end_ms)}</span>
                    {#if sub.offset_ms !== 0}
                      <span class="{sub.offset_ms > 0 ? 'text-green-400' : 'text-red-400'}">
                        {formatOffset(sub.offset_ms)}
                      </span>
                    {/if}
                  </div>
                </div>
                {#if sub.is_anchor}
                  <span class="text-green-400 flex-shrink-0">
                    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
                    </svg>
                  </span>
                {/if}
              </div>
            </button>
          {/each}

          {#if subtitles.length === 0 && !status?.is_loaded}
            <div class="text-center text-gray-500 py-12">
              <svg class="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p>Carica un file SRT per iniziare</p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>

  <!-- Error Toast -->
  {#if error}
    <div class="fixed bottom-4 right-4 glass-card bg-red-500/20 border border-red-500/30 text-white px-6 py-4 rounded-xl shadow-xl flex items-center gap-3 animate-fade-in">
      <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <span class="text-red-200">{error}</span>
      <button onclick={() => (error = null)} class="text-red-400 hover:text-red-300 ml-2">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/if}
</div>
