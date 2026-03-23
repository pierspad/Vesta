import re

with open("/home/ribben/Desktop/Various_Projects/vesta/apps/srt-gui/src/lib/TranscribeTab.svelte", "r", encoding="utf-8") as f:
    code = f.read()

start_block = '<div\n        class="glass-card p-4 flex flex-col"\n        style="min-height: 170px; max-height: 360px;"\n      >'
end_block = '{:else}\n            <p class="text-gray-600 text-xs p-2">{t("transcribe.noLog")}</p>\n          {/if}\n        </div>\n      </div>'

start_idx = code.find(start_block)
end_idx = code.find(end_block, start_idx)

if start_idx != -1 and end_idx != -1:
    old_html = code[start_idx:end_idx + len(end_block)]
    new_html = '''<LogPanel
        title={t("transcribe.log")}
        clearLogText={t("transcribe.clearLog")}
        noLogText={t("transcribe.noLog")}
        {logs}
        onclear={clearLogs}
        minHeight="170px"
        maxHeightContent="100%"
      />'''
    code = code.replace(old_html, new_html)

with open("/home/ribben/Desktop/Various_Projects/vesta/apps/srt-gui/src/lib/TranscribeTab.svelte", "w", encoding="utf-8") as f:
    f.write(code)

