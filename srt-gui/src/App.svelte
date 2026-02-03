<script lang="ts">
  import Sidebar from "./lib/Sidebar.svelte";
  import TranslateTab from "./lib/TranslateTab.svelte";
  import SyncTab from "./lib/SyncTab.svelte";
  import SettingsTab from "./lib/SettingsTab.svelte";
  import ShortcutsTab from "./lib/ShortcutsTab.svelte";
  import TranscribeTab from "./lib/TranscribeTab.svelte";
  import FlashcardsTab from "./lib/FlashcardsTab.svelte";

  let activeTab = $state<"translate" | "sync" | "transcribe" | "flashcards" | "settings" | "shortcuts">("translate");
  let sidebarCollapsed = $state(false);

  // Expose function to change tab programmatically
  function changeTab(tab: typeof activeTab) {
    activeTab = tab;
  }

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

  // Make available globally for TranslateTab link
  if (typeof window !== 'undefined') {
    (window as any).changeTab = changeTab;
  }
</script>

<main class="flex h-screen bg-gradient-to-br from-gray-900 via-gray-950 to-gray-900 text-gray-100">
  <!-- Sidebar -->
  <Sidebar {activeTab} onTabChange={(tab) => (activeTab = tab)} collapsed={sidebarCollapsed} onToggleCollapse={toggleSidebar} />

  <!-- Main Content - use CSS visibility to preserve state -->
  <div class="flex-1 overflow-hidden relative">
    <div class="absolute inset-0" class:hidden={activeTab !== "translate"}>
      <TranslateTab onGoToSettings={() => (activeTab = "settings")} />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "sync"}>
      <SyncTab />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "transcribe"}>
      <TranscribeTab />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "flashcards"}>
      <FlashcardsTab />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "settings"}>
      <SettingsTab />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "shortcuts"}>
      <ShortcutsTab />
    </div>
  </div>
</main>
