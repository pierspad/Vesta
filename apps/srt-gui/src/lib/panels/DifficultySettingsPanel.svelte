<script lang="ts">
  import { locale } from "$lib/i18n";
  import { snackbar } from "$lib/stores/snackbarStore.svelte";
  import { SCHEMES, CEFR_LANG_MAP, exportSchemeTsv } from "$lib/utils/difficultySchemes";
  import { guardedOpen } from "$lib/utils/dialogGuard";
  import { difficultyStore } from "$lib/stores/difficultyStore.svelte";
  import { getFileName } from "$lib/utils/models";

  let t = $derived($locale);

  // Custom Schemes management in Settings
  let showAddCustomScheme = $state(false);
  let newSchemeName = $state("");
  let newSchemeFilePath = $state("");
  let newSchemeTagPrefix = $state("");

  let newSchemeNameTrimmed = $derived(newSchemeName.trim());
  let isNewSchemeNameValid = $derived(newSchemeNameTrimmed.length > 0 && difficultyStore.isNameAvailable(newSchemeNameTrimmed));
  let isNewSchemeDuplicate = $derived(newSchemeNameTrimmed.length > 0 && !difficultyStore.isNameAvailable(newSchemeNameTrimmed));
  let canSaveNewScheme = $derived(isNewSchemeNameValid && newSchemeFilePath.trim().length > 0);

  async function pickCustomSchemeFile() {
    try {
      const selected = await guardedOpen({
        title: t("settings.difficulty.selectFile") || "Select Vocabulary List File",
        filters: [
          { name: "Vocabulary Lists (*.tsv, *.csv, *.txt)", extensions: ["tsv", "csv", "txt"] },
          { name: "All Files", extensions: ["*"] },
        ],
      });
      if (selected && typeof selected === "string") {
        newSchemeFilePath = selected;
        if (!newSchemeName.trim()) {
          const fileName = getFileName(selected);
          const inferredName = fileName.replace(/\.[^/.]+$/, "").replace(/[-_]/g, " ");
          if (inferredName && difficultyStore.isNameAvailable(inferredName)) {
            newSchemeName = inferredName;
          }
        }
      }
    } catch (err) {
      console.error("Failed to select custom vocabulary file:", err);
    }
  }

  function saveCustomScheme() {
    if (!canSaveNewScheme) return;
    const res = difficultyStore.addCustomScheme({
      name: newSchemeName,
      filePath: newSchemeFilePath,
      tagPrefix: newSchemeTagPrefix,
    });
    if (res.success) {
      snackbar.show(t("settings.difficulty.schemeAddedSuccess") || "Custom vocabulary added successfully", "success", 2000);
      newSchemeName = "";
      newSchemeFilePath = "";
      newSchemeTagPrefix = "";
      showAddCustomScheme = false;
    } else {
      snackbar.show(res.error || t("settings.difficulty.duplicateNameError"), "error", 3000);
    }
  }

  function cancelAddCustomScheme() {
    newSchemeName = "";
    newSchemeFilePath = "";
    newSchemeTagPrefix = "";
    showAddCustomScheme = false;
  }

  function removeCustomScheme(id: string) {
    difficultyStore.removeCustomScheme(id);
    snackbar.show(t("settings.difficulty.schemeRemovedSuccess") || "Vocabulary removed", "info", 2000);
  }
</script>

<div class="glass-card p-6 space-y-6">
  <!-- Difficulty Schemes Header -->
  <div class="flex items-center gap-3">
    <div class="w-9 h-9 rounded-lg bg-violet-500/20 text-violet-300 flex items-center justify-center shrink-0">
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M3 11l8.586-8.586A2 2 0 0113 2h6a2 2 0 012 2v6a2 2 0 01-.586 1.414L11.828 20a2 2 0 01-2.828 0L3 14a2 2 0 010-3z" />
      </svg>
    </div>
    <div>
      <h3 class="text-sm font-bold text-white">{t("settings.difficulty.title") || "Difficulty Schemes & Vocabulary Databases"}</h3>
    </div>
  </div>

  <!-- Built-in Schemes List -->
  <div class="space-y-3">
    <div class="text-xs font-semibold text-gray-300 flex items-center gap-2">
      <span>{t("settings.difficulty.standardTitle")}</span>
    </div>
    {#each SCHEMES as scheme}
      <div class="p-3.5 rounded-lg bg-white/5 border border-white/10 flex flex-col gap-3">
        <div class="flex flex-col md:flex-row md:items-center justify-between gap-3">
          <div class="space-y-1">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-violet-300">
                {scheme.id === "cefr" ? t("settings.difficulty.schemeCefrName") :
                 scheme.id === "hsk" ? t("settings.difficulty.schemeHskName") :
                 scheme.id === "tocfl" ? t("settings.difficulty.schemeTocflName") :
                 scheme.id === "jlpt" ? t("settings.difficulty.schemeJlptName") :
                 scheme.id === "topik" ? t("settings.difficulty.schemeTopikName") : scheme.name}
              </span>
              <span class="text-[10px] px-1.5 py-0.5 rounded bg-violet-500/20 text-violet-200 border border-violet-500/30 font-mono">{scheme.levels}</span>
            </div>
            <p class="text-xs text-gray-400">
              {scheme.id === "cefr" ? t("settings.difficulty.schemeCefrDesc") :
               scheme.id === "hsk" ? t("settings.difficulty.schemeHskDesc") :
               scheme.id === "tocfl" ? t("settings.difficulty.schemeTocflDesc") :
               scheme.id === "jlpt" ? t("settings.difficulty.schemeJlptDesc") :
               scheme.id === "topik" ? t("settings.difficulty.schemeTopikDesc") : scheme.description}
            </p>
            <p class="text-[11px] text-gray-500">
              <strong class="text-gray-400">{t("settings.difficulty.languages") || "Languages"}:</strong>
              {scheme.id === "cefr" ? t("settings.difficulty.schemeCefrLangs") :
               scheme.id === "hsk" ? t("settings.difficulty.schemeHskLangs") :
               scheme.id === "tocfl" ? t("settings.difficulty.schemeTocflLangs") :
               scheme.id === "jlpt" ? t("settings.difficulty.schemeJlptLangs") :
               scheme.id === "topik" ? t("settings.difficulty.schemeTopikLangs") : scheme.langCoverage}
            </p>
          </div>
          <div class="shrink-0 flex items-center gap-2 self-start md:self-center">
            <button
              type="button"
              onclick={() => exportSchemeTsv(scheme.id)}
              class="px-3 py-1.5 rounded-lg border border-violet-500/30 bg-violet-500/10 hover:bg-violet-500/20 text-violet-200 text-xs font-semibold flex items-center gap-1.5 transition-colors cursor-pointer"
              title={t("settings.difficulty.exportTooltip") || "Download starter TSV template"}
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              <span>{t("settings.difficulty.exportButton") || "Download TSV Template"}</span>
            </button>
          </div>
        </div>

        {#if scheme.id === "cefr"}
          <div class="pt-2 border-t border-white/5 flex flex-wrap items-center gap-1.5">
            <span class="text-[10px] text-gray-500 font-medium mr-1">{t("settings.difficulty.cefrLanguageSelect") || "Download specific language:"}</span>
            {#each Object.entries(CEFR_LANG_MAP) as [code, info]}
              <button
                type="button"
                onclick={() => exportSchemeTsv("cefr", code)}
                class="px-2 py-0.5 rounded-md border border-white/10 bg-white/5 hover:bg-violet-500/20 hover:border-violet-500/30 text-gray-300 hover:text-violet-200 text-[11px] font-medium transition-colors cursor-pointer"
                title={`Scarica database CEFR ${info.label} TSV`}
              >
                {info.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Custom Schemes Section (Subtitle removed as requested) -->
  <div class="pt-3 space-y-3 border-t border-white/10">
    <div class="flex items-center justify-between gap-2">
      <h4 class="text-xs font-bold text-white flex items-center gap-1.5">
        <svg class="w-3.5 h-3.5 text-violet-400" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
        </svg>
        <span>{t("settings.difficulty.customTitle")}</span>
      </h4>
      {#if !showAddCustomScheme}
        <button
          type="button"
          onclick={() => (showAddCustomScheme = true)}
          class="px-3 py-1.5 rounded-lg bg-violet-600 hover:bg-violet-500 text-white font-semibold text-xs transition-colors flex items-center gap-1.5 shrink-0 shadow-sm cursor-pointer"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          <span>{t("settings.difficulty.addCustomScheme")}</span>
        </button>
      {/if}
    </div>

    <!-- Add Custom Scheme Form -->
    {#if showAddCustomScheme}
      <div class="p-4 rounded-xl bg-violet-950/20 border border-violet-500/30 space-y-3 shadow-md">
        <div class="flex items-center justify-between pb-1 border-b border-violet-500/20">
          <span class="text-xs font-bold text-violet-200 flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 text-violet-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            <span>{t("settings.difficulty.addCustomScheme")}</span>
          </span>
          <button
            type="button"
            onclick={cancelAddCustomScheme}
            class="p-1 rounded-md text-gray-400 hover:text-white hover:bg-white/10 text-xs transition-colors cursor-pointer"
            title={t("common.cancel") || "Close"}
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <!-- Scheme Name -->
          <div>
            <label for="new-scheme-name-input" class="block text-[11px] font-semibold text-gray-300 mb-1">
              {t("settings.difficulty.schemeNameLabel")} <span class="text-rose-400">*</span>
            </label>
            <input
              id="new-scheme-name-input"
              type="text"
              bind:value={newSchemeName}
              placeholder={t("settings.difficulty.schemeNamePlaceholder")}
              class="w-full h-8.5 rounded-lg bg-black/50 border {isNewSchemeDuplicate ? 'border-rose-500/80 focus:border-rose-500' : 'border-white/15 focus:border-violet-500'} px-2.5 text-xs text-white placeholder-gray-500 outline-none transition-colors"
            />
            {#if isNewSchemeDuplicate}
              <span class="block text-[10px] text-rose-400 mt-1">
                {t("settings.difficulty.duplicateNameError")}
              </span>
            {/if}
          </div>

          <!-- Tag Prefix -->
          <div>
            <label for="new-scheme-prefix-input" class="block text-[11px] font-semibold text-gray-300 mb-1">
              {t("settings.difficulty.customPrefixOptional")}
            </label>
            <input
              id="new-scheme-prefix-input"
              type="text"
              bind:value={newSchemeTagPrefix}
              placeholder={t("settings.difficulty.customPrefixPlaceholder")}
              class="w-full h-8.5 rounded-lg bg-black/50 border border-white/15 focus:border-violet-500 px-2.5 text-xs text-white placeholder-gray-500 outline-none transition-colors"
            />
            <span class="block text-[10px] text-gray-500 mt-1">
              {t("settings.difficulty.tagPreview").replace("{{prefix}}", newSchemeTagPrefix.trim() || newSchemeName.trim() || "Tag")}
            </span>
          </div>
        </div>

        <!-- File Picker -->
        <div>
          <span class="block text-[11px] font-semibold text-gray-300 mb-1">
            {t("settings.difficulty.selectFile")} <span class="text-rose-400">*</span>
          </span>
          <div class="flex items-center gap-2">
            <input
              type="text"
              readonly
              value={newSchemeFilePath}
              placeholder={t("flashcards.difficulty.selectCustomFile") || "Select a .tsv, .csv, or .txt file"}
              class="flex-1 h-8.5 rounded-lg bg-black/50 border border-white/15 px-3 text-xs text-gray-200 placeholder-gray-500 outline-none font-mono truncate"
            />
            <button
              type="button"
              onclick={pickCustomSchemeFile}
              class="h-8.5 px-3 rounded-lg bg-white/10 hover:bg-white/20 text-white font-medium text-xs transition-colors shrink-0 flex items-center gap-1.5 border border-white/10 cursor-pointer"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              <span>{t("common.browse") || "Browse"}</span>
            </button>
          </div>
        </div>

        <!-- Form Buttons -->
        <div class="flex items-center justify-end gap-2 pt-2 border-t border-violet-500/20">
          <button
            type="button"
            onclick={cancelAddCustomScheme}
            class="px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-gray-300 text-xs font-medium transition-colors cursor-pointer"
          >
            {t("common.cancel")}
          </button>
          <button
            type="button"
            onclick={saveCustomScheme}
            disabled={!canSaveNewScheme}
            class="px-4 py-1.5 rounded-lg bg-violet-600 hover:bg-violet-500 disabled:opacity-40 disabled:cursor-not-allowed text-white font-semibold text-xs transition-colors shadow-sm cursor-pointer"
          >
            {t("settings.difficulty.saveScheme")}
          </button>
        </div>
      </div>
    {/if}

    <!-- Custom Schemes List with Trash Bin Icon -->
    <div class="space-y-2.5">
      {#if difficultyStore.customSchemes.length === 0}
        <div class="p-4 rounded-lg bg-white/5 border border-white/10 text-center text-xs text-gray-400">
          {t("settings.difficulty.noCustomSchemes")}
        </div>
      {:else}
        {#each difficultyStore.customSchemes as cs (cs.id)}
          <div class="p-3 rounded-lg bg-white/5 border border-white/10 flex items-center justify-between gap-3 hover:border-violet-500/30 transition-colors">
            <div class="min-w-0 flex-1 space-y-1">
              <div class="flex items-center gap-2 flex-wrap">
                <span class="text-xs font-bold text-violet-300 truncate flex items-center gap-1">
                  <svg class="w-3 h-3 text-violet-400 shrink-0" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
                  </svg>
                  <span>{cs.name}</span>
                </span>
                <span class="text-[10px] px-1.5 py-0.5 rounded bg-violet-500/20 text-violet-200 border border-violet-500/30 font-mono">
                  {cs.tagPrefix || cs.name}::*
                </span>
              </div>
              <div class="flex items-center gap-1.5 text-[11px] text-gray-400 truncate" title={cs.filePath}>
                <svg class="w-3.5 h-3.5 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <span class="font-mono truncate">{cs.filePath}</span>
              </div>
            </div>
            <div class="shrink-0 flex items-center gap-2">
              <button
                type="button"
                onclick={() => removeCustomScheme(cs.id)}
                class="p-2 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-300 hover:text-red-200 border border-red-500/20 text-xs transition-colors cursor-pointer"
                title={t("common.delete") || "Delete"}
                aria-label={t("common.delete") || "Delete"}
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Community Feedback / Contributing Box -->
  <div class="p-4 rounded-xl bg-gradient-to-r from-violet-950/40 via-purple-950/20 to-black/40 border border-violet-500/20 flex flex-col md:flex-row md:items-center justify-between gap-3.5">
    <div class="space-y-1">
      <h4 class="text-xs font-bold text-violet-200 flex items-center gap-1.5">
        <svg class="w-4 h-4 text-violet-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
        </svg>
        <span>{t("settings.difficulty.communityTitle") || "Hai fatto correzioni o vuoi suggerire nuovi schemi?"}</span>
      </h4>
      <p class="text-xs text-gray-400 leading-relaxed max-w-2xl">
        {t("settings.difficulty.communityDesc") || "I database di difficoltà sono open source ed esportabili. Se hai corretto dei vocaboli, ottimizzato i livelli o creato un nuovo database per un'altra lingua, puoi condividerlo con la community su GitHub per integrarlo nelle future versioni di Vesta."}
      </p>
    </div>
    <a
      href="https://github.com/pierspad/vesta"
      target="_blank"
      rel="noopener noreferrer"
      class="px-3.5 py-2 rounded-lg border border-violet-500/30 bg-violet-500/10 hover:bg-violet-500/20 text-violet-200 text-xs font-semibold flex items-center gap-2 transition-all shrink-0 cursor-pointer self-start md:self-center"
    >
      <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
        <path fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.05A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd" />
      </svg>
      <span>{t("settings.difficulty.communityButton") || "Contribuisci su GitHub"}</span>
      <svg class="w-3 h-3 text-violet-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
      </svg>
    </a>
  </div>
</div>
