<script lang="ts">
  let { value = $bindable(""), language = "html", onchange = () => {} } = $props();

  let history = $state<string[]>([value || ""]);
  let historyIndex = $state(0);
  let isUndoRedo = false;

  function pushHistory(newValue: string) {
    if (isUndoRedo) {
      isUndoRedo = false;
      return;
    }
    if (history[historyIndex] === newValue) return;
    
    // truncate future history if we are in the middle and making a new edit
    history = history.slice(0, historyIndex + 1);
    history.push(newValue);
    if (history.length > 100) history.shift(); // limit history size
    historyIndex = history.length - 1;
  }

  function undo() {
    if (historyIndex > 0) {
      isUndoRedo = true;
      historyIndex--;
      value = history[historyIndex];
      onchange();
    }
  }

  function redo() {
    if (historyIndex < history.length - 1) {
      isUndoRedo = true;
      historyIndex++;
      value = history[historyIndex];
      onchange();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Check for Ctrl+Z and Ctrl+Y / Ctrl+Shift+Z
    if (e.ctrlKey || e.metaKey) {
      if (e.key === 'z') {
        e.preventDefault();
        if (e.shiftKey) {
          redo();
        } else {
          undo();
        }
      } else if (e.key === 'y') {
        e.preventDefault();
        redo();
      }
    }
    
    // Auto-indent support on Tab (optional, but good for code editors)
    if (e.key === 'Tab') {
      e.preventDefault();
      const target = e.target as HTMLTextAreaElement;
      const start = target.selectionStart;
      const end = target.selectionEnd;
      
      const v = value || "";
      const before = v.substring(0, start);
      const after = v.substring(end);
      
      value = before + "  " + after;
      
      // Update cursor position after Svelte updates DOM
      setTimeout(() => {
        target.selectionStart = target.selectionEnd = start + 2;
        pushHistory(value);
      }, 0);
    }
  }

  function handleInput(e: Event) {
    value = (e.target as HTMLTextAreaElement).value;
    pushHistory(value);
    onchange();
  }

  let lines = $derived((value || "").split("\n"));
  let scrollTop = $state(0);
  let scrollLeft = $state(0);

  // Basic highlight
  function highlight(code: string, lang: string) {
    if (!code) return "";
    let html = code
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
    
    if (lang === "html") {
      html = html.replace(/([a-zA-Z-]+)="([^"]*)"/g, '<span class="text-amber-300">$1</span>="<span class="text-emerald-300">$2</span>"');
      html = html.replace(/(&lt;[\/\w:-]+)/g, '<span class="text-pink-400">$1</span>');
      html = html.replace(/(\/?&gt;)/g, '<span class="text-pink-400">$1</span>');
      html = html.replace(/\{\{(.*?)\}\}/g, '<span class="text-indigo-400">{{$1}}</span>');
    } else if (lang === "css") {
      html = html.replace(/([\.#]?[\w-]+)\s*{/g, '<span class="text-pink-400">$1</span> {');
      html = html.replace(/([\w-]+)\s*:/g, '<span class="text-blue-300">$1</span>:');
      html = html.replace(/:\s*(.*?);/g, ': <span class="text-emerald-300">$1</span>;');
      html = html.replace(/\{\{(.*?)\}\}/g, '<span class="text-indigo-400">{{$1}}</span>');
    }
    // ensure blank line at the end renders spaces correctly if needed
    return html;
  }
</script>

<div class="relative flex w-full h-64 bg-gray-900 rounded-lg overflow-hidden border border-white/10 group focus-within:border-indigo-500/50 transition-colors">
  <!-- Line Numbers -->
  <div class="w-10 bg-black/40 py-3 text-right pr-2 text-[11px] font-mono text-gray-600 select-none overflow-hidden shrink-0">
    <div style="transform: translateY(-{scrollTop}px)">
      {#each lines as _, i}
        <div class="leading-relaxed whitespace-pre">{i + 1}</div>
      {/each}
    </div>
  </div>

  <div class="relative flex-1 overflow-hidden h-full">
    <!-- Highlighted Code -->
    <pre class="absolute w-full p-3 m-0 font-mono text-sm leading-relaxed text-gray-300 whitespace-pre break-normal pointer-events-none" aria-hidden="true" style="transform: translate(-{scrollLeft}px, -{scrollTop}px);">{@html highlight(value, language)}<br/></pre>
    
    <!-- Transparent Textarea -->
    <textarea
      bind:value
      oninput={handleInput}
      onkeydown={handleKeydown}
      onscroll={(e) => {
        scrollTop = e.currentTarget.scrollTop;
        scrollLeft = e.currentTarget.scrollLeft;
      }}
      wrap="off"
      class="absolute inset-0 w-full h-full p-3 m-0 font-mono text-sm leading-relaxed text-transparent bg-transparent border-none resize-none outline-none caret-white whitespace-pre break-normal pr-9 custom-scrollbar"
      spellcheck="false"
    ></textarea>
  </div>
</div>

<style>
  textarea::selection {
    background-color: rgba(99, 102, 241, 0.4);
    color: transparent;
  }
</style>
