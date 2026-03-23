import re

with open("apps/srt-gui/src/lib/TranslateTab.svelte", "r", encoding="utf-8") as f:
    code = f.read()

# Add import
code = re.sub(r'(import SearchableSelect from "\./SearchableSelect\.svelte";)', r'\1\nimport LogPanel, { type LogEntry } from "./LogPanel.svelte";', code)

# Update let logs
code = re.sub(r'let logs = \$state<string\[\]>\(\[\]\);', r'let logs = $state<LogEntry[]>([]);', code)

# Update addLog function
code = re.sub(
    r'  function addLog\(message: string\) \{\n    const timestamp = new Date\(\)\.toLocaleTimeString\(\);\n    logs = \[\.\.\.logs, `\[\$\{timestamp\}\] \$\{message\}`\];\n  \}',
    r'''  let logIdCounter = 0;
  function addLog(message: string, type: LogEntry["type"] = "info") {
    const timestamp = new Date().toLocaleTimeString();
    logs = [...logs, { id: logIdCounter++, timestamp, message, type }];
  }''',
    code
)

# Replace HTML
html_to_replace = r'''      <div class="glass-card p-4 shrink-0" style="min-height: 190px;">
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
            \{t\("translate\.logs"\)\}
          </h4>
          \{#if logs\.length > 0\}
            <button
              onclick=\{clearLogs\}
              class="text-xs text-gray-500 hover:text-gray-400 transition-colors"
            >
              \{t\("translate\.clearLog"\)\}
            </button>
          \{/if\}
        </div>
        <div class="max-h-64 overflow-y-auto bg-black/20 rounded-lg p-3">
          \{#if logs\.length > 0\}
            <div class="space-y-1">
              \{#each logs as log\}
                <p class="text-gray-500 text-xs font-mono">\{log\}</p>
              \{/each\}
            </div>
          \{:else\}
            <p class="text-gray-600 text-xs">\{t\("translate\.noLog"\)\}</p>
          \{/if\}
        </div>
      </div>'''

new_html = '''      <LogPanel
        title={t("translate.logs") || t("translate.log")}
        clearLogText={t("translate.clearLog")}
        noLogText={t("translate.noLog")}
        {logs}
        onclear={clearLogs}
        minHeight="190px"
        maxHeightContent="16rem"
      />'''

code = re.sub(html_to_replace, new_html, code, flags=re.MULTILINE | re.DOTALL)

with open("apps/srt-gui/src/lib/TranslateTab.svelte", "w", encoding="utf-8") as f:
    f.write(code)

