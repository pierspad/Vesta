<script lang="ts">
  import { onMount } from "svelte";
  import {
    defaultShortcuts,
    getShortcuts,
    saveShortcutOverride,
    resetShortcuts,
    type ShortcutDefinition,
  } from "./models";

  // State
  let shortcuts = $state<ShortcutDefinition[]>([]);
  let editingShortcut = $state<string | null>(null);
  let recordingKey = $state(false);
  let success = $state<string | null>(null);
  let error = $state<string | null>(null);
  let filter = $state<"all" | "global" | "translate" | "sync" | "settings">("all");

  // Filtered shortcuts
  let filteredShortcuts = $derived(
    filter === "all" ? shortcuts : shortcuts.filter((s) => s.category === filter)
  );

  // Group by category
  let groupedShortcuts = $derived(() => {
    const groups: Record<string, ShortcutDefinition[]> = {
      global: [],
      translate: [],
      sync: [],
      settings: [],
    };
    filteredShortcuts.forEach((s) => {
      if (groups[s.category]) {
        groups[s.category].push(s);
      }
    });
    return groups;
  });

  const categoryLabels: Record<string, string> = {
    global: "🌐 Globali",
    translate: "🌍 Traduzione",
    sync: "⏱️ Sincronizzazione",
    settings: "⚙️ Impostazioni",
  };

  const categoryDescriptions: Record<string, string> = {
    global: "Shortcut disponibili ovunque nell'applicazione",
    translate: "Shortcut per la tab di traduzione",
    sync: "Shortcut per la sincronizzazione video",
    settings: "Shortcut per le impostazioni",
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
          error = `Conflitto: "${newKey}" è già usata per "${conflict.description}"`;
          setTimeout(() => (error = null), 3000);
        } else {
          saveShortcutOverride(editingShortcut!, newKey);
          shortcuts = getShortcuts();
          success = `Shortcut aggiornata: ${newKey}`;
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
    if (!confirm("Sei sicuro di voler ripristinare tutte le shortcut predefinite?")) return;
    resetShortcuts();
    shortcuts = getShortcuts();
    success = "Shortcut ripristinate ai valori predefiniti";
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
    success = "Shortcut ripristinata";
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
      Scorciatoie da Tastiera
    </h2>
    <p class="text-gray-400 mt-1">
      Personalizza le shortcut per velocizzare il tuo workflow
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
        Tutte
      </button>
      {#each Object.keys(categoryLabels) as cat}
        <button
          onclick={() => (filter = cat as any)}
          class="px-4 py-2 rounded-lg text-sm font-medium transition-all
            {filter === cat ? 'bg-indigo-500 text-white' : 'bg-white/5 text-gray-400 hover:bg-white/10'}"
        >
          {categoryLabels[cat].split(' ')[0]}
        </button>
      {/each}
    </div>

    <div class="flex-1"></div>

    <button onclick={resetToDefaults} class="btn-secondary py-2 px-4 text-sm">
      <svg class="w-4 h-4 inline mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
      Ripristina Tutte
    </button>
  </div>

  <!-- Shortcuts Grid -->
  <div class="grid grid-cols-2 gap-6 flex-1 overflow-auto">
    {#each Object.entries(groupedShortcuts()) as [category, categoryShortcuts]}
      {#if categoryShortcuts.length > 0}
        <div class="glass-card p-5">
          <div class="mb-4">
            <h3 class="text-lg font-semibold text-white">{categoryLabels[category]}</h3>
            <p class="text-xs text-gray-500 mt-1">{categoryDescriptions[category]}</p>
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
                      Modificata (default: {getDefaultKey(shortcut.id)})
                    </p>
                  {/if}
                </div>

                <div class="flex items-center gap-2">
                  {#if editingShortcut === shortcut.id}
                    <div class="flex items-center gap-2 text-indigo-300 animate-pulse">
                      <span class="text-sm">Premi i tasti...</span>
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
                    <!-- Key Display -->
                    <div class="flex gap-1">
                      {#each formatKey(shortcut.defaultKey) as keyPart}
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

  <!-- Help Info -->
  <div class="mt-6 p-4 bg-white/5 rounded-xl border border-white/10">
    <h4 class="text-sm font-semibold text-white mb-2">💡 Come funziona</h4>
    <ul class="text-sm text-gray-400 space-y-1">
      <li>• Clicca sull'icona ✏️ per modificare una shortcut</li>
      <li>• Premi la combinazione di tasti desiderata</li>
      <li>• Le shortcut modificate sono evidenziate in giallo</li>
      <li>• Usa "Ripristina Tutte" per tornare ai valori predefiniti</li>
    </ul>
  </div>
</div>
