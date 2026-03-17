<script lang="ts">
  import { PhysicalSize } from "@tauri-apps/api/dpi";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import FlashcardsTab from "./lib/FlashcardsTab.svelte";
  import SettingsTab from "./lib/SettingsTab.svelte";
  import ShortcutOverlay from "./lib/ShortcutOverlay.svelte";
  import ShortcutsTab from "./lib/ShortcutsTab.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import SyncTab from "./lib/SyncTab.svelte";
  import TranscribeTab from "./lib/TranscribeTab.svelte";
  import TranslateTab from "./lib/TranslateTab.svelte";
  import AlignTab from "./lib/AlignTab.svelte";

  let activeTab = $state<"translate" | "sync" | "transcribe" | "align" | "flashcards" | "settings" | "shortcuts">("translate");
  let sidebarCollapsed = $state(false);

  const MIN_WIDTH = 1440;
  const MIN_HEIGHT = 940;

  // Enforce minimum window size at runtime (Linux WMs may ignore config)
  onMount(() => {
    const appWindow = getCurrentWindow();
    let unlisten: (() => void) | null = null;

    (async () => {
      const scaleFactor = await appWindow.scaleFactor();
      const physMinW = Math.round(MIN_WIDTH * scaleFactor);
      const physMinH = Math.round(MIN_HEIGHT * scaleFactor);

      await appWindow.setMinSize(new PhysicalSize(physMinW, physMinH)).catch(() => {});

      // Fallback: enforce min size on resize events for WMs that ignore setMinSize
      unlisten = await appWindow.onResized(async ({ payload: size }) => {
        if (size.width < physMinW || size.height < physMinH) {
          const w = Math.max(size.width, physMinW);
          const h = Math.max(size.height, physMinH);
          await appWindow.setSize(new PhysicalSize(w, h)).catch(() => {});
        }
      });
    })();

    return () => { unlisten?.(); };
  });

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

<!-- svelte-ignore a11y_no_static_element_interactions -->
<main
  class="flex h-screen min-w-[1440px] min-h-[940px] bg-gradient-to-br from-gray-900 via-gray-950 to-gray-900 text-gray-100"
  ondragover={(e) => { e.preventDefault(); if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy'; }}
  ondrop={(e) => e.preventDefault()}
>
  <Sidebar {activeTab} onTabChange={(tab) => (activeTab = tab)} collapsed={sidebarCollapsed} onToggleCollapse={toggleSidebar} />

  <!-- Main Content - use CSS visibility to preserve state -->
  <div class="flex-1 overflow-hidden relative">
    <div class="absolute inset-0" class:hidden={activeTab !== "translate"}>
      <TranslateTab onGoToSettings={() => (activeTab = "settings")} active={activeTab === "translate"} />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "sync"}>
      <SyncTab active={activeTab === "sync"} />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "transcribe"}>
      <TranscribeTab onGoToSettings={() => (activeTab = "settings")} />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "align"}>
      <AlignTab />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "flashcards"}>
      <FlashcardsTab active={activeTab === "flashcards"} />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "settings"}>
      <SettingsTab />
    </div>
    <div class="absolute inset-0" class:hidden={activeTab !== "shortcuts"}>
      <ShortcutsTab />
    </div>
  </div>

  <ShortcutOverlay {activeTab} />
</main>
