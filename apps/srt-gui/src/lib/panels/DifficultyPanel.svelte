<script lang="ts">
  import { locale } from "$lib/i18n";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";
  import { inferSchemeForLanguage } from "$lib/utils/difficultySchemes";
  import { difficultyStore } from "$lib/stores/difficultyStore.svelte";
  import { getFileName } from "$lib/utils/models";

  export interface DifficultySettings {
    enabled: boolean;
    scheme: string;
    language?: string;
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

  let { settings = $bindable(), studiedLanguage = "en" }: Props = $props();
  let t = $derived($locale);

  // Auto-preset scheme from studied language if user hasn't explicitly chosen custom or another scheme
  $effect(() => {
    if (studiedLanguage && settings.scheme !== "custom" && !settings.customSchemeId) {
      const autoScheme = inferSchemeForLanguage(studiedLanguage);
      if (autoScheme && autoScheme !== settings.scheme && !settings.enabled) {
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
        settings.scheme = inferSchemeForLanguage(studiedLanguage);
        settings.customSchemeId = undefined;
        settings.customFilePath = "";
        settings.customPrefix = "";
      }
    }
  });

  let schemeOptions = $derived.by(() => {
    const builtIn = [
      {
        value: "cefr_en",
        label: "CEFR English (Inglese · A1 - C2)",
        searchTerms: "cefr english inglese cambridge oxford a1 a2 b1 b2 c1 c2 en eng standard",
      },
      {
        value: "cefr_it",
        label: "CEFR Italiano (Italian · A1 - C2)",
        searchTerms: "cefr italiano italian cils celi plida colfis a1 a2 b1 b2 c1 c2 it ita",
      },
      {
        value: "cefr_es",
        label: "CEFR Español (Spagnolo / Spanish · A1 - C2)",
        searchTerms: "cefr espanol spanish spagnolo dele cervantes elelex a1 a2 b1 b2 c1 c2 es spa",
      },
      {
        value: "cefr_fr",
        label: "CEFR Français (Francese / French · A1 - C2)",
        searchTerms: "cefr francais french francese delf dalf fle flelex a1 a2 b1 b2 c1 c2 fr fra fre",
      },
      {
        value: "cefr_de",
        label: "CEFR Deutsch (Tedesco / German · A1 - C2)",
        searchTerms: "cefr deutsch german tedesco goethe telc testdaf a1 a2 b1 b2 c1 c2 de deu ger",
      },
      {
        value: "cefr_ru",
        label: "CEFR Русский (Russo / Russian · A1 - C2)",
        searchTerms: "cefr russian russo trki torfl трки a1 a2 b1 b2 c1 c2 ru rus",
      },
      {
        value: "cefr_pt",
        label: "CEFR Português (Portoghese / Portuguese · A1 - C2)",
        searchTerms: "cefr portugues portuguese portoghese caple celpe a1 a2 b1 b2 c1 c2 pt por",
      },
      {
        value: "hsk",
        label: "HSK (Cinese Semplificato / Mandarin · 1 - 6)",
        searchTerms: "hsk chinese cinese mandarino simplified 汉语 汉语水平考试 hanban hsk1 hsk2 hsk3 hsk4 hsk5 hsk6 zh cmn",
      },
      {
        value: "tocfl",
        label: "TOCFL (Cinese Tradizionale / Taiwan · 1 - 6)",
        searchTerms: "tocfl chinese traditional taiwanese taiwan hong kong 繁體中文 華語文能力測驗 1 2 3 4 5 6 zh-tw zh-hk",
      },
      {
        value: "jlpt",
        label: "JLPT (Giapponese / Japanese · N5 - N1)",
        searchTerms: "jlpt japanese giapponese nihongo 日本語 日本語能力試験 n5 n4 n3 n2 n1 ja jpn",
      },
      {
        value: "topik",
        label: "TOPIK (Coreano / Korean · 1 - 6)",
        searchTerms: "topik korean coreano hangul 한국어 한국어능력시험 nikl 1 2 3 4 5 6 ko kor",
      },
    ];

    const custom = difficultyStore.customSchemes.map((cs) => ({
      value: `custom:${cs.id}`,
      label: `Personalizzato: ${cs.name} (${getFileName(cs.filePath) || cs.filePath})`,
      searchTerms: `custom personalizzato tsv csv ${cs.name} ${cs.tagPrefix || ""}`,
    }));

    return [...builtIn, ...custom];
  });

  let currentSelectedValue = $derived(
    settings.scheme === "custom" && settings.customSchemeId
      ? `custom:${settings.customSchemeId}`
      : (settings.scheme === "cefr" ? "cefr_en" : settings.scheme)
  );

  let policyOptions = $derived([
    {
      value: "ignore",
      label: t("flashcards.difficulty.ignore") || "Ignore unlisted words",
      searchTerms: "ignore ignora parole sconosciute fuori lista",
    },
    {
      value: "highest",
      label: t("flashcards.difficulty.highest") || "Assign highest level to unknown words",
      searchTerms: "highest massimo livello parole sconosciute",
    },
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
        placeholder={t("flashcards.difficulty.schemePlaceholder") || "Cerca o seleziona uno schema di livello..."}
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
          } else {
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

