<script lang="ts">
  import { locale } from "$lib/i18n";
  import SearchableSelect from "$lib/components/SearchableSelect.svelte";

  export interface DifficultySettings {
    enabled: boolean;
    scheme: "cefr" | "hsk" | "jlpt";
    unknownPolicy: "ignore" | "highest";
  }

  interface Props {
    settings: DifficultySettings;
    studiedLanguage?: string;
  }

  let { settings = $bindable(), studiedLanguage = "zh" }: Props = $props();
  let t = $derived($locale);

  // Auto-preset scheme from studied language if user hasn't explicitly toggled scheme
  $effect(() => {
    if (!settings.enabled) {
      const primary = studiedLanguage.toLowerCase().split(/[-_]/)[0];
      if (primary === "zh") {
        settings.scheme = "hsk";
      } else if (primary === "ja") {
        settings.scheme = "jlpt";
      } else {
        settings.scheme = "cefr";
      }
    }
  });

  let schemeOptions = $derived([
    { value: "hsk", label: "HSK (Chinese 1 - 6/9)" },
    { value: "jlpt", label: "JLPT (Japanese N5 - N1)" },
    { value: "cefr", label: "CEFR (English & Others A1 - C2)" },
  ]);

  let policyOptions = $derived([
    { value: "ignore", label: t("flashcards.difficulty.ignore") },
    { value: "highest", label: t("flashcards.difficulty.highest") },
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
        value={settings.scheme}
        onchange={(val) => {
          if (val === "hsk" || val === "jlpt" || val === "cefr") {
            settings.scheme = val;
          }
        }}
      />
    </div>

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

    <p class="text-[11px] text-gray-400 leading-relaxed bg-white/5 p-2.5 rounded-lg border border-white/10">
      💡 <strong class="text-gray-300">{t("flashcards.difficulty.note")}:</strong> {t("flashcards.difficulty.hintText")}
    </p>
  </div>
</div>
