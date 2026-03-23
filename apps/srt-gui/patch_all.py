import os

def patch_file(filepath, log_interface_removal=True, log_style_removal=True, replace_html="", with_new_html="", inject_import=True, extend_translate=False):
    with open(filepath, "r", encoding="utf-8") as f:
        code = f.read()

    if inject_import:
        code = code.replace('import SearchableSelect from "./SearchableSelect.svelte";', 'import SearchableSelect from "./SearchableSelect.svelte";\nimport LogPanel, { type LogEntry } from "./LogPanel.svelte";')

    if log_interface_removal:
        i_start = code.find("  interface LogEntry {")
        if i_start != -1:
            i_end = code.find("  }\n  let logIdCounter", i_start)
            if i_end != -1:
                code = code[:i_start] + code[i_end+4:]

    if log_style_removal:
        i_start = code.find("  function logStyle(type:")
        if i_start != -1:
            i_end = code.find("  }\n\n", i_start)
            if i_end != -1:
                code = code[:i_start] + code[i_end+4:]

    if extend_translate and filepath.endswith("TranslateTab.svelte"):
        code = code.replace("let logs = $state<string[]>([]);", "let logs = $state<LogEntry[]>([]);\n  let logIdCounter = 0;")
        code = code.replace("  function addLog(message: string) {\n    const timestamp = new Date().toLocaleTimeString();\n    logs = [...logs, `[${timestamp}] ${message}`];\n  }", "  function addLog(message: string, type: LogEntry[\"type\"] = \"info\") {\n    const timestamp = new Date().toLocaleTimeString();\n    logs = [...logs, { id: logIdCounter++, timestamp, message, type }];\n  }")

    if replace_html:
        code = code.replace(replace_html, with_new_html)

    with open(filepath, "w", encoding="utf-8") as f:
        f.write(code)

flashcards_html = """      <div class="glass-card p-4 flex flex-col min-h-[180px]">
        <div class="flex items-center justify-between mb-3">
          <h4 class="text-sm font-medium text-gray-400 flex items-center gap-2">
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 12h16m-7 6h7"
              />
            </svg>
            {t("flashcards.logs")}
          </h4>
          {#if logs.length > 0}
            <button
              onclick={clearLogs}
              class="text-xs text-gray-500 hover:text-gray-400 transition-colors"
            >
              {t("flashcards.clearLog")}
            </button>
          {/if}
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto max-h-64 bg-black/20 rounded-lg p-3">
          {#if logs.length > 0}
            <div class="space-y-1.5">
            {#each logs as log (log.id)}
              {@const style = logStyle(log.type)}
              <div
                class="p-2 rounded-lg border {style.bg} {style.border} flex items-start gap-2 animate-fade-in"
              >
                <span class="text-xs flex-shrink-0">{style.icon}</span>
                <div class="flex-1 min-w-0">
                  <p
                    class="text-xs {style.text} leading-tight break-words whitespace-pre-wrap"
                  >
                    {log.message}
                  </p>
                  {#if log.details}
                    <p
                      class="text-[10px] text-gray-500 break-words whitespace-pre-wrap mt-0.5"
                      title={log.details}
                    >
                      {log.details}
                    </p>
                  {/if}
                </div>
                <span class="text-[10px] text-gray-600 flex-shrink-0"
                  >{log.timestamp}</span
                >
              </div>
            {/each}
            </div>
          {:else}
            <p class="text-gray-600 text-xs">{t("flashcards.noLog")}</p>
          {/if}
        </div>
      </div>"""

flashcards_new = """      <LogPanel
        title={t("flashcards.logs")}
        clearLogText={t("flashcards.clearLog")}
        noLogText={t("flashcards.noLog")}
        {logs}
        onclear={clearLogs}
        minHeight="180px"
        maxHeightContent="16rem"
      />"""

transcribe_html = """      <div
        class="glass-card p-4 flex flex-col"
        style="min-height: 170px; max-height: 360px;"
      >
        <div class="flex items-center justify-between mb-3 shrink-0">
          <h4 class="text-sm font-medium text-gray-400 flex items-center gap-2">
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 12h16m-7 6h7"
              />
            </svg>
            {t("transcribe.log")}
          </h4>
          {#if logs.length > 0}
            <button
              onclick={clearLogs}
              class="text-xs text-gray-500 hover:text-gray-400 transition-colors"
              >{t("transcribe.clearLog")}</button
            >
          {/if}
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto bg-black/20 rounded-lg p-3">
          {#if logs.length > 0}
            <div class="space-y-1.5">
            {#each logs as log (log.id)}
              {@const style = logStyle(log.type)}
              <div
                class="p-2 rounded-lg border {style.bg} {style.border} flex items-start gap-2 animate-fade-in"
              >
                <span class="text-xs flex-shrink-0">{style.icon}</span>
                <p
                  class="text-xs {style.text} leading-tight break-words whitespace-pre-wrap flex-1 min-w-0"
                >
                  {log.message}
                </p>
                <span class="text-[10px] text-gray-600 flex-shrink-0"
                  >{log.timestamp}</span
                >
              </div>
            {/each}
            </div>
          {:else}
            <p class="text-gray-600 text-xs">{t("transcribe.noLog")}</p>
          {/if}
        </div>
      </div>"""

transcribe_new = """      <LogPanel
        title={t("transcribe.log")}
        clearLogText={t("transcribe.clearLog")}
        noLogText={t("transcribe.noLog")}
        {logs}
        onclear={clearLogs}
        minHeight="170px"
        maxHeightContent="100%"
      />"""

translate_html = """      <div class="glass-card p-4 shrink-0" style="min-height: 190px;">
        <div class="flex items-center justify-between mb-3">
          <h4 class="text-sm font-medium text-gray-400 flex items-center gap-2">
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 12h16m-7 6h7"
              />
            </svg>
            {t("translate.logs")}
          </h4>
          {#if logs.length > 0}
            <button
              onclick={clearLogs}
              class="text-xs text-gray-500 hover:text-gray-400 transition-colors"
            >
              {t("translate.clearLog")}
            </button>
          {/if}
        </div>
        <div class="max-h-64 overflow-y-auto bg-black/20 rounded-lg p-3">
          {#if logs.length > 0}
            <div class="space-y-1">
              {#each logs as log}
                <p class="text-gray-500 text-xs font-mono">{log}</p>
              {#else}
              {/each}
            </div>
          {:else}
            <p class="text-gray-600 text-xs">{t("translate.noLog")}</p>
          {/if}
        </div>
      </div>"""

translate_html2 = """      <div class="glass-card p-4 shrink-0" style="min-height: 190px;">
        <div class="flex items-center justify-between mb-3">
          <h4 class="text-sm font-medium text-gray-400 flex items-center gap-2">
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 12h16m-7 6h7"
              />
            </svg>
            {t("translate.logs")}
          </h4>
          {#if logs.length > 0}
            <button
              onclick={clearLogs}
              class="text-xs text-gray-500 hover:text-gray-400 transition-colors"
            >
              {t("translate.clearLog")}
            </button>
          {/if}
        </div>
        <div class="max-h-64 overflow-y-auto bg-black/20 rounded-lg p-3">
          {#if logs.length > 0}
            <div class="space-y-1">
              {#each logs as log}
                <p class="text-gray-500 text-xs font-mono">{log}</p>
              {/each}
            </div>
          {:else}
            <p class="text-gray-600 text-xs">{t("translate.noLog")}</p>
          {/if}
        </div>
      </div>"""

translate_new = """      <LogPanel
        title={t("translate.logs")}
        clearLogText={t("translate.clearLog")}
        noLogText={t("translate.noLog")}
        {logs}
        onclear={clearLogs}
        minHeight="190px"
        maxHeightContent="16rem"
      />"""

patch_file("src/lib/FlashcardsTab.svelte", replace_html=flashcards_html, with_new_html=flashcards_new)
patch_file("src/lib/TranscribeTab.svelte", replace_html=transcribe_html, with_new_html=transcribe_new)
patch_file("src/lib/TranslateTab.svelte", replace_html=translate_html, with_new_html=translate_new, log_interface_removal=False, log_style_removal=False, extend_translate=True)
patch_file("src/lib/TranslateTab.svelte", replace_html=translate_html2, with_new_html=translate_new, log_interface_removal=False, log_style_removal=False, extend_translate=False, inject_import=False)
