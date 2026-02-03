<script lang="ts">
  import { locale } from "./i18n";

  // Reactive translation
  let t = $derived($locale);
</script>

<div class="h-full flex flex-col p-6 overflow-auto bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950">
  <!-- Header -->
  <div class="mb-6">
    <div class="flex items-center gap-3">
      <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center shadow-lg">
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
        </svg>
      </div>
      <div>
        <h2 class="text-3xl font-bold bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent">
          {t("nav.transcribe")}
        </h2>
        <p class="text-gray-400 mt-1">
          {t("nav.transcribe.desc")}
        </p>
      </div>
    </div>
  </div>

  <!-- Coming Soon Card -->
  <div class="flex-1 flex items-center justify-center">
    <div class="glass-card p-12 text-center max-w-lg">
      <!-- Animated Icon -->
      <div class="relative mb-8">
        <div class="w-24 h-24 mx-auto rounded-2xl bg-gradient-to-br from-cyan-500/20 to-blue-600/20 flex items-center justify-center">
          <svg class="w-12 h-12 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
          </svg>
        </div>
        <!-- Pulse ring -->
        <div class="absolute inset-0 flex items-center justify-center">
          <div class="w-32 h-32 rounded-full border-2 border-cyan-500/30 animate-ping"></div>
        </div>
      </div>

      <h3 class="text-2xl font-bold text-white mb-4">{t("comingSoon.title")}</h3>
      <p class="text-gray-400 mb-6 leading-relaxed">
        {t("transcribe.comingSoonDesc")}
      </p>

      <!-- Features Preview -->
      <div class="space-y-3 text-left mb-8">
        <div class="flex items-center gap-3 p-3 bg-white/5 rounded-lg">
          <svg class="w-5 h-5 text-cyan-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-gray-300 text-sm">{t("transcribe.feature1")}</span>
        </div>
        <div class="flex items-center gap-3 p-3 bg-white/5 rounded-lg">
          <svg class="w-5 h-5 text-cyan-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-gray-300 text-sm">{t("transcribe.feature2")}</span>
        </div>
        <div class="flex items-center gap-3 p-3 bg-white/5 rounded-lg">
          <svg class="w-5 h-5 text-cyan-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-gray-300 text-sm">{t("transcribe.feature3")}</span>
        </div>
      </div>

      <!-- Badge -->
      <div class="inline-flex items-center gap-2 px-4 py-2 bg-amber-500/10 border border-amber-500/30 rounded-full">
        <svg class="w-4 h-4 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span class="text-amber-400 text-sm font-medium">{t("comingSoon.badge")}</span>
      </div>
    </div>
  </div>
</div>
