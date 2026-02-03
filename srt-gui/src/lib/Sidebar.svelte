<script lang="ts">
  import { locale } from "./i18n";

  interface Props {
    activeTab: "translate" | "sync" | "transcribe" | "flashcards" | "settings" | "shortcuts";
    onTabChange: (tab: "translate" | "sync" | "transcribe" | "flashcards" | "settings" | "shortcuts") => void;
    collapsed?: boolean;
    onToggleCollapse?: () => void;
  }

  let { activeTab, onTabChange, collapsed = false, onToggleCollapse }: Props = $props();
  
  // Reactive translation
  let t = $derived($locale);
</script>

<aside class="{collapsed ? 'w-20' : 'w-72'} bg-gradient-to-b from-gray-900 via-gray-900 to-gray-950 border-r border-white/10 flex flex-col transition-[width] duration-200 ease-out relative will-change-[width]">
  <!-- Collapse Toggle Button -->
  <button
    onclick={onToggleCollapse}
    class="absolute -right-3 top-6 w-6 h-6 bg-gray-800 border border-white/20 rounded-full flex items-center justify-center text-gray-400 hover:text-white hover:bg-gray-700 transition-all z-10 shadow-lg"
    aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
  >
    <svg class="w-3 h-3 transition-transform {collapsed ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
    </svg>
  </button>

  <!-- Logo / Title -->
  <div class="p-6 border-b border-white/10">
    <div class="flex items-center gap-3 {collapsed ? 'justify-center' : ''}">
      <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg flex-shrink-0">
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
        </svg>
      </div>
      {#if !collapsed}
        <div>
          <h1 class="text-xl font-bold bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
            {t("app.title")}
          </h1>
          <p class="text-xs text-gray-500">{t("app.subtitle")}</p>
        </div>
      {/if}
    </div>
  </div>

  <!-- Navigation - Main Features -->
  <nav class="flex-1 p-4 space-y-2 flex flex-col">
    <!-- Active Features -->
    <button
      class="w-full flex items-center gap-3 {collapsed ? 'px-2 justify-center' : 'px-4'} py-3 rounded-xl transition-all duration-300 {activeTab ===
      'translate'
        ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-indigo-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("translate")}
      title={collapsed ? t("nav.translate") : undefined}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'translate' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center flex-shrink-0">
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
      {#if !collapsed}
        <div class="text-left">
          <span class="block font-medium">{t("nav.translate")}</span>
          <span class="text-xs {activeTab === 'translate' ? 'text-white/70' : 'text-gray-500'}">{t("nav.translate.desc")}</span>
        </div>
      {/if}
    </button>

    <button
      class="w-full flex items-center gap-3 {collapsed ? 'px-2 justify-center' : 'px-4'} py-3 rounded-xl transition-all duration-300 {activeTab ===
      'sync'
        ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-indigo-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("sync")}
      title={collapsed ? t("nav.sync") : undefined}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'sync' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center flex-shrink-0">
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
      {#if !collapsed}
        <div class="text-left">
          <span class="block font-medium">{t("nav.sync")}</span>
          <span class="text-xs {activeTab === 'sync' ? 'text-white/70' : 'text-gray-500'}">{t("nav.sync.desc")}</span>
        </div>
      {/if}
    </button>

    <!-- Transcribe - now clickable -->
    <button
      class="w-full flex items-center gap-3 {collapsed ? 'px-2 justify-center' : 'px-4'} py-3 rounded-xl transition-all duration-300 {activeTab ===
      'transcribe'
        ? 'bg-gradient-to-r from-cyan-600 to-blue-600 text-white shadow-lg shadow-cyan-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("transcribe")}
      title={collapsed ? t("nav.transcribe") : undefined}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'transcribe' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center flex-shrink-0 relative">
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
            d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
          />
        </svg>
      </div>
      {#if !collapsed}
        <div class="text-left flex-1">
          <span class="block font-medium {activeTab === 'transcribe' ? 'text-white' : ''}">{t("nav.transcribe")}</span>
          <span class="text-xs {activeTab === 'transcribe' ? 'text-white/70' : 'text-gray-500'}">{t("nav.transcribe.desc")}</span>
        </div>
        <span class="text-[10px] bg-amber-500/20 text-amber-400 px-2 py-0.5 rounded-full font-medium">{t("nav.comingSoon")}</span>
      {/if}
    </button>

    <!-- Flashcards - now clickable -->
    <button
      class="w-full flex items-center gap-3 {collapsed ? 'px-2 justify-center' : 'px-4'} py-3 rounded-xl transition-all duration-300 {activeTab ===
      'flashcards'
        ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-lg shadow-emerald-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("flashcards")}
      title={collapsed ? t("nav.flashcards") : undefined}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'flashcards' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center flex-shrink-0 relative">
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
            d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
          />
        </svg>
      </div>
      {#if !collapsed}
        <div class="text-left flex-1">
          <span class="block font-medium {activeTab === 'flashcards' ? 'text-white' : ''}">{t("nav.flashcards")}</span>
          <span class="text-xs {activeTab === 'flashcards' ? 'text-white/70' : 'text-gray-500'}">{t("nav.flashcards.desc")}</span>
        </div>
        <span class="text-[10px] bg-amber-500/20 text-amber-400 px-2 py-0.5 rounded-full font-medium">{t("nav.comingSoon")}</span>
      {/if}
    </button>

    <!-- Spacer -->
    <div class="flex-1"></div>

    <!-- Separator -->
    <div class="border-t border-white/10 my-2"></div>

    <!-- Settings and Shortcuts at bottom -->
    <button
      class="w-full flex items-center gap-3 {collapsed ? 'px-2 justify-center' : 'px-4'} py-3 rounded-xl transition-all duration-300 {activeTab ===
      'settings'
        ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-indigo-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("settings")}
      title={collapsed ? t("nav.settings") : undefined}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'settings' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center flex-shrink-0">
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
      {#if !collapsed}
        <div class="text-left">
          <span class="block font-medium">{t("nav.settings")}</span>
          <span class="text-xs {activeTab === 'settings' ? 'text-white/70' : 'text-gray-500'}">{t("nav.settings.desc")}</span>
        </div>
      {/if}
    </button>

    <button
      class="w-full flex items-center gap-3 {collapsed ? 'px-2 justify-center' : 'px-4'} py-3 rounded-xl transition-all duration-300 {activeTab ===
      'shortcuts'
        ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-indigo-500/30'
        : 'text-gray-400 hover:bg-white/5 hover:text-white'}"
      onclick={() => onTabChange("shortcuts")}
      title={collapsed ? t("nav.shortcuts") : undefined}
    >
      <div class="w-8 h-8 rounded-lg {activeTab === 'shortcuts' ? 'bg-white/20' : 'bg-white/5'} flex items-center justify-center flex-shrink-0">
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
      {#if !collapsed}
        <div class="text-left">
          <span class="block font-medium">{t("nav.shortcuts")}</span>
          <span class="text-xs {activeTab === 'shortcuts' ? 'text-white/70' : 'text-gray-500'}">{t("nav.shortcuts.desc")}</span>
        </div>
      {/if}
    </button>
  </nav>

  <!-- Footer -->
  <div class="p-4 border-t border-white/10">
    <div class="glass-card p-3 {collapsed ? 'flex items-center justify-center' : ''}">
      {#if collapsed}
        <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
      {:else}
        <div class="flex items-center gap-2 text-xs text-gray-400">
          <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
          <span>{t("app.status.ready")}</span>
        </div>
        <p class="text-xs text-gray-500 mt-1">{t("app.version")}</p>
      {/if}
    </div>
  </div>
</aside>

