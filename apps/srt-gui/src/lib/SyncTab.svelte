<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { locale } from "./i18n";

  interface Props {
    active?: boolean;
  }

  let { active = true }: Props = $props();

  let t = $derived($locale);

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

  let audioElement = $state<HTMLMediaElement | null>(null);
  let status = $state<SyncStatus | null>(null);
  let subtitles = $state<SubtitleInfo[]>([]);
  let anchors = $state<AnchorInfo[]>([]);
  let currentVideoTime = $state(0);
  let isPlaying = $state(false);
  let error = $state<string | null>(null);
  let audioSrc = $state<string | null>(null);
  let audioError = $state<string | null>(null);

  let showResetModal = $state(false);

  let snackbarMessage = $state<string | null>(null);
  let snackbarTimeout = $state<ReturnType<typeof setTimeout> | null>(null);

  let wizardSubtitle = $state<SubtitleInfo | null>(null);
  let offsetAdjustment = $state(0);
  let targetOffset = 0;
  let offsetUpdateFrame = 0;

  function updateOffset(delta: number) {
    targetOffset += delta;
    if (offsetUpdateFrame) cancelAnimationFrame(offsetUpdateFrame);
    offsetUpdateFrame = requestAnimationFrame(() => {
      offsetAdjustment = targetOffset;
    });
  }

  function resetOffset() {
    targetOffset = 0;
    offsetAdjustment = 0;
  }

  let wizardHistory = $state<number[]>([]);
  let showSaveSuggestion = $state(false);
  let manualGoToId = $state("");

  const PAGE_SIZE = 30;
  let loadedRangeStart = $state(1);
  let loadedRangeEnd = $state(0);
  let isLoadingMore = $state(false);
  let subtitleListElement = $state<HTMLDivElement | null>(null);

  let isDraggingOver = $state(false);
  let audioDuration = $state(0);
  let hasAudio = $derived(!!audioSrc && !audioError);

  let subtitleContextMenu = $state<{
    x: number;
    y: number;
    sub: SubtitleInfo;
  } | null>(null);

  let isNavigating = $state(false);
  let isStartingPlayback = $state(false);
  let isConfirmingCheckpoint = $state(false);

  let helpSection = $state<string | null>(null);
  let expandedPathField = $state<"srt" | "media" | null>(null);

  type SyncPanelId = "toolbar" | "wizard" | "status" | "subtitleList";

  function syncDebug(message: string, payload?: Record<string, unknown>) {
    if (payload) {
      console.info(`[SyncTab] ${message}`, payload);
      return;
    }
    console.info(`[SyncTab] ${message}`);
  }

  function getFileName(path: string): string {
    return path.split("/").pop() || path;
  }

  async function tryAutoSelectMediaForSrt(srtPath: string) {
    try {
      const suggestedPath = await invoke<string | null>("sync_suggest_media_for_srt", {
        srtPath,
      });
      if (!suggestedPath) return;

      audioError = null;
      cleanupAudioSrc();
      audioSrc = await loadMediaFile(suggestedPath);
      status = await invoke<SyncStatus>("sync_set_video", { path: suggestedPath });
      showSnackbar(`Auto-selected media: ${getFileName(suggestedPath)}`);
    } catch (e) {
      syncDebug("auto-media-suggestion-failed", { error: String(e) });
    }
  }

  async function safePlayAudio(source: string): Promise<boolean> {
    if (!audioElement || !hasAudio) {
      syncDebug(`${source}: play skipped`, {
        hasAudio,
        hasElement: !!audioElement,
        audioError,
      });
      return false;
    }
    if (isStartingPlayback) {
      syncDebug(`${source}: play ignored (already starting)`);
      return false;
    }

    isStartingPlayback = true;
    try {
      syncDebug(`${source}: play requested`, {
        currentTime: audioElement.currentTime,
        paused: audioElement.paused,
        readyState: audioElement.readyState,
      });
      await audioElement.play();
      syncDebug(`${source}: play started`, {
        currentTime: audioElement.currentTime,
        paused: audioElement.paused,
      });
      return true;
    } catch (e) {
      const err = e instanceof Error ? `${e.name}: ${e.message}` : String(e);
      const isBenignAbort =
        err.includes("AbortError") ||
        err.toLowerCase().includes("operation was aborted");
      syncDebug(`${source}: play failed`, { error: err });
      if (!isBenignAbort) {
        showSnackbar(`Play error: ${err}`);
      }
      return false;
    } finally {
      isStartingPlayback = false;
    }
  }

  function openSubtitleContextMenu(e: MouseEvent, sub: SubtitleInfo) {
    e.preventDefault();
    subtitleContextMenu = { x: e.clientX, y: e.clientY, sub };
  }

  function closeSubtitleContextMenu() {
    subtitleContextMenu = null;
  }

  async function playSubtitleFromList(sub: SubtitleInfo) {
    if (isNavigating) return;
    isNavigating = true;
    try {
      syncDebug("playSubtitleFromList", { subtitleId: sub.id });
      wizardSubtitle = sub;
      resetOffset();
      showSaveSuggestion = false;
      if (!wizardHistory.includes(sub.id)) {
        wizardHistory = [...wizardHistory, sub.id];
      }
      seekToSubtitleStart(sub);
      scrollToSubtitle(sub.id);
      await safePlayAudio("playSubtitleFromList");
    } finally {
      isNavigating = false;
    }
  }

  let mediaServerPort: number | null = null;

  async function getMediaPort(): Promise<number> {
    if (mediaServerPort) return mediaServerPort;
    mediaServerPort = await invoke<number>("get_media_server_port");
    return mediaServerPort;
  }

  async function loadMediaFile(filePath: string): Promise<string> {
    const port = await getMediaPort();
    return `http://127.0.0.1:${port}/media?path=${encodeURIComponent(filePath)}`;
  }

  function cleanupAudioSrc() {
    audioSrc = null;
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

  function showSnackbar(message: string) {
    if (snackbarTimeout) clearTimeout(snackbarTimeout);
    snackbarMessage = message;
    snackbarTimeout = setTimeout(() => {
      snackbarMessage = null;
    }, 3500);
  }

  const OFFSET_TOLERANCE_MS = 200;

  function computeNextCheckpoint(): number | null {
    if (!status || status.total_subtitles === 0) return null;
    const total = status.total_subtitles;

    const initialCheckpoints = [
      1,
      Math.max(2, Math.round(total * 0.33)),
      Math.max(3, Math.round(total * 0.66)),
    ];

    for (const cp of initialCheckpoints) {
      if (!wizardHistory.includes(cp)) return cp;
    }

    if (areOffsetsConsistent()) return null;

    const sortedAnchors = [...anchors].sort(
      (a, b) => a.subtitle_id - b.subtitle_id,
    );
    if (sortedAnchors.length < 2) {
      const mid = Math.round(total / 2);
      return wizardHistory.includes(mid) ? null : mid;
    }

    let maxDiff = 0;
    let bestMid = -1;
    for (let i = 0; i < sortedAnchors.length - 1; i++) {
      const a = sortedAnchors[i];
      const b = sortedAnchors[i + 1];
      const diff = Math.abs(a.offset_ms - b.offset_ms);
      const gap = b.subtitle_id - a.subtitle_id;
      if (diff > OFFSET_TOLERANCE_MS && gap > 1) {
        const mid = Math.round((a.subtitle_id + b.subtitle_id) / 2);
        if (!wizardHistory.includes(mid) && diff > maxDiff) {
          maxDiff = diff;
          bestMid = mid;
        }
      }
    }

    if (bestMid > 0) return bestMid;

    const first = sortedAnchors[0];
    const last = sortedAnchors[sortedAnchors.length - 1];
    if (first.subtitle_id > 2) {
      const mid = Math.round(first.subtitle_id / 2);
      if (!wizardHistory.includes(mid)) return mid;
    }
    if (last.subtitle_id < total - 1) {
      const mid = Math.round((last.subtitle_id + total) / 2);
      if (!wizardHistory.includes(mid)) return mid;
    }

    return null;
  }

  function areOffsetsConsistent(): boolean {
    if (anchors.length < 2) return false;
    const offsets = anchors.map((a) => a.offset_ms);
    const minOff = Math.min(...offsets);
    const maxOff = Math.max(...offsets);
    return maxOff - minOff <= OFFSET_TOLERANCE_MS;
  }

  async function advanceWizard() {
    console.debug("[SyncTab] advanceWizard called");
    const nextId = computeNextCheckpoint();
    if (nextId === null) {
      showSaveSuggestion = true;
      wizardSubtitle = null;
      return;
    }
    showSaveSuggestion = false;
    await goToCheckpoint(nextId);
  }

  async function goToCheckpoint(id: number) {
    console.debug(`[SyncTab] goToCheckpoint(${id}) called, isNavigating=${isNavigating}`);
    if (isNavigating) return;
    isNavigating = true;
    try {
      const sub = await invoke<SubtitleInfo>("sync_get_subtitle", { id });
      wizardSubtitle = sub;
      resetOffset();
      if (!wizardHistory.includes(id)) {
        wizardHistory = [...wizardHistory, id];
      }
      seekToSubtitleStart(sub);
      await loadSubtitlesAround(id);
      setTimeout(() => scrollToSubtitle(id), 50);
    } catch (e) {
      error = `Error loading subtitle: ${e}`;
    } finally {
      isNavigating = false;
    }
  }

  function seekToSubtitleStart(sub: SubtitleInfo) {
    if (!audioElement) return;
    const startSec = (sub.synced_start_ms + offsetAdjustment) / 1000;
    audioElement.currentTime = Math.max(0, startSec);
  }

  function replayCurrentSubtitle() {
    if (!wizardSubtitle || !audioElement) return;
    seekToSubtitleStart(wizardSubtitle);
    void safePlayAudio("replayCurrentSubtitle");
  }

  async function selectSrtFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "SRT Files", extensions: ["srt"] }],
      });
      if (selected) {
        const selectedPath = selected as string;
        status = await invoke<SyncStatus>("sync_load_srt", {
          path: selectedPath,
        });
        await loadSubtitles();
        await loadAnchors();
        wizardHistory = [];
        showSaveSuggestion = false;
        await advanceWizard();
        await tryAutoSelectMediaForSrt(selectedPath);
      }
    } catch (e) {
      error = `${t("sync.errorLoadingSrt")} ${e}`;
    }
  }

  async function selectAudioFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Audio/Video Files",
            extensions: [
              "mp3",
              "wav",
              "ogg",
              "flac",
              "m4a",
              "aac",
              "wma",
              "opus",
              "m4b",
              "mp4",
              "mkv",
              "avi",
              "webm",
              "mov",
              "m4v",
              "m2ts",
              "mpeg",
              "mpg",
            ],
          },
        ],
      });
      if (selected) {
        const path = selected as string;
        audioError = null;
        cleanupAudioSrc();
        audioSrc = await loadMediaFile(path);
        status = await invoke<SyncStatus>("sync_set_video", { path });
      }
    } catch (e) {
      error = `${t("sync.errorLoadingAudio")} ${e}`;
    }
  }

  async function loadSubtitles() {
    console.debug("[SyncTab] loadSubtitles");
    try {
      subtitles = await invoke<SubtitleInfo[]>("sync_get_subtitles_range", {
        startId: 1,
        count: PAGE_SIZE,
      });
      if (subtitles.length > 0) {
        loadedRangeStart = subtitles[0].id;
        loadedRangeEnd = subtitles[subtitles.length - 1].id;
      } else {
        loadedRangeStart = 1;
        loadedRangeEnd = 0;
      }
    } catch (e) {
      error = `${t("sync.errorLoadingSrt")} ${e}`;
    }
  }

  async function loadSubtitlesAround(targetId: number) {
    console.debug(`[SyncTab] loadSubtitlesAround(${targetId})`);
    try {
      isLoadingMore = true;
      const halfPage = Math.floor(PAGE_SIZE / 2);
      const startId = Math.max(1, targetId - halfPage);
      subtitles = await invoke<SubtitleInfo[]>("sync_get_subtitles_range", {
        startId,
        count: PAGE_SIZE,
      });
      if (subtitles.length > 0) {
        loadedRangeStart = subtitles[0].id;
        loadedRangeEnd = subtitles[subtitles.length - 1].id;
      }
    } catch (e) {
      error = `${t("sync.errorLoadingSrt")} ${e}`;
    } finally {
      isLoadingMore = false;
    }
  }

  async function loadMoreSubtitlesAfter() {
    if (isLoadingMore || !status || loadedRangeEnd >= status.total_subtitles)
      return;
    try {
      isLoadingMore = true;
      const newSubs = await invoke<SubtitleInfo[]>("sync_get_subtitles_range", {
        startId: loadedRangeEnd + 1,
        count: PAGE_SIZE,
      });
      if (newSubs.length > 0) {
        const maxItems = PAGE_SIZE * 3;
        let combined = [...subtitles, ...newSubs];
        if (combined.length > maxItems) {
          const toRemove = combined.length - maxItems;
          combined = combined.slice(toRemove);
          loadedRangeStart = combined[0].id;
        }
        subtitles = combined;
        loadedRangeEnd = combined[combined.length - 1].id;
      }
    } catch (e) {
      error = `${t("sync.errorLoadingSrt")} ${e}`;
    } finally {
      isLoadingMore = false;
    }
  }

  async function loadMoreSubtitlesBefore() {
    if (isLoadingMore || loadedRangeStart <= 1) return;
    try {
      isLoadingMore = true;
      const startId = Math.max(1, loadedRangeStart - PAGE_SIZE);
      const count = loadedRangeStart - startId;
      if (count <= 0) return;
      const newSubs = await invoke<SubtitleInfo[]>("sync_get_subtitles_range", {
        startId,
        count,
      });
      if (newSubs.length > 0) {
        const maxItems = PAGE_SIZE * 3;
        let combined = [...newSubs, ...subtitles];
        if (combined.length > maxItems) combined = combined.slice(0, maxItems);
        subtitles = combined;
        loadedRangeStart = combined[0].id;
        loadedRangeEnd = combined[combined.length - 1].id;
      }
    } catch (e) {
      error = `${t("sync.errorLoadingSrt")} ${e}`;
    } finally {
      isLoadingMore = false;
    }
  }

  function handleSubtitleListScroll(e: Event) {
    const target = e.target as HTMLDivElement;
    if (!target || !status) return;
    if (target.scrollHeight - target.scrollTop - target.clientHeight < 100)
      loadMoreSubtitlesAfter();
    if (target.scrollTop < 100) loadMoreSubtitlesBefore();
  }

  function scrollToSubtitle(subtitleId: number) {
    if (!subtitleListElement) return;
    const element = subtitleListElement.querySelector(
      `[data-subtitle-id="${subtitleId}"]`,
    );
    if (element)
      element.scrollIntoView({ behavior: "smooth", block: "center" });
  }

  async function loadAnchors() {
    console.debug("[SyncTab] loadAnchors");
    try {
      anchors = await invoke<AnchorInfo[]>("sync_get_anchors");
    } catch (e) {
      error = `${t("sync.errorAddingAnchor")} ${e}`;
    }
  }

  async function refreshCurrentSubtitles() {
    console.debug(`[SyncTab] refreshCurrentSubtitles (range ${loadedRangeStart}-${loadedRangeEnd})`);
    if (loadedRangeStart > 0 && loadedRangeEnd > 0) {
      try {
        const count = loadedRangeEnd - loadedRangeStart + 1;
        subtitles = await invoke<SubtitleInfo[]>("sync_get_subtitles_range", {
          startId: loadedRangeStart,
          count: Math.max(count, PAGE_SIZE),
        });
        if (subtitles.length > 0) {
          loadedRangeStart = subtitles[0].id;
          loadedRangeEnd = subtitles[subtitles.length - 1].id;
        }
      } catch (e) {
        error = `${t("sync.errorLoadingSrt")} ${e}`;
      }
    } else {
      await loadSubtitles();
    }
  }

  async function confirmCurrentCheckpoint() {
    console.debug("[SyncTab] confirmCurrentCheckpoint called");
    if (isConfirmingCheckpoint) {
      syncDebug("confirmCurrentCheckpoint ignored (already running)");
      return;
    }
    if (!wizardSubtitle) return;
    if (!audioSrc || audioError) {
      showSnackbar(t("sync.needAudioForAnchor"));
      return;
    }

    const correctedTime = wizardSubtitle.start_ms + offsetAdjustment;
    isConfirmingCheckpoint = true;

    try {
      syncDebug("confirmCurrentCheckpoint start", {
        subtitleId: wizardSubtitle.id,
        correctedTime: Math.round(correctedTime),
        offsetAdjustment,
      });
      status = await invoke<SyncStatus>("sync_add_anchor", {
        subtitleId: wizardSubtitle.id,
        correctedTimeMs: Math.round(correctedTime),
      });
      await refreshCurrentSubtitles();
      await loadAnchors();

      const updated = await invoke<SubtitleInfo>("sync_get_subtitle", {
        id: wizardSubtitle.id,
      });
      wizardSubtitle = updated;

      resetOffset();
      await advanceWizard();
      syncDebug("confirmCurrentCheckpoint completed", {
        subtitleId: updated.id,
      });
    } catch (e) {
      error = `${t("sync.errorAddingAnchor")} ${e}`;
      syncDebug("confirmCurrentCheckpoint failed", { error: String(e) });
    } finally {
      isConfirmingCheckpoint = false;
    }
  }

  async function removeAnchor(subtitleId: number) {
    console.debug(`[SyncTab] removeAnchor(${subtitleId})`);
    try {
      status = await invoke<SyncStatus>("sync_remove_anchor", { subtitleId });
      await refreshCurrentSubtitles();
      await loadAnchors();
    } catch (e) {
      error = `${t("sync.errorRemovingAnchor")} ${e}`;
    }
  }

  async function goToSubtitleManual(sub: SubtitleInfo) {
    if (isNavigating) return;
    isNavigating = true;
    try {
      wizardSubtitle = sub;
      resetOffset();
      showSaveSuggestion = false;
      if (!wizardHistory.includes(sub.id)) {
        wizardHistory = [...wizardHistory, sub.id];
      }
      seekToSubtitleStart(sub);
      scrollToSubtitle(sub.id);
    } finally {
      isNavigating = false;
    }
  }

  async function goToLineById() {
    const id = parseInt(manualGoToId);
    if (isNaN(id) || id < 1) return;
    try {
      const sub = await invoke<SubtitleInfo>("sync_get_subtitle", { id });
      await goToSubtitleManual(sub);
      await loadSubtitlesAround(id);
      setTimeout(() => scrollToSubtitle(id), 50);
      manualGoToId = "";
    } catch (e) {
      showSnackbar(`Subtitle #${id} not found`);
    }
  }

  async function saveFile() {
    console.debug("[SyncTab] saveFile");
    try {
      const selected = await save({
        filters: [{ name: "SRT Files", extensions: ["srt"] }],
        defaultPath: status?.srt_path?.replace(".srt", ".synced.srt"),
      });
      if (selected) {
        await invoke<string>("sync_save_file", { outputPath: selected });
        showSnackbar(`${t("sync.fileSaved")} ${selected}`);
      }
    } catch (e) {
      error = `${t("sync.errorSaving")} ${e}`;
    }
  }

  async function saveSession() {
    console.debug("[SyncTab] saveSession");
    try {
      const selected = await save({
        filters: [{ name: "Session Files", extensions: ["json"] }],
      });
      if (selected) {
        await invoke<string>("sync_save_session", { sessionPath: selected });
        showSnackbar(`${t("sync.sessionSaved")} ${selected}`);
      }
    } catch (e) {
      error = `${t("sync.errorSaving")} ${e}`;
    }
  }

  async function loadSession() {
    console.debug("[SyncTab] loadSession");
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
        wizardHistory = anchors.map((a) => a.subtitle_id);
        await advanceWizard();
      }
    } catch (e) {
      error = `${t("sync.errorLoadingSrt")} ${e}`;
    }
  }

  async function confirmReset() {
    showResetModal = false;
    console.debug("[SyncTab] confirmReset");
    try {
      status = await invoke<SyncStatus>("sync_reset");
      cleanupAudioSrc();
      audioError = null;
      subtitles = [];
      anchors = [];
      wizardSubtitle = null;
      wizardHistory = [];
      showSaveSuggestion = false;
      currentVideoTime = 0;
      isPlaying = false;
      resetOffset();
      loadedRangeStart = 1;
      loadedRangeEnd = 0;
      manualGoToId = "";
      if (audioElement) {
        audioElement.pause();
        audioElement.src = "";
      }
    } catch (e) {
      error = `${t("sync.errorSaving")} ${e}`;
    }
  }

  function isSrtFile(name: string): boolean {
    return name.toLowerCase().endsWith(".srt");
  }
  function isMediaFile(name: string): boolean {
    const ext = name.toLowerCase().split(".").pop() || "";
    return [
      "mp4",
      "mkv",
      "avi",
      "webm",
      "mov",
      "m4v",
      "m2ts",
      "mpeg",
      "mpg",
      "mp3",
      "wav",
      "ogg",
      "flac",
      "m4a",
      "aac",
      "wma",
      "opus",
      "m4b",
    ].includes(ext);
  }

  async function handleDroppedPaths(paths: string[]) {
    if (paths.length === 0) return;

    // Sort paths so SRT files are processed first
    const sortedPaths = [...paths].sort((a, b) => {
      const isASrt = isSrtFile(a);
      const isBSrt = isSrtFile(b);
      if (isASrt && !isBSrt) return -1;
      if (!isASrt && isBSrt) return 1;
      return 0;
    });

    for (const filePath of sortedPaths) {
      const fileName = getFileName(filePath);
      if (isSrtFile(fileName)) {
        try {
          status = await invoke<SyncStatus>("sync_load_srt", {
            path: filePath,
          });
          await loadSubtitles();
          await loadAnchors();
          wizardHistory = [];
          showSaveSuggestion = false;
          await advanceWizard();
          await tryAutoSelectMediaForSrt(filePath);
        } catch (e: any) {
          error = `${t("sync.errorLoadingSrt")} ${e}`;
        }
      } else if (isMediaFile(fileName)) {
        if (!status?.is_loaded) {
          showSnackbar(t("sync.dropSrtFirst"));
          return;
        }
        try {
          audioError = null;
          cleanupAudioSrc();
          audioSrc = await loadMediaFile(filePath);
          status = await invoke<SyncStatus>("sync_set_video", {
            path: filePath,
          });
        } catch (e: any) {
          error = `${t("sync.errorLoadingAudio")} ${e}`;
        }
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (
      document.activeElement?.tagName === "INPUT" ||
      document.activeElement?.tagName === "TEXTAREA"
    )
      return;

    switch (e.key) {
      case " ":
        e.preventDefault();
        if (hasAudio && audioElement) {
          if (isPlaying) {
            audioElement.pause();
          } else {
            void safePlayAudio("keyboard-space");
          }
        }
        break;
      case "ArrowLeft":
        e.preventDefault();
        if (hasAudio && audioElement)
          audioElement.currentTime -= e.shiftKey ? 1 : 0.1;
        break;
      case "ArrowRight":
        e.preventDefault();
        if (hasAudio && audioElement)
          audioElement.currentTime += e.shiftKey ? 1 : 0.1;
        break;
      case "ArrowUp":
        if (wizardSubtitle) {
          e.preventDefault();
          updateOffset(e.altKey ? 5000 : e.shiftKey ? 500 : 100);
          seekToSubtitleStart(wizardSubtitle);
        }
        break;
      case "ArrowDown":
        if (wizardSubtitle) {
          e.preventDefault();
          updateOffset(-(e.altKey ? 5000 : e.shiftKey ? 500 : 100));
          seekToSubtitleStart(wizardSubtitle);
        }
        break;
      case "Enter":
        e.preventDefault();
        confirmCurrentCheckpoint();
        break;
      case "r":
      case "R":
        e.preventDefault();
        replayCurrentSubtitle();
        break;
    }
  }

  let singleClickTimer: ReturnType<typeof setTimeout> | null = null;

  function handleSubtitleClick(sub: SubtitleInfo) {
    if (singleClickTimer) {
      clearTimeout(singleClickTimer);
      singleClickTimer = null;
    }
    singleClickTimer = setTimeout(() => {
      singleClickTimer = null;
      goToSubtitleManual(sub);
    }, 200);
  }

  function handleSubtitleDblClick(sub: SubtitleInfo) {
    if (singleClickTimer) {
      clearTimeout(singleClickTimer);
      singleClickTimer = null;
    }
    playSubtitleFromList(sub);
  }

  onMount(() => {
    syncDebug("mount", { active });
    window.addEventListener("keydown", handleKeydown);
    let unlistenDragDrop: (() => void) | null = null;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (!active) return;
        if (event.payload.type === "over") {
          isDraggingOver = true;
        }
        else if (event.payload.type === "drop") {
          isDraggingOver = false;
          syncDebug("file-drop", { count: event.payload.paths.length });
          handleDroppedPaths(event.payload.paths);
        } else if (event.payload.type === "leave") isDraggingOver = false;
      })
      .then((fn) => {
        unlistenDragDrop = fn;
      });
    return () => {
      syncDebug("unmount");
      window.removeEventListener("keydown", handleKeydown);
      if (unlistenDragDrop) unlistenDragDrop();
      if (singleClickTimer) clearTimeout(singleClickTimer);
    };
  });
</script>

<div
  class="h-full flex flex-col p-6 overflow-hidden bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950 relative"
  role="application"
  ondrop={(e) => {
    e.preventDefault();
    if (e.dataTransfer?.types.includes("Files")) {
      isDraggingOver = false;
    }
  }}
  ondragover={(e) => {
    e.preventDefault();
    if (e.dataTransfer?.types.includes("Files")) {
      isDraggingOver = true;
    }
  }}
  ondragleave={(e) => {
    const rt = e.relatedTarget as HTMLElement | null;
    const ct = e.currentTarget as HTMLElement;
    if (rt && ct.contains(rt)) return;
    if (e.dataTransfer?.types.includes("Files")) {
      isDraggingOver = false;
    }
  }}
>
  {#if isDraggingOver}
    <div
      class="absolute inset-0 z-50 bg-indigo-500/10 border-2 border-dashed border-indigo-400 rounded-2xl flex items-center justify-center pointer-events-none"
    >
      <div class="text-center">
        <svg
          class="w-16 h-16 mx-auto mb-3 text-indigo-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          ><path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
          /></svg
        >
        <p class="text-lg font-medium text-indigo-300">
          {t("sync.dropFileHere")}
        </p>
        <p class="text-sm text-gray-400 mt-1">{t("sync.dropFileHint")}</p>
      </div>
    </div>
  {/if}

  <!-- svelte-ignore a11y_media_has_caption -->
  <audio
    bind:this={audioElement}
    src={audioSrc || undefined}
    class="hidden"
    preload="metadata"
    ontimeupdate={() => {
      if (audioElement) {
        currentVideoTime = audioElement.currentTime;
        if (wizardSubtitle && !audioElement.paused) {
          const endSec =
            (wizardSubtitle.synced_end_ms + offsetAdjustment) / 1000;
          if (audioElement.currentTime >= endSec + 0.05) {
            audioElement.pause();
            seekToSubtitleStart(wizardSubtitle);
          }
        }
      }
    }}
    onplay={() => {
      isPlaying = true;
      syncDebug("audio:onplay", {
        currentTime: audioElement?.currentTime,
        duration: audioElement?.duration,
      });
    }}
    onpause={() => {
      isPlaying = false;
      syncDebug("audio:onpause", {
        currentTime: audioElement?.currentTime,
      });
    }}
    onloadedmetadata={() => {
      if (audioElement) audioDuration = audioElement.duration;
      audioError = null;
      syncDebug("audio:onloadedmetadata", {
        duration: audioElement?.duration,
        src: audioSrc,
      });
    }}
    onerror={(e) => {
      const el = e.currentTarget as HTMLMediaElement;
      const mediaErr = el?.error;
      const codeMap: Record<number, string> = {
        1: "MEDIA_ERR_ABORTED",
        2: "MEDIA_ERR_NETWORK",
        3: "MEDIA_ERR_DECODE",
        4: "MEDIA_ERR_SRC_NOT_SUPPORTED",
      };
      const code = mediaErr?.code || 0;
      const codeStr = codeMap[code] || `Unknown error: ${code}`;
      const msg = mediaErr?.message || "";
      const gstMissingSink = /autoaudiosink|audiosink/i.test(msg);
      if (gstMissingSink) {
        audioError = `${codeStr}. Audio backend non disponibile su Linux. Installa almeno gstreamer1.0-plugins-good e, se necessario, gstreamer1.0-pulseaudio o pipewire-audio, poi riavvia l'app. ${msg}`;
      } else if (code === 3 || code === 4) {
        audioError = `${codeStr}. Su Linux potrebbe servire: gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav. ${msg}`;
      } else {
        audioError = `${codeStr}. ${msg}`;
      }
      syncDebug("audio:onerror", {
        code,
        codeStr,
        message: msg,
      });
    }}
    oncanplay={() => {
      audioError = null;
      syncDebug("audio:oncanplay", {
        currentTime: audioElement?.currentTime,
        readyState: audioElement?.readyState,
      });
    }}
    onseeking={() => {
      syncDebug("audio:onseeking", { currentTime: audioElement?.currentTime });
    }}
    onseeked={() => {
      syncDebug("audio:onseeked", { currentTime: audioElement?.currentTime });
    }}
  ></audio>

  {#snippet panelContent(panelId: SyncPanelId)}
    {#if panelId === "toolbar"}
      <div class="glass-card flex items-center gap-4 p-4 flex-shrink-0">
        <div class="flex items-center gap-2 flex-1 max-w-2xl">
          <div class="flex-1 min-w-[180px]">
            <div class="flex gap-2">
              <button
                type="button"
                onclick={() => {
                  if (status?.srt_path) expandedPathField = "srt";
                }}
                class="input-modern flex-1 text-sm text-left transition-colors truncate {status?.srt_path
                  ? 'cursor-pointer hover:bg-white/10'
                  : 'cursor-default hover:bg-transparent'}"
                style="direction: rtl; text-align: left;"
                title={status?.srt_path || t("sync.noSrt")}
              >
                <span
                  class={status?.srt_path ? "text-white" : "text-gray-500"}
                  style="unicode-bidi: plaintext;"
                >
                  {status?.srt_path || t("sync.loadSrt")}
                </span>
              </button>
              <button
                onclick={selectSrtFile}
                class="btn-primary py-2 px-3 flex items-center justify-center"
                title={t("sync.tooltip.loadSrt")}
              >
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
              </button>
            </div>
          </div>

          <div class="text-gray-500 {status?.is_loaded ? 'text-indigo-400' : ''}">
            <svg
              class="w-5 h-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 7l5 5m0 0l-5 5m5-5H6"
              /></svg
            >
          </div>

          <div class="flex-1 min-w-[180px]">
            <div class="flex gap-2">
              <button
                type="button"
                onclick={() => {
                  if (status?.video_path) expandedPathField = "media";
                }}
                class="input-modern flex-1 text-sm text-left transition-colors truncate {status?.video_path
                  ? 'cursor-pointer hover:bg-white/10'
                  : 'cursor-default hover:bg-transparent'} {!status?.is_loaded ? 'opacity-60' : ''}"
                style="direction: rtl; text-align: left;"
                title={status?.video_path || t("sync.noVideo")}
              >
                <span
                  class={status?.video_path ? "text-white" : "text-gray-500"}
                  style="unicode-bidi: plaintext;"
                >
                  {status?.video_path || t("sync.loadAudio")}
                </span>
              </button>
              <button
                onclick={selectAudioFile}
                disabled={!status?.is_loaded}
                class="btn-secondary py-2 px-3 disabled:opacity-30 disabled:cursor-not-allowed"
                title={t("sync.tooltip.loadVideo")}
              >
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
              </button>
            </div>
          </div>
        </div>

        <div class="flex-1"></div>
        <div class="relative group">
          <button
            type="button"
            disabled
            class="hidden lg:flex py-1.5 px-4 rounded-lg bg-indigo-500/10 border border-indigo-500/30 text-indigo-300 font-medium items-center gap-2 opacity-60 cursor-not-allowed transition-all"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" /></svg>
            Auto-Sync
          </button>
          <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1.5 bg-gray-800 border border-white/10 text-xs text-indigo-300 rounded-lg shadow-xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50">
            Coming soon — requires Whisper model
          </div>
        </div>
        {#if status?.is_loaded || audioSrc}
          <button
            onclick={() => (showResetModal = true)}
            class="py-1.5 px-4 rounded-lg border border-amber-500/30 bg-amber-500/10 text-amber-300 hover:bg-amber-500/20 transition-colors text-sm font-medium flex items-center gap-2"
            title={t("sync.newSyncDesc")}
          >
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              /></svg
            >
            {t("sync.newSync")}
          </button>
        {/if}

        <button
          onclick={loadSession}
          class="btn-secondary py-2 px-4 flex items-center gap-2"
          title={t("sync.tooltipLoadSession")}
        >
          <svg
            class="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
            /></svg
          >
          {t("sync.loadSession")}
        </button>
        <button
          onclick={saveSession}
          disabled={!status?.is_loaded}
          class="btn-secondary py-2 px-4 flex items-center gap-2 disabled:opacity-50"
          title={t("sync.tooltipSaveSession")}
        >
          <svg
            class="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"
            /></svg
          >
          {t("sync.saveSession")}
        </button>
        <button
          onclick={saveFile}
          disabled={!status?.is_loaded}
          class="btn-success py-2 px-4 flex items-center gap-2 disabled:opacity-50"
          title={t("sync.tooltipSaveFile")}
        >
          <svg
            class="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            /></svg
          >
          {t("sync.saveFile")}
        </button>
      </div>
    {:else if panelId === "wizard"}
      <div class="glass-card relative flex flex-col h-full overflow-hidden">
        <div
          class="p-3 border-b border-white/10 flex items-center gap-2 flex-shrink-0"
        >
          <svg
            class="w-5 h-5 text-indigo-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
            /></svg
          >
          <h3 class="text-sm font-semibold text-white">
            {t("sync.wizard.title")}
          </h3>
          <button
            type="button"
            onclick={() => (helpSection = "wizard")}
            class="ml-auto text-gray-500 hover:text-cyan-300 transition-colors"
            title="Info"
          >
            <svg
              class="w-3.5 h-3.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              /></svg
            >
          </button>
        </div>

        <div
          class="flex-1 flex flex-col items-center justify-center p-6 min-h-0 overflow-y-auto"
        >
          {#if !status?.is_loaded}
            <div class="text-gray-500 text-center">
              <svg
                class="w-20 h-20 mx-auto mb-4 opacity-50"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                ><path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                /></svg
              >
              <p class="text-lg">{t("sync.srtPlaceholder")}</p>
            </div>
          {:else if showSaveSuggestion}
            <div class="text-center max-w-md">
              <div
                class="w-20 h-20 mx-auto mb-6 rounded-full bg-gradient-to-br from-green-500/20 to-emerald-500/20 flex items-center justify-center"
              >
                <svg
                  class="w-10 h-10 text-green-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                  ><path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5 13l4 4L19 7"
                  /></svg
                >
              </div>
              <h3 class="text-2xl font-bold text-white mb-3">
                {t("sync.wizard.allSynced")}
              </h3>
              <p class="text-gray-400 mb-6">{t("sync.wizard.suggestSave")}</p>
              <div class="flex gap-3 justify-center">
                <button
                  onclick={saveFile}
                  class="btn-success py-3 px-8 flex items-center gap-2 text-lg shadow-lg shadow-green-500/30"
                >
                  <svg
                    class="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    ><path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M5 13l4 4L19 7"
                    /></svg
                  >
                  {t("sync.saveFile")}
                </button>
                <button
                  onclick={() => {
                    showSaveSuggestion = false;
                  }}
                  class="btn-secondary py-3 px-6"
                >
                  {t("sync.wizard.continueChecking")}
                </button>
              </div>
            </div>
          {:else if wizardSubtitle}
            <div class="w-full max-w-2xl flex flex-col gap-4">
              <div class="text-center flex-shrink-0">
                <span
                  class="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-indigo-500/20 text-indigo-300 text-sm font-medium"
                >
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    ><path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
                    /></svg
                  >
                  {t("sync.wizard.checkpoint")} — #{wizardSubtitle.id} / {status?.total_subtitles}
                </span>
              </div>

              <div
                class="bg-white/5 rounded-2xl p-6 text-center flex-shrink-0 flex flex-col items-center justify-center"
                style="min-height: 120px;"
              >
                <p class="text-2xl text-white font-medium leading-relaxed">
                  {wizardSubtitle.text}
                </p>
                <p class="text-sm text-gray-500 mt-3 font-mono">
                  {formatTime(wizardSubtitle.start_ms)} → {formatTime(
                    wizardSubtitle.end_ms,
                  )}
                </p>
              </div>

              <div class="flex-shrink-0">
                {#if audioSrc && !audioError}
                  <div class="flex items-center gap-4">
                    <span class="text-sm text-gray-400 font-mono w-24"
                      >{formatTime(currentVideoTime * 1000)}</span
                    >
                    <input
                      type="range"
                      min="0"
                      max={audioDuration || 100}
                      step="0.01"
                      bind:value={currentVideoTime}
                      oninput={() => {
                        if (audioElement)
                          audioElement.currentTime = currentVideoTime;
                      }}
                      class="flex-1"
                    />
                    <span
                      class="text-sm text-gray-400 font-mono w-24 text-right"
                      >{audioDuration
                        ? formatTime(audioDuration * 1000)
                        : "--:--"}</span
                    >
                  </div>
                {:else if !audioSrc}
                  <div class="text-center py-4">
                    <p class="text-gray-500">{t("sync.audioPlaceholder")}</p>
                    <p class="text-xs text-gray-600 mt-1">
                      {t("sync.audioFormats")}
                    </p>
                  </div>
                {:else if audioError}
                  <div class="text-center py-4 max-w-xl mx-auto">
                    <p
                      class="text-red-400 text-sm whitespace-pre-wrap break-words"
                    >
                      {audioError}
                    </p>
                    <button
                      onclick={() => {
                        audioError = null;
                        audioSrc = null;
                        selectAudioFile();
                      }}
                      class="btn-secondary text-sm mt-2"
                      >{t("sync.tryAnotherFile")}</button
                    >
                  </div>
                {/if}
              </div>

              <div
                class="flex items-center justify-center gap-4 flex-wrap flex-shrink-0"
                style="min-height: 60px;"
              >
                <button
                  onclick={() =>
                    hasAudio &&
                    audioElement &&
                    (isPlaying
                      ? audioElement.pause()
                      : void safePlayAudio("wizard-play-button"))}
                  disabled={!hasAudio}
                  class="w-14 h-14 flex items-center justify-center rounded-full bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700 shadow-lg shadow-indigo-500/30 transition-all disabled:opacity-40 disabled:cursor-not-allowed"
                  title={isPlaying ? "Pause" : "Play"}
                >
                  {#if isPlaying}
                    <svg
                      class="w-7 h-7 text-white"
                      fill="currentColor"
                      viewBox="0 0 24 24"
                      ><path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" /></svg
                    >
                  {:else}
                    <svg
                      class="w-7 h-7 text-white ml-1"
                      fill="currentColor"
                      viewBox="0 0 24 24"><path d="M8 5v14l11-7z" /></svg
                    >
                  {/if}
                </button>

                <button
                  onclick={replayCurrentSubtitle}
                  disabled={!hasAudio}
                  class="w-12 h-12 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                  title={t("sync.wizard.replay") + " (R)"}
                >
                  <svg
                    class="w-6 h-6 text-cyan-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    ><path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                    /></svg
                  >
                </button>

                <div
                  class="flex items-center gap-2 bg-white/5 rounded-xl px-4 py-2 {!hasAudio
                    ? 'opacity-40 pointer-events-none'
                    : ''}"
                >
                  <button
                    onclick={() => {
                      updateOffset(-5000);
                      replayCurrentSubtitle();
                    }}
                    disabled={!hasAudio}
                    class="w-12 h-8 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg text-[11px] font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    title="-5s"
                  >
                    −5s
                  </button>
                  <button
                    onclick={() => {
                      updateOffset(-500);
                      replayCurrentSubtitle();
                    }}
                    disabled={!hasAudio}
                    class="w-12 h-8 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg text-[11px] font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    title="-0.5s"
                  >
                    −0.5s
                  </button>
                  <button
                    onclick={() => {
                      updateOffset(-100);
                      replayCurrentSubtitle();
                    }}
                    disabled={!hasAudio}
                    class="w-8 h-8 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg text-lg font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    title="-100ms"
                  >
                    −
                  </button>
                  <div class="flex flex-col items-center min-w-[80px]">
                    <span class="text-xs text-gray-500 uppercase tracking-wide"
                      >{t("sync.offset")}</span
                    >
                    <span
                      class="text-base font-mono font-medium {offsetAdjustment >
                      0
                        ? 'text-green-400'
                        : offsetAdjustment < 0
                          ? 'text-red-400'
                          : 'text-white'}"
                      >{formatOffset(offsetAdjustment)}</span
                    >
                  </div>
                  <button
                    onclick={() => {
                      updateOffset(100);
                      replayCurrentSubtitle();
                    }}
                    disabled={!hasAudio}
                    class="w-8 h-8 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg text-lg font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    title="+100ms"
                  >
                    +
                  </button>
                  <button
                    onclick={() => {
                      updateOffset(500);
                      replayCurrentSubtitle();
                    }}
                    disabled={!hasAudio}
                    class="w-12 h-8 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg text-[11px] font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    title="+0.5s"
                  >
                    +0.5s
                  </button>
                  <button
                    onclick={() => {
                      updateOffset(5000);
                      replayCurrentSubtitle();
                    }}
                    disabled={!hasAudio}
                    class="w-12 h-8 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg text-[11px] font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    title="+5s"
                  >
                    +5s
                  </button>
                </div>

                <button
                  onclick={confirmCurrentCheckpoint}
                  disabled={!audioSrc || !!audioError}
                  class="btn-success py-3 px-6 flex items-center gap-2 disabled:opacity-50 shadow-lg shadow-green-500/20 text-base font-medium"
                >
                  <svg
                    class="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    ><path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M5 13l4 4L19 7"
                    /></svg
                  >
                  {t("sync.wizard.confirm")}
                </button>
              </div>

              <div
                class="flex flex-wrap gap-3 text-xs text-gray-500 justify-center flex-shrink-0"
              >
                <div class="flex items-center gap-1">
                  <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400"
                    >Space</kbd
                  ><span>{t("sync.playPause")}</span>
                </div>
                <div class="flex items-center gap-1">
                  <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400"
                    >←/→</kbd
                  ><span>{t("sync.seek")}</span>
                </div>
                <div class="flex items-center gap-1">
                  <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400"
                    >↑/↓</kbd
                  ><span>{t("sync.offsetAdjust")}</span>
                </div>
                <div class="flex items-center gap-1">
                  <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400"
                    >R</kbd
                  ><span>{t("sync.wizard.replay")}</span>
                </div>
                <div class="flex items-center gap-1">
                  <kbd class="px-2 py-1 bg-white/10 rounded text-gray-400"
                    >Enter</kbd
                  ><span>{t("sync.confirm")}</span>
                </div>
              </div>
            </div>
          {:else}
            <div class="text-gray-500 text-center">
              <p class="text-lg">{t("sync.wizard.selectCheckpoint")}</p>
            </div>
          {/if}
        </div>
      </div>
    {:else if panelId === "status"}
      <div class="glass-card p-4 space-y-4">
        <div class="flex items-center gap-2">
          <svg
            class="w-5 h-5 text-cyan-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
            /></svg
          >
          <h3 class="text-sm font-semibold text-white">
            {t("sync.statusTitle")}
          </h3>
          <button
            type="button"
            onclick={() => (helpSection = "status")}
            class="ml-auto text-gray-500 hover:text-cyan-300 transition-colors"
            title="Info"
          >
            <svg
              class="w-3.5 h-3.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              /></svg
            >
          </button>
        </div>

        {#if status?.is_loaded}
          <div class="grid grid-cols-2 gap-3">
            <div class="bg-white/5 rounded-xl p-3 text-center">
              <p class="text-2xl font-bold text-white">
                {status.total_subtitles}
              </p>
              <p class="text-xs text-gray-500">{t("sync.subtitles")}</p>
            </div>
            <div class="bg-white/5 rounded-xl p-3 text-center">
              <p class="text-2xl font-bold text-green-400">
                {status.anchor_count}
              </p>
              <p class="text-xs text-gray-500">{t("sync.anchors")}</p>
            </div>
          </div>
          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span class="text-gray-400">{t("sync.averageOffset")}:</span>
              <span
                class={status.average_offset_ms > 0
                  ? "text-green-400"
                  : status.average_offset_ms < 0
                    ? "text-red-400"
                    : "text-white"}
                >{formatOffset(status.average_offset_ms)}</span
              >
            </div>
            <div class="progress-modern h-2">
              <div
                class="progress-modern-bar"
                style="width: {status.completion_percentage}%"
              ></div>
            </div>
            <p class="text-xs text-gray-500 text-center">
              {status.completion_percentage.toFixed(1)}% {t("sync.completed")}
            </p>
          </div>

          <div class="flex gap-2 items-center">
            <input
              type="number"
              min="1"
              max={status.total_subtitles}
              bind:value={manualGoToId}
              placeholder={t("sync.wizard.goToLine")}
              class="flex-1 bg-white/5 border border-white/10 rounded-lg px-3 py-1.5 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-indigo-500"
              onkeydown={(e) => {
                if (e.key === "Enter") goToLineById();
              }}
            />
            <button
              onclick={goToLineById}
              class="btn-secondary py-1.5 px-3 text-sm"
              >{t("sync.wizard.go")}</button
            >
          </div>

          {#if anchors.length > 0}
            <div class="border-t border-white/10 pt-3">
              <h4
                class="text-sm font-semibold text-indigo-400 mb-2 flex items-center gap-2"
              >
                <svg
                  class="w-4 h-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                  ><path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
                  /></svg
                >
                {t("sync.anchors")} ({anchors.length})
              </h4>
              <div class="space-y-2 max-h-32 overflow-y-auto">
                {#each anchors as anchor}
                  <div
                    class="flex items-center justify-between text-sm bg-white/5 rounded-lg px-3 py-2"
                  >
                    <button
                      onclick={() => goToCheckpoint(anchor.subtitle_id)}
                      class="text-gray-400 hover:text-indigo-300 transition-colors"
                      >#{anchor.subtitle_id}</button
                    >
                    <span
                      class={anchor.offset_ms >= 0
                        ? "text-green-400"
                        : "text-red-400"}>{formatOffset(anchor.offset_ms)}</span
                    >
                    <button
                      onclick={() => removeAnchor(anchor.subtitle_id)}
                      class="text-red-400 hover:text-red-300 p-1 hover:bg-red-500/20 rounded transition-colors"
                      aria-label={t("sync.tooltipRemoveAnchor")}
                    >
                      <svg
                        class="w-4 h-4"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        ><path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M6 18L18 6M6 6l12 12"
                        /></svg
                      >
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {:else}
          <p class="text-gray-500 text-sm text-center py-4">
            {t("sync.srtPlaceholder")}
          </p>
        {/if}
      </div>
    {:else if panelId === "subtitleList"}
      <div
        class="glass-card flex-1 overflow-hidden flex flex-col"
        style="min-height: 200px;"
      >
        <div
          class="p-4 border-b border-white/10 flex-shrink-0 flex items-center gap-2"
        >
          <svg
            class="w-4 h-4 text-purple-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 6h16M4 10h16M4 14h16M4 18h16"
            /></svg
          >
          <h4 class="text-sm font-semibold text-purple-400">
            {t("sync.subtitles")}
            {#if status?.is_loaded}<span class="text-gray-500 font-normal"
                >({loadedRangeStart}-{loadedRangeEnd} / {status.total_subtitles})</span
              >{/if}
          </h4>
          <button
            type="button"
            onclick={() => (helpSection = "subtitleList")}
            class="ml-auto text-gray-500 hover:text-purple-300 transition-colors"
            title="Info"
          >
            <svg
              class="w-3.5 h-3.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              /></svg
            >
          </button>
        </div>
        <div
          class="flex-1 overflow-y-auto"
          bind:this={subtitleListElement}
          onscroll={handleSubtitleListScroll}
        >
          {#if isLoadingMore && loadedRangeStart > 1}<div
              class="text-center py-2"
            >
              <span class="text-xs text-gray-500">{t("sync.loading")}</span>
            </div>{/if}
          {#each subtitles as sub (sub.id)}
            <button
              onclick={() => handleSubtitleClick(sub)}
              ondblclick={() => handleSubtitleDblClick(sub)}
              oncontextmenu={(e) => openSubtitleContextMenu(e, sub)}
              data-subtitle-id={sub.id}
              class="w-full text-left p-3 border-b border-white/5 hover:bg-white/5
                {wizardSubtitle?.id === sub.id
                ? 'bg-indigo-500/20 border-l-4 border-l-indigo-500'
                : ''}
                {sub.is_anchor ? 'bg-green-500/5' : ''}"
            >
              <div class="flex items-start gap-2">
                <span class="text-xs text-gray-500 w-8 flex-shrink-0"
                  >#{sub.id}</span
                >
                <div class="flex-1 min-w-0">
                  <p class="text-sm truncate text-gray-200">{sub.text}</p>
                  <div class="flex gap-2 text-xs text-gray-500 mt-1">
                    <span class="font-mono"
                      >{formatTime(sub.synced_start_ms)}</span
                    >
                    <span class="text-gray-700">→</span>
                    <span class="font-mono"
                      >{formatTime(sub.synced_end_ms)}</span
                    >
                    {#if sub.offset_ms !== 0}<span
                        class={sub.offset_ms > 0
                          ? "text-green-400"
                          : "text-red-400"}>{formatOffset(sub.offset_ms)}</span
                      >{/if}
                  </div>
                </div>
                {#if sub.is_anchor}<span class="text-green-400 flex-shrink-0"
                    ><svg
                      class="w-4 h-4"
                      fill="currentColor"
                      viewBox="0 0 24 24"
                      ><path
                        d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
                      /></svg
                    ></span
                  >{/if}
              </div>
            </button>
          {/each}
          {#if isLoadingMore && status && loadedRangeEnd < status.total_subtitles}<div
              class="text-center py-2"
            >
              <span class="text-xs text-gray-500">{t("sync.loading")}</span>
            </div>{/if}
          {#if subtitles.length === 0 && !status?.is_loaded}
            <div class="text-center text-gray-500 py-12">
              <svg
                class="w-12 h-12 mx-auto mb-4 opacity-50"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                ><path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                /></svg
              >
              <p>{t("sync.srtPlaceholder")}</p>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {/snippet}

  <div class="mb-3">
    {@render panelContent("toolbar")}
  </div>

  <div class="flex-1 grid grid-cols-2 gap-6 min-h-0 overflow-hidden">
    <div class="flex flex-col gap-3 overflow-y-auto min-h-[100px]" role="list">
      <div class="flex-1 flex flex-col" role="listitem">
        {@render panelContent("wizard")}
      </div>
    </div>

    <div class="flex flex-col gap-3 overflow-y-auto min-h-[100px]" role="list">
      <div role="listitem">
        {@render panelContent("status")}
      </div>
      <div class="flex-1 flex flex-col" role="listitem">
        {@render panelContent("subtitleList")}
      </div>
    </div>
  </div>

  {#if subtitleContextMenu}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-50"
      onclick={closeSubtitleContextMenu}
      oncontextmenu={(e) => {
        e.preventDefault();
        closeSubtitleContextMenu();
      }}
      onkeydown={(e) => {
        if (e.key === "Escape") closeSubtitleContextMenu();
      }}
      role="presentation"
      tabindex="-1"
    >
      <div
        class="absolute bg-gray-800 border border-white/20 rounded-lg shadow-xl py-1 min-w-[180px] animate-fade-in"
        style="left: {subtitleContextMenu.x}px; top: {subtitleContextMenu.y}px;"
      >
        <button
          onclick={() => {
            if (subtitleContextMenu)
              playSubtitleFromList(subtitleContextMenu.sub);
            closeSubtitleContextMenu();
          }}
          class="w-full text-left px-4 py-2 text-sm text-gray-200 hover:bg-indigo-500/20 hover:text-indigo-300 flex items-center gap-2 transition-colors"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"
            ><path d="M8 5v14l11-7z" /></svg
          >
          {t("sync.playSubtitle")}
        </button>
        <button
          onclick={() => {
            if (subtitleContextMenu)
              goToSubtitleManual(subtitleContextMenu.sub);
            closeSubtitleContextMenu();
          }}
          class="w-full text-left px-4 py-2 text-sm text-gray-200 hover:bg-indigo-500/20 hover:text-indigo-300 flex items-center gap-2 transition-colors"
        >
          <svg
            class="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            ><path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 12a9 9 0 11-18 0 9 9 0 0118 0z"
            /></svg
          >
          {t("sync.goToSubtitle")}
        </button>
      </div>
    </div>
  {/if}

  {#if error}
    <div
      class="fixed bottom-4 right-4 glass-card bg-red-500/20 border border-red-500/30 text-white px-6 py-4 rounded-xl shadow-xl flex items-center gap-3 animate-fade-in"
    >
      <svg
        class="w-5 h-5 text-red-400"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
        ><path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
        /></svg
      >
      <span class="text-red-200">{error}</span>
      <button
        onclick={() => (error = null)}
        class="text-red-400 hover:text-red-300 ml-2"
        aria-label="Close"
        ><svg
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          ><path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M6 18L18 6M6 6l12 12"
          /></svg
        ></button
      >
    </div>
  {/if}

  {#if snackbarMessage}
    <div
      class="fixed bottom-4 left-1/2 -translate-x-1/2 glass-card bg-amber-500/20 border border-amber-500/30 text-amber-200 px-6 py-3 rounded-xl shadow-xl flex items-center gap-3 animate-fade-in z-50"
    >
      <svg
        class="w-5 h-5 text-amber-400 flex-shrink-0"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
        ><path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
        /></svg
      >
      <span>{snackbarMessage}</span>
      <button
        onclick={() => (snackbarMessage = null)}
        class="text-amber-400 hover:text-amber-300 ml-2"
        aria-label="Close"
        ><svg
          class="w-4 h-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          ><path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M6 18L18 6M6 6l12 12"
          /></svg
        ></button
      >
    </div>
  {/if}

  {#if showResetModal}
    <div
      class="absolute inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    >
      <div
        class="p-6 max-w-sm w-full mx-4 shadow-2xl border border-white/10 rounded-2xl"
        style="background: #1e1e2e;"
      >
        <h3 class="text-lg font-semibold text-white mb-2">
          {t("sync.resetSync")}
        </h3>
        <p class="text-gray-400 text-sm mb-6">{t("sync.confirmReset")}</p>
        <div class="flex gap-3 justify-end">
          <button
            onclick={() => (showResetModal = false)}
            class="btn-secondary py-2 px-5 text-sm"
            >{t("sync.cancelReset")}</button
          >
          <button onclick={confirmReset} class="btn-danger py-2 px-5 text-sm"
            >OK</button
          >
        </div>
      </div>
    </div>
  {/if}

  {#if expandedPathField}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-6"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={() => (expandedPathField = null)}
      onkeydown={(e) => {
        if (e.key === "Escape") expandedPathField = null;
      }}
    >
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="bg-gray-900 border border-gray-700 rounded-xl w-full max-w-2xl p-5 animate-fade-in"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold text-gray-300">
            {#if expandedPathField === "srt"}SRT Path
            {:else}Media Path
            {/if}
          </h3>
          <button
            onclick={() => (expandedPathField = null)}
            class="text-gray-400 hover:text-white text-lg leading-none">✕</button
          >
        </div>
        <div class="bg-gray-800/80 rounded-lg p-3 border border-gray-700/50">
          <p class="text-sm text-white font-mono break-all select-all leading-relaxed">
            {#if expandedPathField === "srt"}
              {status?.srt_path || ""}
            {:else}
              {status?.video_path || ""}
            {/if}
          </p>
        </div>
      </div>
    </div>
  {/if}

  {#if helpSection}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-50 bg-black/70 flex items-center justify-center p-6"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={() => (helpSection = null)}
      onkeydown={(e) => {
        if (e.key === "Escape") helpSection = null;
      }}
    >
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="bg-gray-900 border border-gray-700 rounded-xl w-full max-w-2xl p-6"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-bold text-white">
            {#if helpSection === "wizard"}{t("sync.wizard.title")}
            {:else if helpSection === "status"}{t("sync.statusTitle")}
            {:else if helpSection === "subtitleList"}{t("sync.subtitles")}
            {/if}
          </h2>
          <button
            onclick={() => (helpSection = null)}
            class="text-gray-400 hover:text-white text-xl">✕</button
          >
        </div>
        <div
          class="text-gray-300 text-sm leading-relaxed max-h-[60vh] overflow-y-auto"
        >
          {#if helpSection === "wizard"}
            {@html t("sync.wizardHelp")}
          {:else if helpSection === "status"}
            {@html t("sync.statusHelp")}
          {:else if helpSection === "subtitleList"}
            {@html t("sync.subtitleListHelp")}
          {/if}
        </div>
        <div class="mt-4 flex justify-end">
          <button
            onclick={() => (helpSection = null)}
            class="btn-primary py-1.5 px-4 text-sm">OK</button
          >
        </div>
      </div>
    </div>
  {/if}
</div>
