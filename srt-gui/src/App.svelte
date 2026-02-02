<script lang="ts">
  import Sidebar from "./lib/Sidebar.svelte";
  import TranslateTab from "./lib/TranslateTab.svelte";
  import SyncTab from "./lib/SyncTab.svelte";
  import SettingsTab from "./lib/SettingsTab.svelte";
  import ShortcutsTab from "./lib/ShortcutsTab.svelte";

  let activeTab = $state<"translate" | "sync" | "settings" | "shortcuts">("translate");

  // Expose function to change tab programmatically
  function changeTab(tab: typeof activeTab) {
    activeTab = tab;
  }

  // Make available globally for TranslateTab link
  if (typeof window !== 'undefined') {
    (window as any).changeTab = changeTab;
  }
</script>

<main class="flex h-screen bg-gradient-to-br from-gray-900 via-gray-950 to-gray-900 text-gray-100">
  <!-- Sidebar -->
  <Sidebar {activeTab} onTabChange={(tab) => (activeTab = tab)} />

  <!-- Main Content -->
  <div class="flex-1 overflow-hidden">
    {#if activeTab === "translate"}
      <TranslateTab onGoToSettings={() => (activeTab = "settings")} />
    {:else if activeTab === "sync"}
      <SyncTab />
    {:else if activeTab === "settings"}
      <SettingsTab />
    {:else if activeTab === "shortcuts"}
      <ShortcutsTab />
    {/if}
  </div>
</main>
