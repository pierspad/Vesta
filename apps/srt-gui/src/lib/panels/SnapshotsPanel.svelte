<script lang="ts">
  import { locale } from "$lib/i18n";
  import { uiMode } from "$lib/stores/uiModeStore.svelte";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";
  import {
    QUALITY_STEPS,
    RESOLUTION_PRESETS,
    matchQualityStep,
    matchResolutionPreset,
    type EpisodeMediaOverrides,
    type SnapshotFormat,
  } from "$lib/types/flashcardMediaTypes";

  interface Props {
    settings: Required<EpisodeMediaOverrides>;
    hasVideo: boolean;
    effectiveExportFormat: "tsv" | "apkg" | "anki";
    hintLoadVideoFirst: string;
  }
  let { settings = $bindable(), hasVideo, effectiveExportFormat, hintLoadVideoFirst }: Props = $props();

  let t = $derived($locale);
  let easyMode = $derived(!uiMode.expertMode);

  const CUSTOM = "__custom__";

  // Presets are shortcuts that *write into* the raw values rather than a state
  // living beside them, so the two can never disagree: whatever the numbers
  // say is the truth, and a number no preset produces reads as "Custom".
  let activePreset = $derived(matchResolutionPreset(settings.snapshotWidth, settings.snapshotHeight));
  let activeQuality = $derived(matchQualityStep(settings.snapshotQuality));
  let customResolutionLabel = $derived(
    `${t("flashcards.custom")} (${settings.snapshotWidth}x${settings.snapshotHeight})`,
  );

  function applyResolutionPreset(id: string) {
    const preset = RESOLUTION_PRESETS.find((p) => p.id === id);
    if (!preset) return;
    settings.snapshotWidth = preset.width;
    settings.snapshotHeight = preset.height;
  }

  function applyQualityStep(id: string) {
    const step = QUALITY_STEPS.find((s) => s.id === id);
    if (!step) return;
    settings.snapshotQuality = step.snapshotQuality;
    settings.snapshotWidth = step.snapshotWidth;
    settings.snapshotHeight = step.snapshotHeight;
    if (settings.audioFormat === "opus") settings.audioBitrate = step.opusBitrate;
  }
</script>

<div
  inert={!hasVideo}
  title={!hasVideo ? hintLoadVideoFirst : undefined}
  class="glass-card p-5 relative z-30 overflow-visible {!hasVideo ? 'opacity-40' : ''}"
>
  <div class="flex items-center justify-between mb-3">
    <h3 class="text-lg font-semibold flex items-center gap-2 text-purple-400">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
        />
      </svg>
      <span class="flex flex-col">
        <span>{t("flashcards.generateSnapshots")}</span>
        {#if effectiveExportFormat === "apkg"}
          <span class="text-[10px] text-purple-300/60 font-normal normal-case mt-0.5">
            {t("flashcards.snapshotsExclusiveHint")}
          </span>
        {/if}
      </span>
    </h3>
    <button
      onclick={() => {
        if (hasVideo) settings.generateSnapshots = !settings.generateSnapshots;
      }}
      class="w-10 h-5 rounded-full transition-all duration-200 relative
        {settings.generateSnapshots ? 'bg-purple-500' : 'bg-gray-600'}"
      aria-label="Toggle snapshots"
      disabled={!hasVideo}
    >
      <div
        class="absolute w-4 h-4 bg-white rounded-full top-0.5 transition-all duration-200
        {settings.generateSnapshots ? 'left-5' : 'left-0.5'}"
      ></div>
    </button>
  </div>

  <div class="space-y-2 transition-all duration-200 {!settings.generateSnapshots ? 'opacity-40 pointer-events-none' : ''}">
    {#if easyMode}
      <!-- Easy Mode: Intuitive Quality vs Deck Size Balance Selector -->
      <div class="space-y-3 bg-gray-950/30 p-3.5 rounded-xl border border-purple-500/15">
        <div class="flex items-center justify-between text-xs">
          <span class="font-medium text-gray-300 flex items-center gap-1.5">
            <svg class="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6l3 18h12l3-18H3z" />
            </svg>
            {t("flashcards.qualityVsSize")}
          </span>
          <span class="text-[11px] font-semibold px-2 py-0.5 rounded-full bg-purple-500/20 text-purple-300 border border-purple-500/30">
            {t(`flashcards.quality.${activeQuality?.id ?? "balanced"}`)}
          </span>
        </div>

        <div class="grid grid-cols-3 gap-1.5 p-1 bg-black/40 rounded-xl border border-white/5">
          {#each QUALITY_STEPS as step}
            <button
              type="button"
              onclick={() => applyQualityStep(step.id)}
              class="py-1.5 px-2 rounded-lg text-xs font-medium transition-all duration-150 flex flex-col items-center gap-0.5
                {activeQuality?.id === step.id
                  ? 'bg-purple-600/90 text-white shadow-md shadow-purple-900/40 font-semibold scale-[1.02]'
                  : 'text-gray-400 hover:text-gray-200 hover:bg-white/5'}"
            >
              <span>{t(`flashcards.quality.${step.id}`)}</span>
              <span class="text-[9px] opacity-70">
                {step.id === 'light' ? '~15 KB' : step.id === 'balanced' ? '~35 KB' : '~70 KB'}
              </span>
            </button>
          {/each}
        </div>

        <div class="space-y-1 pt-1">
          <div class="flex justify-between text-[10px] text-gray-400">
            <span class="flex items-center gap-1">
              <svg class="w-3 h-3 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
              </svg>
              {t("flashcards.weightLow")}
            </span>
            <span class="flex items-center gap-1">
              {t("flashcards.weightHigh")}
              <svg class="w-3 h-3 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18" />
              </svg>
            </span>
          </div>
          <div class="h-2 rounded-full bg-gray-800/80 p-0.5 relative overflow-hidden">
            <div
              class="h-full w-full rounded-full transition-[clip-path] duration-300 ease-out"
              style:background="linear-gradient(90deg, #059669 0%, #10b981 18%, #34d399 35%, #22c55e 38%, #84cc16 52%, #eab308 68%, #f97316 85%, #dc2626 100%)"
              style:clip-path="inset(0 {activeQuality?.id === 'light' ? '68%' : activeQuality?.id === 'balanced' ? '32%' : '0%'} 0 0)"
            ></div>
          </div>
        </div>
        <p class="text-[10px] text-gray-400 leading-snug">
          {t(`flashcards.qualityHint.${activeQuality?.id ?? "balanced"}`)}
        </p>
      </div>
    {:else}
      <!-- Expert Mode: Fine-grained controls -->
      <div class="grid grid-cols-2 gap-2">
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.resolution")}</span>
          <SearchableSelect
            noResultsText={t("common.noResults")}
            options={[
              ...RESOLUTION_PRESETS.map((p) => ({
                value: p.id,
                label: `${p.label} (${p.width}x${p.height})`,
              })),
              ...(activePreset ? [] : [{ value: CUSTOM, label: customResolutionLabel }]),
            ]}
            value={activePreset?.id ?? CUSTOM}
            onchange={applyResolutionPreset}
            placeholder={t("flashcards.resolution")}
          />
        </div>
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.quality")}</span>
          <SearchableSelect
            noResultsText={t("common.noResults")}
            options={[
              ...QUALITY_STEPS.map((s) => ({
                value: s.id,
                label: t(`flashcards.quality.${s.id}`),
              })),
              ...(activeQuality ? [] : [{ value: CUSTOM, label: `${t("flashcards.custom")} (${settings.snapshotQuality})` }]),
            ]}
            value={activeQuality?.id ?? CUSTOM}
            onchange={applyQualityStep}
            placeholder={t("flashcards.quality")}
          />
        </div>
      </div>
      <p class="text-[10px] text-gray-500 leading-snug">
        {t(`flashcards.qualityHint.${activeQuality?.id ?? "custom"}`)}
      </p>

      <div class="grid grid-cols-2 gap-2 pt-1">
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.snapshotFormat")}</span>
          <SearchableSelect
            noResultsText={t("common.noResults")}
            options={[
              { value: "webp", label: "WebP" },
              { value: "jpeg", label: "JPEG" },
              { value: "avif", label: "AVIF" },
            ]}
            value={settings.snapshotFormat}
            onchange={(v) => (settings.snapshotFormat = v as SnapshotFormat)}
            placeholder="WebP"
          />
        </div>
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.qualityValue")}</span>
          <div class="flex items-center gap-1">
            <input
              type="number"
              min="0"
              max="100"
              bind:value={settings.snapshotQuality}
              class="input-modern w-full text-xs"
            />
            <span class="text-xs text-gray-500">/100</span>
          </div>
        </div>
      </div>

      {#if settings.snapshotFormat === "avif"}
        <p class="text-[10px] text-amber-500/80 leading-snug">{t("flashcards.avifWarning")}</p>
      {/if}

      <div class="grid grid-cols-3 gap-2">
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.width")}</span>
          <div class="flex items-center gap-1">
            <input type="number" bind:value={settings.snapshotWidth} class="input-modern w-full text-xs" />
            <span class="text-xs text-gray-500">px</span>
          </div>
        </div>
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.height")}</span>
          <div class="flex items-center gap-1">
            <input type="number" bind:value={settings.snapshotHeight} class="input-modern w-full text-xs" />
            <span class="text-xs text-gray-500">px</span>
          </div>
        </div>
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.cropBottom")}</span>
          <div class="flex items-center gap-1">
            <input type="number" bind:value={settings.cropBottom} class="input-modern w-full text-xs" />
            <span class="text-xs text-gray-500">px</span>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
