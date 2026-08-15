<script lang="ts">
  import { locale } from "$lib/i18n";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";
  import { inferSchemeForLanguage } from "$lib/utils/difficultySchemes";
  import { difficultyStore } from "$lib/stores/difficultyStore.svelte";
  import { getFileName } from "$lib/utils/models";

  export interface DifficultySettings {
    enabled: boolean;
    scheme: "cefr" | "hsk" | "jlpt" | "custom" | string;
    customSchemeId?: string;
    unknownPolicy: "ignore" | "highest";
    customFilePath?: string;
    customTsv?: string;
    customPrefix?: string;
  }

  interface Props {
    settings: DifficultySettings;
    studiedLanguage?: string;
  }

  let { settings = $bindable(), studiedLanguage = "zh" }: Props = $props();
  let t = $derived($locale);

  // Auto-preset scheme from studied language if user hasn't explicitly chosen custom or another scheme
  $effect(() => {
    if (!settings.enabled && settings.scheme !== "custom" && !settings.customSchemeId) {
      const autoScheme = inferSchemeForLanguage(studiedLanguage);
      if (autoScheme === "hsk" || autoScheme === "jlpt" || autoScheme === "cefr") {
        settings.scheme = autoScheme;
      }
    }
  });

  // Keep custom scheme synchronized if selected custom scheme was updated/removed
  $effect(() => {
    if (settings.scheme === "custom" && settings.customSchemeId) {
      const found = difficultyStore.getSchemeById(settings.customSchemeId);
      if (found) {
        settings.customFilePath = found.filePath;
        settings.customPrefix = found.tagPrefix || found.name;
      } else if (difficultyStore.customSchemes.length > 0) {
        // Fallback to first available custom scheme
        const first = difficultyStore.customSchemes[0];
        settings.customSchemeId = first.id;
        settings.customFilePath = first.filePath;
        settings.customPrefix = first.tagPrefix || first.name;
      } else {
        // No custom schemes exist anymore, fallback to inferred
        settings.scheme = inferSchemeForLanguage(studiedLanguage) as any;
        settings.customSchemeId = undefined;
        settings.customFilePath = "";
        settings.customPrefix = "";
      }
    }
  });

  let schemeOptions = $derived.by(() => {
    const builtIn = [
      { value: "cefr", label: t("flashcards.difficulty.schemeCefr") || "CEFR (English, Italian & Others A1 - C2)" },
      { value: "hsk", label: t("flashcards.difficulty.schemeHsk") || "HSK (Chinese 1 - 6/9)" },
      { value: "jlpt", label: t("flashcards.difficulty.schemeJlpt") || "JLPT (Japanese N5 - N1)" },
    ];

    const custom = difficultyStore.customSchemes.map((cs) => ({
      value: `custom:${cs.id}`,
      label: `★ ${cs.name} (${getFileName(cs.filePath) || cs.filePath})`,
    }));

    return [...builtIn, ...custom];
  });

  let currentSelectedValue = $derived(
    settings.scheme === "custom" && settings.customSchemeId
      ? `custom:${settings.customSchemeId}`
      : settings.scheme
  );

  let policyOptions = $derived([
    { value: "ignore", label: t("flashcards.difficulty.ignore") || "Ignore unlisted words" },
    { value: "highest", label: t("flashcards.difficulty.highest") || "Assign highest level to unknown words" },
  ]);
</script>

<div class="glass-card p-5">
  <div class="flex items-center justify-between mb-3">
    <h3 class="text-lg font-semibold flex items-center gap-2 text-violet-400">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M3 11l8.586-8.586A2 2 0 0113 2h6a2 2 0 012 2v6a2 2 0 01-.586 1.414L11.828 20a2 2 0 01-2.828 0L3 14a2 2 0 010-3z" />
      </svg>
      {t("flashcards.difficulty.title")}
    </h3>
    <button
      onclick={() => {
        settings.enabled = !settings.enabled;
      }}
      class="w-10 h-5 rounded-full transition-all duration-200 relative {settings.enabled ? 'bg-violet-500' : 'bg-gray-600'}"
      aria-label="Toggle difficulty tagging"
    >
      <div
        class="absolute w-4 h-4 bg-white rounded-full top-0.5 transition-all duration-200 {settings.enabled ? 'left-5' : 'left-0.5'}"
      ></div>
    </button>
  </div>

  <div class="space-y-3 transition-all duration-200 {!settings.enabled ? 'opacity-40 pointer-events-none' : ''}">
    <div>
      <label for="difficulty-scheme-select" class="block text-xs font-semibold text-gray-400 mb-1">
        {t("flashcards.difficulty.schemeLabel")}
      </label>
      <SearchableSelect
        options={schemeOptions}
        value={currentSelectedValue}
        onchange={(val) => {
          if (val.startsWith("custom:")) {
            const schemeId = val.replace("custom:", "");
            const cs = difficultyStore.getSchemeById(schemeId);
            if (cs) {
              settings.scheme = "custom";
              settings.customSchemeId = cs.id;
              settings.customFilePath = cs.filePath;
              settings.customPrefix = cs.tagPrefix || cs.name;
            }
          } else if (val === "hsk" || val === "jlpt" || val === "cefr") {
            settings.scheme = val;
            settings.customSchemeId = undefined;
            settings.customFilePath = "";
            settings.customPrefix = "";
          }
        }}
      />
    </div>

    {#if settings.scheme === "custom" && settings.customFilePath}
      <div class="p-2.5 rounded-lg bg-violet-950/20 border border-violet-500/15 flex items-center gap-2 text-xs text-violet-300">
        <svg class="w-4 h-4 text-violet-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <span class="truncate font-mono text-[11px] text-gray-300" title={settings.customFilePath}>
          {settings.customFilePath}
        </span>
        {#if settings.customPrefix}
          <span class="ml-auto px-1.5 py-0.5 rounded bg-violet-500/20 text-violet-200 border border-violet-500/30 text-[10px] shrink-0 font-mono">
            {settings.customPrefix}::*
          </span>
        {/if}
      </div>
    {/if}

    <div>
      <label for="difficulty-policy-select" class="block text-xs font-semibold text-gray-400 mb-1">
        {t("flashcards.difficulty.unknownPolicyLabel")}
      </label>
      <SearchableSelect
        options={policyOptions}
        value={settings.unknownPolicy}
        onchange={(val) => {
          if (val === "ignore" || val === "highest") {
            settings.unknownPolicy = val;
          }
        }}
      />
    </div>
  </div>
</div>
