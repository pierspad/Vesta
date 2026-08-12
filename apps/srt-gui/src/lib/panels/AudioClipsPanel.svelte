<script lang="ts">
  import { locale } from "$lib/i18n";
  import { uiMode } from "$lib/stores/uiModeStore.svelte";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";
  import {
    bitratesFor,
    equivalentBitrate,
    formatAudioTrackLabel,
    type AudioFormat,
    type AudioTrackInfo,
    type EpisodeMediaOverrides,
  } from "$lib/types/flashcardMediaTypes";

  interface Props {
    settings: Required<EpisodeMediaOverrides>;
    hasAudio: boolean;
    mediaType: "none" | "video" | "audio";
    audioTracks: AudioTrackInfo[];
    audioTracksLoading: boolean;
    hintLoadMediaFirst: string;
    /** Called (in addition to updating settings.audioTrackIndex) whenever the
     * user manually picks a track, so the caller can stop auto-selecting it. */
    onTrackPicked: () => void;
  }
  let { settings = $bindable(), hasAudio, mediaType, audioTracks, audioTracksLoading, hintLoadMediaFirst, onTrackPicked }: Props = $props();

  let t = $derived($locale);
  let easyMode = $derived(!uiMode.expertMode);

  let bitrateOptions = $derived(
    bitratesFor(settings.audioFormat).map((b) => ({ value: String(b), label: `${b} kb/s` })),
  );

  /** Switching codec carries the quality *intent*, not the number: 128k MP3 and
   *  128k Opus are not the same trade-off, and leaving 128k on Opus would throw
   *  away most of the saving the user just asked for. */
  function setAudioFormat(next: AudioFormat) {
    const previous = settings.audioFormat;
    if (previous === next) return;
    settings.audioBitrate = equivalentBitrate(previous, next, settings.audioBitrate);
    settings.audioFormat = next;
  }
</script>

<div
  inert={!hasAudio}
  title={!hasAudio ? hintLoadMediaFirst : undefined}
  class="glass-card p-5 relative z-30 overflow-visible {!hasAudio ? 'opacity-40' : ''}"
>
  <div class="flex items-center justify-between mb-3">
    <h3 class="text-lg font-semibold flex items-center gap-2 text-cyan-400">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"
        />
      </svg>
      {t("flashcards.generateAudioClips")}
    </h3>
    <button
      onclick={() => {
        if (hasAudio) settings.generateAudio = !settings.generateAudio;
      }}
      class="w-10 h-5 rounded-full transition-all duration-200 relative
        {settings.generateAudio ? 'bg-cyan-500' : 'bg-gray-600'}"
      aria-label="Toggle audio clips"
      disabled={!hasAudio}
    >
      <div
        class="absolute w-4 h-4 bg-white rounded-full top-0.5 transition-all duration-200
        {settings.generateAudio ? 'left-5' : 'left-0.5'}"
      ></div>
    </button>
  </div>

  <div class="space-y-2 transition-all duration-200 {!settings.generateAudio ? 'opacity-40 pointer-events-none' : ''}">
    <div class="grid grid-cols-2 gap-2">
      {#if mediaType === "video" && (audioTracksLoading || audioTracks.length >= 1)}
        <div class={easyMode ? "col-span-2" : ""}>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.audioTrack")}</span>
          {#if audioTracksLoading}
            <div class="input-modern text-xs text-gray-500">
              {t("flashcards.audioTracksLoading")}
            </div>
          {:else if audioTracks.length > 1}
            <SearchableSelect
              noResultsText={t("common.noResults")}
              options={audioTracks.map((track) => ({
                value: String(track.index),
                label: formatAudioTrackLabel(track),
              }))}
              value={settings.audioTrackIndex === null ? "" : String(settings.audioTrackIndex)}
              onchange={(value) => {
                settings.audioTrackIndex = value === "" ? null : Number(value);
                onTrackPicked();
              }}
              placeholder={t("flashcards.audioTrack")}
            />
          {:else}
            <div class="input-modern text-xs text-gray-500 opacity-60 cursor-not-allowed">
              {formatAudioTrackLabel(audioTracks[0])}
            </div>
          {/if}
        </div>
      {/if}

      {#if !easyMode}
        <div class={mediaType === "video" && (audioTracksLoading || audioTracks.length >= 1) ? "" : "col-span-2"}>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.bitrate")}</span>
          <SearchableSelect
            noResultsText={t("common.noResults")}
            options={bitrateOptions}
            value={String(settings.audioBitrate)}
            onchange={(v) => (settings.audioBitrate = parseInt(v))}
            placeholder="Bitrate"
          />
        </div>
      {/if}
    </div>

    <!-- Shown in both modes: shrinking a shared deck is the whole reason Opus
         is here, and that is not an expert-only concern. -->
    <label class="vesta-check-row">
      <input
        type="checkbox"
        checked={settings.audioFormat === "opus"}
        onchange={(e) => setAudioFormat(e.currentTarget.checked ? "opus" : "mp3")}
        class="vesta-check-input shrink-0"
      />
      <span class="min-w-0 text-left text-xs font-medium text-gray-300">
        {t("flashcards.compactAudio")}
      </span>
    </label>
    {#if settings.audioFormat === "opus"}
      <p class="text-[10px] text-amber-500/80 leading-snug">{t("flashcards.opusWarning")}</p>
    {/if}
    {#if !easyMode}
      <div class="grid grid-cols-3 gap-2 items-end">
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.padStart")}</span>
          <div class="flex items-center gap-1">
            <input type="number" bind:value={settings.audioPadStart} class="input-modern w-full text-xs" />
            <span class="text-xs text-gray-500">ms</span>
          </div>
        </div>
        <div>
          <span class="block text-xs text-gray-500 mb-1">{t("flashcards.padEnd")}</span>
          <div class="flex items-center gap-1">
            <input type="number" bind:value={settings.audioPadEnd} class="input-modern w-full text-xs" />
            <span class="text-xs text-gray-500">ms</span>
          </div>
        </div>
        <div class="flex justify-center">
          <label class="vesta-check-row min-h-[42px] w-full">
            <input type="checkbox" bind:checked={settings.normalizeAudio} class="vesta-check-input shrink-0" />
            <span class="min-w-0 text-left text-xs font-medium text-gray-300">{t("flashcards.normalizeAudio")}</span>
          </label>
        </div>
      </div>
    {/if}
  </div>
</div>
