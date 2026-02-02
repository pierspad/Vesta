<script lang="ts">
  interface Props {
    activeTab: "translate" | "sync" | "settings" | "shortcuts";
    onTabChange: (tab: "translate" | "sync" | "settings" | "shortcuts") => void;
  }

  let { activeTab, onTabChange }: Props = $props();
</script>

<aside class="w-72 bg-gradient-to-b from-gray-900 via-gray-900 to-gray-950 border-r border-white/10 flex flex-col">
  <!-- Logo / Title -->
  <div class="p-6 border-b border-white/10">
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg">
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
        </svg>
      </div>
      <div>
        <h1 class="text-xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
          SRT Tools
        </h1>
        <p class="text-xs text-gray-500">Traduzione & Sincronizzazione</p>
      </div>
    </div>
  </div>

  <!-- Navigation -->
  <nav class="flex-1 p-4 space-y-2">
    <button
      class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 {activeTab ===
      'translate'
        ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-indigo-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("translate")}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'translate' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center">
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
            d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"
          />
        </svg>
      </div>
      <div class="text-left">
        <span class="block font-medium">Traduzione</span>
        <span class="text-xs {activeTab === 'translate' ? 'text-white/70' : 'text-gray-500'}">Traduci sottotitoli con AI</span>
      </div>
    </button>

    <button
      class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 {activeTab ===
      'sync'
        ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-indigo-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("sync")}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'sync' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center">
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
            d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      </div>
      <div class="text-left">
        <span class="block font-medium">Sincronizzazione</span>
        <span class="text-xs {activeTab === 'sync' ? 'text-white/70' : 'text-gray-500'}">Allinea con il video</span>
      </div>
    </button>

    <button
      class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 {activeTab ===
      'settings'
        ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-indigo-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("settings")}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'settings' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center">
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
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
          />
        </svg>
      </div>
      <div class="text-left">
        <span class="block font-medium">Impostazioni</span>
        <span class="text-xs {activeTab === 'settings' ? 'text-white/70' : 'text-gray-500'}">API Keys e modelli</span>
      </div>
    </button>

    <button
      class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 {activeTab ===
      'shortcuts'
        ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-indigo-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("shortcuts")}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'shortcuts' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center">
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
            d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
          />
        </svg>
      </div>
      <div class="text-left">
        <span class="block font-medium">Shortcut</span>
        <span class="text-xs {activeTab === 'shortcuts' ? 'text-white/70' : 'text-gray-500'}">Scorciatoie tastiera</span>
      </div>
    </button>
  </nav>

  <!-- Footer -->
  <div class="p-4 border-t border-white/10">
    <div class="glass-card p-3">
      <div class="flex items-center gap-2 text-xs text-gray-400">
        <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
        <span>Sistema pronto</span>
      </div>
      <p class="text-xs text-gray-500 mt-1">v0.1.0 • Tauri + Svelte</p>
    </div>
  </div>
</aside>

