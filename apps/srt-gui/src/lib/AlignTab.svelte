<script lang="ts">
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
  import { t } from './i18n';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import { languages } from './models';

  interface Subtitle {
    id: number;
    start: string;
    end: string;
    text: string;
  }

  let targetPath = $state("");
  let sourcePath = $state("");
  
  let targetSubs: Subtitle[] = $state([]);
  let sourceSubs: Subtitle[] = $state([]);

  let currentPage = $state(0);
  const ITEMS_PER_PAGE_OPTIONS = [5, 10, 15] as const;
  let itemsPerPageIndex = $state(1); // default 10
  let itemsPerPage = $derived(ITEMS_PER_PAGE_OPTIONS[itemsPerPageIndex]);

  function cycleItemsPerPage() {
    // Remember the first visible subtitle index before changing
    const firstVisibleIdx = currentPage * itemsPerPage;
    itemsPerPageIndex = (itemsPerPageIndex + 1) % ITEMS_PER_PAGE_OPTIONS.length;
    // Adjust currentPage so the same subtitle region is visible
    const newPerPage = ITEMS_PER_PAGE_OPTIONS[itemsPerPageIndex];
    currentPage = Math.min(Math.floor(firstVisibleIdx / newPerPage), Math.max(0, Math.ceil(Math.max(targetSubs.length, sourceSubs.length) / newPerPage) - 1));
  }

  let error = $state("");
  let success = $state("");

  // Expanded path modal
  let expandedPathField = $state<string | null>(null);

  // ─── Undo History ──────────────────────────────────────────────────────────
  const MAX_UNDO = 50;
  let undoStack = $state<string[]>([]);
  let undoDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  function pushUndo() {
    // Serialize current sourceSubs state
    const snapshot = JSON.stringify(sourceSubs.map(s => ({ id: s.id, start: s.start, end: s.end, text: s.text })));
    // Don't push if identical to last snapshot
    if (undoStack.length > 0 && undoStack[undoStack.length - 1] === snapshot) return;
    undoStack = [...undoStack.slice(-(MAX_UNDO - 1)), snapshot];
  }

  function scheduleUndo() {
    // Push undo snapshot on first keystroke, debounce subsequent ones
    if (undoDebounceTimer === null) {
      pushUndo();
    } else {
      clearTimeout(undoDebounceTimer);
    }
    undoDebounceTimer = setTimeout(() => {
      undoDebounceTimer = null;
    }, 500);
  }

  function performUndo() {
    if (undoStack.length === 0) return;
    const snapshot = undoStack[undoStack.length - 1];
    undoStack = undoStack.slice(0, -1);
    try {
      const parsed = JSON.parse(snapshot) as Subtitle[];
      sourceSubs = parsed;
    } catch {}
  }

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+Z → undo
    if (e.key === 'z' && (e.ctrlKey || e.metaKey) && !e.shiftKey) {
      e.preventDefault();
      performUndo();
      return;
    }
    // Ctrl+Shift+S → swap files
    if (e.key === 'S' && (e.ctrlKey || e.metaKey) && e.shiftKey) {
      e.preventDefault();
      swapFiles();
      return;
    }
    // Ctrl+Shift+P → cycle items per page
    if (e.key === 'P' && (e.ctrlKey || e.metaKey) && e.shiftKey) {
      e.preventDefault();
      cycleItemsPerPage();
      return;
    }
    // Ctrl+S → save
    if (e.key === 's' && (e.ctrlKey || e.metaKey) && !e.shiftKey) {
      e.preventDefault();
      saveSource();
      return;
    }
    // Tab → next page, Shift+Tab → prev page (only when not in a textarea)
    if (e.key === 'Tab' && !(document.activeElement?.tagName === 'TEXTAREA')) {
      e.preventDefault();
      if (e.shiftKey) {
        prevPage();
      } else {
        nextPage();
      }
      return;
    }
  }

  // ─── Language / Flag Detection ─────────────────────────────────────────────
  const knownLangCodes = new Set(languages.map(l => l.code.toLowerCase()));

  function inferLanguageFromPath(filePath: string): string | null {
    const filename = filePath.split("/").pop()?.toLowerCase() || "";
    const base = filename.replace(/\.[^/.]+$/, "");
    // Split on separators: . - _
    const tokens = base.split(/[.\-_]+/).filter(Boolean);
    // Check from the end for a known language code
    for (let i = tokens.length - 1; i >= 0; i--) {
      if (knownLangCodes.has(tokens[i])) {
        // Return the original-case code from the languages list
        const lang = languages.find(l => l.code.toLowerCase() === tokens[i]);
        if (lang) return lang.code;
      }
    }
    return null;
  }

  function getFlagForPath(path: string): string {
    const code = inferLanguageFromPath(path);
    if (!code) return "";
    const lang = languages.find(l => l.code === code);
    return lang?.flag || "";
  }

  let targetFlag = $derived(targetPath ? getFlagForPath(targetPath) : "");
  let sourceFlag = $derived(sourcePath ? getFlagForPath(sourcePath) : "");

  // ─── Jump to Empty Subtitle ────────────────────────────────────────────────
  function findNextEmptyPage(direction: 'forward' | 'backward'): number | null {
    if (sourceSubs.length === 0) return null;
    const startIdx = direction === 'forward' 
      ? (currentPage + 1) * itemsPerPage 
      : currentPage * itemsPerPage - 1;
    
    if (direction === 'forward') {
      for (let i = startIdx; i < sourceSubs.length; i++) {
        if (!sourceSubs[i].text || sourceSubs[i].text.trim() === '') {
          return Math.floor(i / itemsPerPage);
        }
      }
    } else {
      for (let i = Math.min(startIdx, sourceSubs.length - 1); i >= 0; i--) {
        if (!sourceSubs[i].text || sourceSubs[i].text.trim() === '') {
          return Math.floor(i / itemsPerPage);
        }
      }
    }
    return null;
  }

  let hasEmptyForward = $derived(findNextEmptyPage('forward') !== null);
  let hasEmptyBackward = $derived(findNextEmptyPage('backward') !== null);

  function jumpToNextEmpty() {
    const page = findNextEmptyPage('forward');
    if (page !== null) currentPage = page;
  }

  function jumpToPrevEmpty() {
    const page = findNextEmptyPage('backward');
    if (page !== null) currentPage = page;
  }

  // DnD listener unsubs
  let unlistenFileDrop: () => void;

  onMount(async () => {
    unlistenFileDrop = await listen<{ paths: string[] }>('tauri://file-drop', (event) => {
      const paths = event.payload.paths;
      if (paths && paths.length > 0) {
        handleDroppedFiles(paths);
      }
    });
  });

  onDestroy(() => {
    if (unlistenFileDrop) unlistenFileDrop();
    if (undoDebounceTimer) clearTimeout(undoDebounceTimer);
  });

  function handleDroppedFiles(paths: string[]) {
    // Assign to source or target based on what's empty, or just overwrite first
    for (const p of paths) {
      if (!p.toLowerCase().endsWith('.srt')) continue;
      
      if (!targetPath) {
        loadTarget(p);
      } else if (!sourcePath) {
        loadSource(p);
      } else {
        // Overwrite source if both full
        loadSource(p);
      }
    }
  }

  function parseSrt(content: string): Subtitle[] {
    const blocks = content.trim().replace(/\r\n/g, '\n').split(/\n\s*\n/);
    return blocks.map(block => {
      const lines = block.split('\n');
      const id = parseInt(lines[0], 10) || 0;
      const timeLine = lines[1] || '';
      const times = timeLine.split(' --> ');
      const text = lines.slice(2).join('\n');
      return { id, start: times[0] || '00:00:00,000', end: times[1] || '00:00:00,000', text };
    });
  }

  function serializeSrt(subs: Subtitle[]): string {
    return subs.map(s => `${s.id}\n${s.start} --> ${s.end}\n${s.text}`).join('\n\n') + '\n';
  }

  function normalizeAlignments() {
    // Collect all unique IDs across both arrays
    const targetMap = new Map(targetSubs.map(s => [s.id, s]));
    const sourceMap = new Map(sourceSubs.map(s => [s.id, s]));
    
    // Create a Set of all IDs, sort numerically
    const allIds = Array.from(new Set([...targetMap.keys(), ...sourceMap.keys()])).sort((a, b) => a - b);
    
    // Rebuild the arrays based on the unique IDs, injecting padded items where missing
    targetSubs = allIds.map(id => {
      if (targetMap.has(id)) return targetMap.get(id)!;
      // If missing in target, inject a dummy object
      const s = sourceMap.get(id);
      return { id, start: s?.start || '00:00:00,000', end: s?.end || '00:00:00,000', text: '' };
    });

    sourceSubs = allIds.map(id => {
      if (sourceMap.has(id)) return sourceMap.get(id)!;
      // If missing in source, inject an empty editable item
      const t = targetMap.get(id);
      return { id, start: t?.start || '00:00:00,000', end: t?.end || '00:00:00,000', text: '' };
    });
  }

  async function loadTarget(path: string) {
    try {
      const content = await readTextFile(path);
      targetSubs = parseSrt(content);
      targetPath = path;
      normalizeAlignments();
      error = "";
    } catch (e) {
      error = `Error loading target: ${e}`;
    }
  }

  async function loadSource(path: string) {
    try {
      const content = await readTextFile(path);
      sourceSubs = parseSrt(content);
      sourcePath = path;
      normalizeAlignments();
      undoStack = []; // Reset undo on new file load
      error = "";
    } catch (e) {
      error = `Error loading source: ${e}`;
    }
  }

  async function selectTarget() {
    const selected = await open({
      filters: [{ name: 'Subtitles', extensions: ['srt'] }]
    });
    if (selected && !Array.isArray(selected)) {
      await loadTarget(selected);
    }
  }

  async function selectSource() {
    const selected = await open({
      filters: [{ name: 'Subtitles', extensions: ['srt'] }]
    });
    if (selected && !Array.isArray(selected)) {
      await loadSource(selected);
    }
  }

  function swapFiles() {
    const tempPath = targetPath;
    targetPath = sourcePath;
    sourcePath = tempPath;

    const tempSubs = targetSubs;
    targetSubs = sourceSubs;
    sourceSubs = tempSubs;
    
    normalizeAlignments();
  }

  async function saveSource() {
    try {
      const defaultPath = sourcePath.replace('.srt', '_aligned.srt');
      const savePath = await save({
        defaultPath,
        filters: [{ name: 'Subtitles', extensions: ['srt'] }]
      });

      if (savePath) {
        const content = serializeSrt(sourceSubs);
        await writeTextFile(savePath, content);
        success = `File saved to ${savePath}`;
        setTimeout(() => success = "", 3000);
      }
    } catch (e) {
      error = `Error saving file: ${e}`;
    }
  }

  let totalPages = $derived(Math.ceil(Math.max(targetSubs.length, sourceSubs.length) / itemsPerPage));
  
  // Create padded arrays for the current page so we can iterate side-by-side
  let currentPageItems = $derived(Array.from({ length: itemsPerPage }, (_, i) => {
    const index = currentPage * itemsPerPage + i;
    return {
      index,
      target: targetSubs[index] || null,
      source: sourceSubs[index] || null
    };
  }).filter(item => item.target !== null || item.source !== null));

  function prevPage() {
    if (currentPage > 0) currentPage--;
  }

  function nextPage() {
    if (currentPage < totalPages - 1) currentPage++;
  }

  function jumpStart() {
    currentPage = 0;
  }

  function jumpEnd() {
    if (totalPages > 0) currentPage = totalPages - 1;
  }


  function getFileName(path: string): string {
    return path.split('/').pop() || path;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
  class="h-full flex flex-col pt-6 px-8 pb-8 overflow-hidden animate-fade-in relative text-gray-200 bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950"
  onkeydown={handleKeydown}
>
  <!-- Header -->
  <div class="mb-4 space-y-2 shrink-0">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-teal-400 to-emerald-500">
          Subtitle Alignment
        </h1>
        <p class="text-gray-400 mt-1 flex items-center gap-2">
          Compare and edit subtitles side-by-side
        </p>
      </div>
    </div>
  </div>

  <!-- Error/Success -->
  {#if error}
    <div class="mb-4 p-3 bg-red-500/10 border border-red-500/50 rounded-xl text-red-400 flex items-center shrink-0">
      {error}
      <button class="ml-auto" onclick={() => error = ""}>✕</button>
    </div>
  {/if}
  {#if success}
    <div class="mb-4 p-3 bg-green-500/10 border border-green-500/50 rounded-xl text-green-400 flex items-center shrink-0">
      {success}
      <button class="ml-auto" onclick={() => success = ""}>✕</button>
    </div>
  {/if}

  <!-- File Selection Area -->
  <div class="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] md:items-center gap-4 mb-6 shrink-0 relative">
    
    <!-- Target File -->
    <div class="glass-card p-4 rounded-xl flex flex-col gap-2 relative z-10">
      <div class="text-sm font-semibold text-gray-300 flex items-center gap-2">
        {#if targetFlag}<span class="text-lg">{targetFlag}</span>{/if}
        Target SRT (Reference)
      </div>
      <div class="flex gap-2">
        <button onclick={selectTarget} class="btn-secondary whitespace-nowrap px-4 py-2">Select Target</button>
        <button 
          type="button"
          onclick={() => expandedPathField = "target"}
          class="input-modern flex-1 text-sm text-left cursor-pointer hover:bg-white/10 transition-colors truncate"
          style="direction: rtl; text-align: left;"
          title={targetPath || "Drag & drop SRT here..."}
        >
          <span
            class={targetPath ? "text-white" : "text-gray-500"}
            style="unicode-bidi: plaintext;"
          >
            {targetPath || "Drag & drop SRT here..."}
          </span>
        </button>
      </div>
      {#if targetSubs.length > 0}
        <div class="text-xs text-gray-400">{targetSubs.length} subtitles loaded</div>
      {/if}
    </div>

    <!-- Swap Button -->
    <div class="flex items-center justify-center relative z-20 mx-[-2rem]">
      <button 
        onclick={swapFiles}
        class="p-3 rounded-full bg-gray-800 hover:bg-teal-500/20 text-gray-400 hover:text-teal-400 border border-gray-700 hover:border-teal-500/50 transition-all shadow-xl group relative transform hover:-translate-y-1 hover:scale-110"
        title="Swap Target and Source files"
      >
        <svg class="w-6 h-6 group-hover:rotate-180 transition-transform duration-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
        </svg>
      </button>
    </div>

    <!-- Source File -->
    <div class="glass-card p-4 rounded-xl flex flex-col gap-2 relative z-10">
      <div class="text-sm font-semibold text-gray-300 flex items-center gap-2">
        {#if sourceFlag}<span class="text-lg">{sourceFlag}</span>{/if}
        Source SRT (To Fix)
      </div>
      <div class="flex gap-2">
        <button onclick={selectSource} class="btn-secondary whitespace-nowrap px-4 py-2">Select Source</button>
        <button 
          type="button"
          onclick={() => expandedPathField = "source"}
          class="input-modern flex-1 text-sm text-left cursor-pointer hover:bg-white/10 transition-colors truncate"
          style="direction: rtl; text-align: left;"
          title={sourcePath || "Drag & drop SRT here..."}
        >
          <span
            class={sourcePath ? "text-white" : "text-gray-500"}
            style="unicode-bidi: plaintext;"
          >
            {sourcePath || "Drag & drop SRT here..."}
          </span>
        </button>
      </div>
      {#if sourceSubs.length > 0}
        <div class="text-xs text-gray-400">{sourceSubs.length} subtitles loaded</div>
      {/if}
    </div>
  </div>

  <!-- Editor Area -->
  <div class="flex-1 min-h-[300px] h-full overflow-hidden glass-panel p-4 flex flex-col rounded-xl border border-white/5 bg-gray-900/60 shadow-inner">
    {#if targetSubs.length === 0 && sourceSubs.length === 0}
      <div class="flex-1 flex flex-col items-center justify-center text-gray-500 pb-10">
        <svg class="w-20 h-20 mb-6 opacity-20 text-teal-500 bg-teal-500/5 p-4 rounded-full" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <p class="text-lg font-medium text-gray-400">Load some SRT files to start aligning.</p>
        <p class="text-sm mt-2 text-gray-600">You can drag and drop files anywhere on this window.</p>
      </div>
    {:else}
      <!-- Pagination Top -->
      <div class="flex items-center justify-between mb-4 bg-gray-900/50 p-3 rounded-lg border border-gray-800 shadow-sm shrink-0">
        <div class="flex items-center gap-2">
          <div class="text-sm text-gray-400 font-medium bg-gray-800 px-3 py-1.5 rounded-md">
             Page <span class="text-white mx-1">{currentPage + 1}</span> of <span class="text-white mx-1">{totalPages || 1}</span>
          </div>
          <div class="w-px h-6 bg-gray-700 mx-1"></div>
          <!-- Items per page cycle button -->
          <button
            onclick={cycleItemsPerPage}
            class="flex items-center gap-1.5 bg-gray-800/50 px-3 py-1.5 rounded-md border border-gray-700 hover:border-teal-500/50 hover:bg-teal-500/10 transition-all text-gray-300 hover:text-teal-300"
            title="Cycle visible subtitles per page (5 / 10 / 15)"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" /></svg>
            <span class="text-xs font-medium">{itemsPerPage}</span>
          </button>
        </div>
        <div class="flex gap-1.5">
          <button onclick={jumpStart} disabled={currentPage === 0} class="btn-secondary px-3 py-1.5 disabled:opacity-50 flex items-center" title="Go to Start">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" /></svg>
          </button>
          <!-- Jump to Prev Empty (orange glow) -->
          <button onclick={jumpToPrevEmpty} disabled={!hasEmptyBackward} class="btn-secondary px-3 py-1.5 disabled:opacity-30 flex items-center empty-jump-btn" title="Jump to previous empty subtitle">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7M22 19l-7-7 7-7" /></svg>
          </button>
          <button onclick={prevPage} disabled={currentPage === 0} class="btn-secondary px-4 py-1.5 disabled:opacity-50 flex items-center">
            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>
            Prev
          </button>
          <button onclick={nextPage} disabled={currentPage >= totalPages - 1} class="btn-secondary px-4 py-1.5 disabled:opacity-50 flex items-center">
            Next
            <svg class="w-4 h-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
          </button>
          <!-- Jump to Next Empty (orange glow) -->
          <button onclick={jumpToNextEmpty} disabled={!hasEmptyForward} class="btn-secondary px-3 py-1.5 disabled:opacity-30 flex items-center empty-jump-btn" title="Jump to next empty subtitle">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M2 5l7 7-7 7" /></svg>
          </button>
          <button onclick={jumpEnd} disabled={currentPage >= totalPages - 1} class="btn-secondary px-3 py-1.5 disabled:opacity-50 flex items-center" title="Go to End">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" /></svg>
          </button>
          <div class="w-px h-6 bg-gray-700 mx-1"></div>
          <button onclick={saveSource} class="btn-primary px-6 py-1.5 flex items-center gap-2 shadow-lg shadow-teal-500/20" disabled={sourceSubs.length === 0}>
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" /></svg>
            {#if sourceFlag}<span class="text-base">{sourceFlag}</span>{/if}
            Save Result
          </button>
        </div>
      </div>

      <!-- Content Grid -->
      <div class="flex-1 overflow-y-auto pr-3 pl-1 space-y-4 custom-scrollbar pb-4 min-h-0">
        {#each currentPageItems as item (item.index)}
          <div class="grid grid-cols-2 gap-5 glass-card bg-gray-800/30 p-4 rounded-xl border border-gray-700/50 hover:border-teal-500/30 transition-colors shadow-sm group">
            
            <!-- Target Side (Readonly) -->
            <div class="flex flex-col h-full bg-gray-900/60 rounded-lg border border-gray-800/80 overflow-hidden relative">
              {#if item.target}
                <!-- Header part -->
                <div class="flex justify-between items-center text-xs text-gray-500 bg-gray-900/80 px-3 py-2 border-b border-gray-800 font-mono tracking-wider">
                  <span class="bg-gray-800 px-2 py-0.5 rounded text-gray-400">#{item.target.id}</span>
                  <span class="flex items-center gap-2">
                    <span class="text-teal-400/50">{item.target.start}</span>
                    <svg class="w-3 h-3 mx-1 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"/></svg>
                    <span class="text-emerald-400/50">{item.target.end}</span>
                  </span>
                </div>
                <!-- Content part -->
                <textarea 
                  class="flex-1 w-full bg-transparent p-3 text-sm text-gray-300 resize-none min-h-[90px] focus:outline-none"
                  readonly
                  value={item.target.text}
                ></textarea>
              {:else}
                <div class="h-full min-h-[125px] flex flex-col items-center justify-center text-gray-600 text-sm">
                  <svg class="w-6 h-6 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" /></svg>
                  <span>No Subtitle</span>
                </div>
              {/if}
            </div>

            <!-- Source Side (Editable) -->
            <div class="flex flex-col h-full bg-indigo-950/20 rounded-lg border border-indigo-500/20 focus-within:border-indigo-500/60 focus-within:shadow-[0_0_15px_rgba(99,102,241,0.1)] transition-all overflow-hidden relative">
              {#if item.source}
                 <!-- Header part -->
                 <div class="flex justify-between items-center text-xs text-indigo-400/70 bg-indigo-950/40 px-3 py-2 border-b border-indigo-500/20 font-mono tracking-wider">
                  <span class="bg-indigo-900/50 text-indigo-300 px-2 py-0.5 rounded">#{item.source.id}</span>
                  <span class="flex items-center gap-2">
                    <span class="text-blue-400/70">{item.source.start}</span>
                    <svg class="w-3 h-3 mx-1 text-indigo-500/50" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"/></svg>
                    <span class="text-indigo-400/70">{item.source.end}</span>
                  </span>
                </div>
                <!-- Content part: Svelte 5 runes allow easy bind:value on sourceSubs array -->
                <textarea 
                  class="flex-1 w-full bg-transparent p-3 text-[15px] leading-relaxed text-indigo-100 resize-none min-h-[90px] focus:outline-none placeholder-indigo-900/50"
                  bind:value={sourceSubs[item.index].text}
                  oninput={scheduleUndo}
                  placeholder="Type subtitle here..."
                ></textarea>
              {:else}
                <div class="h-full min-h-[125px] flex flex-col items-center justify-center text-indigo-900/40 text-sm">
                  <svg class="w-6 h-6 mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" /></svg>
                  <span>No Subtitle</span>
                </div>
              {/if}
            </div>
            
          </div>
        {/each}
      </div>
      
    {/if}
  </div>

  <!-- Expanded Path Modal -->
  {#if expandedPathField}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-6"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={() => expandedPathField = null}
      onkeydown={(e) => { if (e.key === "Escape") expandedPathField = null; }}
    >
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="bg-gray-900 border border-gray-700 rounded-xl w-full max-w-2xl p-5 animate-fade-in"
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
      >
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-semibold text-gray-300">
            {#if expandedPathField === "target"}Target SRT Path
            {:else if expandedPathField === "source"}Source SRT Path
            {/if}
          </h3>
          <button
            onclick={() => expandedPathField = null}
            class="text-gray-400 hover:text-white text-lg leading-none"
          >✕</button>
        </div>
        <div class="bg-gray-800/80 rounded-lg p-3 border border-gray-700/50">
          <p class="text-sm text-white font-mono break-all select-all leading-relaxed">
            {#if expandedPathField === "target"}{targetPath || "—"}
            {:else if expandedPathField === "source"}{sourcePath || "—"}
            {/if}
          </p>
        </div>
        <div class="mt-3 flex justify-end">
          <button
            onclick={() => expandedPathField = null}
            class="btn-primary py-1.5 px-4 text-xs"
          >OK</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .empty-jump-btn:not(:disabled) {
    color: #f97316;
    border-color: rgba(249, 115, 22, 0.4);
    filter: drop-shadow(0 0 6px rgba(249, 115, 22, 0.5));
    animation: empty-glow 2s ease-in-out infinite alternate;
  }
  .empty-jump-btn:not(:disabled):hover {
    background-color: rgba(249, 115, 22, 0.15);
    border-color: rgba(249, 115, 22, 0.6);
    filter: drop-shadow(0 0 10px rgba(249, 115, 22, 0.7));
  }
  @keyframes empty-glow {
    0% { filter: drop-shadow(0 0 4px rgba(249, 115, 22, 0.3)); }
    100% { filter: drop-shadow(0 0 8px rgba(249, 115, 22, 0.6)); }
  }
</style>
