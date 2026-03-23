import re

with open("apps/srt-gui/src/lib/FlashcardsTab.svelte", "r", encoding="utf-8") as f:
    code = f.read()

# Add import
code = re.sub(r'(import SearchableSelect from "\./SearchableSelect\.svelte";)', r'\1\nimport LogPanel, { type LogEntry } from "./LogPanel.svelte";', code)

# Remove LogEntry interface
code = re.sub(r'  interface LogEntry \{.*?    \}\n', '', code, flags=re.DOTALL)

# Remove logStyle function
code = re.sub(r'  function logStyle\(type: LogEntry\["type"\]\).*?^\s+\}\n\s+\}\n', '', code, flags=re.MULTILINE | re.DOTALL)

# Replace the HTML
html_to_replace = r'''      <div class="glass-card p-4 flex flex-col min-h-\[180px\]">
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
            \{t\("flashcards\.logs"\)\}
          </h4>
          \{#if logs\.length > 0\}
            <button
              onclick=\{clearLogs\}
              class="text-xs text-gray-500 hover:text-gray-400 transition-colors"
            >
              \{t\("flashcards\.clearLog"\)\}
            </button>
          \{/if\}
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto max-h-64 bg-black/20 rounded-lg p-3">
          \{#if logs\.length > 0\}
            <div class="space-y-1\.5">
            \{#each logs as log \(log\.id\)\}
              \{@const style = logStyle\(log\.type\)\}
              <div
                class="p-2 rounded-lg border \{style\.bg\} \{style\.border\} flex items-start gap-2 animate-fade-in"
              >
                <span class="text-xs flex-shrink-0">\{style\.icon\}</span>
                <div class="flex-1 min-w-0">
                  <p
                    class="text-xs \{style\.text\} leading-tight break-words whitespace-pre-wrap"
                  >
                    \{log\.message\}
                  </p>
                  \{#if log\.details\}
                    <p
                      class="text-\[10px\] text-gray-500 break-words whitespace-pre-wrap mt-0\.5"
                      title=\{log\.details\}
                    >
                      \{log\.details\}
                    </p>
                  \{/if\}
                </div>
                <span class="text-\[10px\] text-gray-600 flex-shrink-0"
                  >\{log\.timestamp\}</span
                >
              </div>
            \{/each\}
            </div>
          \{:else\}
            <p class="text-gray-600 text-xs">\{t\("flashcards\.noLog"\)\}</p>
          \{/if\}
        </div>
      </div>'''

new_html = '''      <LogPanel
        title={t("flashcards.logs")}
        clearLogText={t("flashcards.clearLog")}
        noLogText={t("flashcards.noLog")}
        {logs}
        onclear={clearLogs}
        minHeight="180px"
        maxHeightContent="16rem"
      />'''

code = re.sub(html_to_replace, new_html, code, flags=re.MULTILINE | re.DOTALL)

with open("apps/srt-gui/src/lib/FlashcardsTab.svelte", "w", encoding="utf-8") as f:
    f.write(code)

