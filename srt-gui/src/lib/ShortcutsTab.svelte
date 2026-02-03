<script lang="ts">
  import { onMount } from "svelte";
  import {
    defaultShortcuts,
    getShortcuts,
    saveShortcutOverride,
    resetShortcuts,
    type ShortcutDefinition,
  } from "./models";
  import { locale } from "./i18n";

  // State
  let shortcuts = $state<ShortcutDefinition[]>([]);
  let editingShortcut = $state<string | null>(null);
  let recordingKey = $state(false);
  let success = $state<string | null>(null);
  let error = $state<string | null>(null);
  let filter = $state<"all" | "global" | "translate" | "sync">("all");

  // Reactive translation
  let t = $derived($locale);

  // Filtered shortcuts
  let filteredShortcuts = $derived(
    filter === "all" ? shortcuts : shortcuts.filter((s) => s.category === filter)
  );

  // Group by category (senza settings, spostate in global) - valore diretto, no funzione
  let groupedShortcuts = $derived.by(() => {
    const groups: Record<string, ShortcutDefinition[]> = {
      global: [],
      translate: [],
      sync: [],
    };
    filteredShortcuts.forEach((s) => {
      if (groups[s.category]) {
        groups[s.category].push(s);
      }
    });
    return groups;
  });

  // Split sync shortcuts in two columns - valori diretti
  let syncShortcutsLeft = $derived.by(() => {
    const syncShorts = groupedShortcuts.sync;
    return syncShorts.slice(0, Math.ceil(syncShorts.length / 2));
  });

  let syncShortcutsRight = $derived.by(() => {
    const syncShorts = groupedShortcuts.sync;
    return syncShorts.slice(Math.ceil(syncShorts.length / 2));
  });

  const categoryLabels: Record<string, string> = {
    global: "shortcuts.category.global",
    translate: "shortcuts.category.translate",
    sync: "shortcuts.category.sync",
  };

  const categoryDescriptions: Record<string, string> = {
    global: "shortcuts.category.global.desc",
    translate: "shortcuts.category.translate.desc",
    sync: "shortcuts.category.sync.desc",
  };

  onMount(() => {
    shortcuts = getShortcuts();
  });

  function startEditing(shortcutId: string) {
    editingShortcut = shortcutId;
    recordingKey = true;
    
    // Listen for key press
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      
      // Build key string
      const parts: string[] = [];
      if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      
      // Get key name
      let keyName = e.key;
      if (keyName === " ") keyName = "Space";
      else if (keyName.length === 1) keyName = keyName.toUpperCase();
      else if (keyName.startsWith("Arrow")) keyName = keyName;
      
      if (!["Control", "Alt", "Shift", "Meta"].includes(e.key)) {
        parts.push(keyName);
        const newKey = parts.join("+");
        
        // Check for conflicts
        const conflict = shortcuts.find(
          (s) => s.id !== editingShortcut && s.defaultKey === newKey
        );
        
        if (conflict) {
          error = t("shortcuts.conflict", { key: newKey, action: conflict.description });
          setTimeout(() => (error = null), 3000);
        } else {
          saveShortcutOverride(editingShortcut!, newKey);
          shortcuts = getShortcuts();
          success = t("shortcuts.updated", { key: newKey });
          setTimeout(() => (success = null), 3000);
        }
        
        editingShortcut = null;
        recordingKey = false;
        window.removeEventListener("keydown", handler);
      }
    };
    
    window.addEventListener("keydown", handler);
  }

  function cancelEditing() {
    editingShortcut = null;
    recordingKey = false;
  }

  function resetToDefaults() {
    if (!confirm(t("shortcuts.confirmReset"))) return;
    resetShortcuts();
    shortcuts = getShortcuts();
    success = t("shortcuts.reset");
    setTimeout(() => (success = null), 3000);
  }

  function getDefaultKey(shortcutId: string): string {
    return defaultShortcuts.find((s) => s.id === shortcutId)?.defaultKey || "";
  }

  function isModified(shortcut: ShortcutDefinition): boolean {
    const defaultKey = getDefaultKey(shortcut.id);
    return shortcut.defaultKey !== defaultKey;
  }

  function resetSingle(shortcutId: string) {
    const defaultKey = getDefaultKey(shortcutId);
    saveShortcutOverride(shortcutId, defaultKey);
    shortcuts = getShortcuts();
    success = t("shortcuts.resetSingle");
    setTimeout(() => (success = null), 3000);
  }

  function formatKey(key: string): string[] {
    return key.split("+");
  }
</script>

<div class="h-full flex flex-col p-6 overflow-auto bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950">
  <!-- Header -->
  <div class="mb-6">
    <h2 class="text-3xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
      {t("shortcuts.title")}
    </h2>
    <p class="text-gray-400 mt-1">
      {t("shortcuts.subtitle")}
    </p>
  </div>

  <!-- Notifications -->
  {#if error}
    <div class="mb-4 p-4 bg-red-500/10 border border-red-500/30 rounded-xl flex items-center gap-3 animate-fade-in">
      <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <span class="text-red-300 flex-1">{error}</span>
      <button onclick={() => (error = null)} class="text-red-400 hover:text-red-300">✕</button>
    </div>
  {/if}

  {#if success}
    <div class="mb-4 p-4 bg-green-500/10 border border-green-500/30 rounded-xl flex items-center gap-3 animate-fade-in">
      <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
      </svg>
      <span class="text-green-300">{success}</span>
    </div>
  {/if}

  <!-- Toolbar -->
  <div class="flex items-center gap-4 mb-6">
    <!-- Category Filter -->
    <div class="flex gap-2">
      <button
        onclick={() => (filter = "all")}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-all
          {filter === 'all' ? 'bg-indigo-500 text-white' : 'bg-white/5 text-gray-400 hover:bg-white/10'}"
      >
        {t("shortcuts.filter.all")}
      </button>
      {#each Object.keys(categoryLabels) as cat}
        <button
          onclick={() => (filter = cat as any)}
          class="px-4 py-2 rounded-lg text-sm font-medium transition-all
            {filter === cat ? 'bg-indigo-500 text-white' : 'bg-white/5 text-gray-400 hover:bg-white/10'}"
        >
          {t(categoryLabels[cat]).split(' ')[0]}
        </button>
      {/each}
    </div>

    <div class="flex-1"></div>

    <button onclick={resetToDefaults} class="btn-secondary py-2 px-4 text-sm">
      <svg class="w-4 h-4 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
      {t("shortcuts.resetAll")}
    </button>
  </div>

  <!-- Shortcuts Grid - Global and Translate -->
  <div class="grid grid-cols-2 gap-6">
    {#each Object.entries(groupedShortcuts).filter(([cat]) => cat !== 'sync') as [category, categoryShortcuts]}
      {#if categoryShortcuts.length > 0}
        <div class="glass-card p-5">
          <div class="mb-4">
            <h3 class="text-lg font-semibold text-white">{t(categoryLabels[category])}</h3>
            <p class="text-xs text-gray-500 mt-1">{t(categoryDescriptions[category])}</p>
          </div>

          <div class="space-y-2">
            {#each categoryShortcuts as shortcut}
              <div
                class="flex items-center justify-between p-3 rounded-lg transition-all
                  {editingShortcut === shortcut.id ? 'bg-indigo-500/20 ring-1 ring-indigo-500' : 'bg-white/5 hover:bg-white/10'}"
              >
                <div class="flex-1">
                  <p class="text-sm text-white">{shortcut.description}</p>
                  {#if isModified(shortcut)}
                    <p class="text-xs text-amber-400 mt-1">
                      {t("shortcuts.modified", { key: getDefaultKey(shortcut.id) })}
                    </p>
                  {/if}
                </div>

                <div class="flex items-center gap-2">
                  {#if editingShortcut === shortcut.id}
                    <div class="flex items-center gap-2 text-indigo-300 animate-pulse">
                      <span class="text-sm">{t("shortcuts.pressKeys")}</span>
                      <button
                        onclick={cancelEditing}
                        class="text-gray-400 hover:text-white p-1"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  {:else}
                    <!-- Key Display with + separator -->
                    <div class="flex items-center gap-1">
                      {#each formatKey(shortcut.defaultKey) as keyPart, i}
                        {#if i > 0}
                          <span class="text-gray-500 text-xs font-bold">+</span>
                        {/if}
                        <kbd class="px-2 py-1 bg-gray-800 border border-gray-600 rounded text-xs text-gray-300 font-mono shadow-md">
                          {keyPart}
                        </kbd>
                      {/each}
                    </div>

                    <!-- Edit Button -->
                    <button
                      onclick={() => startEditing(shortcut.id)}
                      class="p-1.5 text-gray-500 hover:text-indigo-400 hover:bg-white/5 rounded transition-colors"
                      title="Modifica shortcut"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>

                    {#if isModified(shortcut)}
                      <button
                        onclick={() => resetSingle(shortcut.id)}
                        class="p-1.5 text-gray-500 hover:text-amber-400 hover:bg-white/5 rounded transition-colors"
                        title="Ripristina default"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                      </button>
                    {/if}
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    {/each}
  </div>

  <!-- Sync Shortcuts - Split in 2 columns -->
  {#if groupedShortcuts.sync.length > 0}
    <div class="mt-6">
      <div class="glass-card p-5">
        <div class="mb-4">
          <h3 class="text-lg font-semibold text-white">{t("shortcuts.category.sync")}</h3>
          <p class="text-xs text-gray-500 mt-1">{t("shortcuts.category.sync.desc")}</p>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <!-- Left column -->
          <div class="space-y-2">
            {#each syncShortcutsLeft as shortcut}
              <div
                class="flex items-center justify-between p-3 rounded-lg transition-all
                  {editingShortcut === shortcut.id ? 'bg-indigo-500/20 ring-1 ring-indigo-500' : 'bg-white/5 hover:bg-white/10'}"
              >
                <div class="flex-1">
                  <p class="text-sm text-white">{shortcut.description}</p>
                  {#if isModified(shortcut)}
                    <p class="text-xs text-amber-400 mt-1">
                      {t("shortcuts.modified", { key: getDefaultKey(shortcut.id) })}
                    </p>
                  {/if}
                </div>

                <div class="flex items-center gap-2">
                  {#if editingShortcut === shortcut.id}
                    <div class="flex items-center gap-2 text-indigo-300 animate-pulse">
                      <span class="text-sm">{t("shortcuts.pressKeys")}</span>
                      <button onclick={cancelEditing} class="text-gray-400 hover:text-white p-1">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  {:else}
                    <div class="flex items-center gap-1">
                      {#each formatKey(shortcut.defaultKey) as keyPart, i}
                        {#if i > 0}
                          <span class="text-gray-500 text-xs font-bold">+</span>
                        {/if}
                        <kbd class="px-2 py-1 bg-gray-800 border border-gray-600 rounded text-xs text-gray-300 font-mono shadow-md">
                          {keyPart}
                        </kbd>
                      {/each}
                    </div>
                    <button
                      onclick={() => startEditing(shortcut.id)}
                      class="p-1.5 text-gray-500 hover:text-indigo-400 hover:bg-white/5 rounded transition-colors"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    {#if isModified(shortcut)}
                      <button onclick={() => resetSingle(shortcut.id)} class="p-1.5 text-gray-500 hover:text-amber-400 hover:bg-white/5 rounded transition-colors">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                      </button>
                    {/if}
                  {/if}
                </div>
              </div>
            {/each}
          </div>

          <!-- Right column -->
          <div class="space-y-2">
            {#each syncShortcutsRight as shortcut}
              <div
                class="flex items-center justify-between p-3 rounded-lg transition-all
                  {editingShortcut === shortcut.id ? 'bg-indigo-500/20 ring-1 ring-indigo-500' : 'bg-white/5 hover:bg-white/10'}"
              >
                <div class="flex-1">
                  <p class="text-sm text-white">{shortcut.description}</p>
                  {#if isModified(shortcut)}
                    <p class="text-xs text-amber-400 mt-1">
                      {t("shortcuts.modified", { key: getDefaultKey(shortcut.id) })}
                    </p>
                  {/if}
                </div>

                <div class="flex items-center gap-2">
                  {#if editingShortcut === shortcut.id}
                    <div class="flex items-center gap-2 text-indigo-300 animate-pulse">
                      <span class="text-sm">{t("shortcuts.pressKeys")}</span>
                      <button onclick={cancelEditing} class="text-gray-400 hover:text-white p-1">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  {:else}
                    <div class="flex items-center gap-1">
                      {#each formatKey(shortcut.defaultKey) as keyPart, i}
                        {#if i > 0}
                          <span class="text-gray-500 text-xs font-bold">+</span>
                        {/if}
                        <kbd class="px-2 py-1 bg-gray-800 border border-gray-600 rounded text-xs text-gray-300 font-mono shadow-md">
                          {keyPart}
                        </kbd>
                      {/each}
                    </div>
                    <button
                      onclick={() => startEditing(shortcut.id)}
                      class="p-1.5 text-gray-500 hover:text-indigo-400 hover:bg-white/5 rounded transition-colors"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    {#if isModified(shortcut)}
                      <button onclick={() => resetSingle(shortcut.id)} class="p-1.5 text-gray-500 hover:text-amber-400 hover:bg-white/5 rounded transition-colors">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                      </button>
                    {/if}
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
