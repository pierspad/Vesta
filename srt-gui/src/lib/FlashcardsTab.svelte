<script lang="ts">
  import { locale } from "./i18n";

  // Reactive translation
  let t = $derived($locale);
</script>

<div class="h-full flex flex-col p-6 overflow-auto bg-gradient-to-br from-gray-900 via-gray-900 to-gray-950">
  <!-- Header -->
  <div class="mb-6">
    <div class="flex items-center gap-3">
      <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-emerald-500 to-teal-600 flex items-center justify-center shadow-lg">
        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
      </div>
      <div>
        <h2 class="text-3xl font-bold bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent">
          {t("nav.flashcards")}
        </h2>
        <p class="text-gray-400 mt-1">
          {t("nav.flashcards.desc")}
        </p>
      </div>
    </div>
  </div>

  <!-- Coming Soon Card -->
  <div class="flex-1 flex items-center justify-center">
    <div class="glass-card p-12 text-center max-w-lg">
      <!-- Animated Icon -->
      <div class="relative mb-8">
        <div class="w-24 h-24 mx-auto rounded-2xl bg-gradient-to-br from-emerald-500/20 to-teal-600/20 flex items-center justify-center">
          <svg class="w-12 h-12 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
        </div>
        <!-- Pulse ring -->
        <div class="absolute inset-0 flex items-center justify-center">
          <div class="w-32 h-32 rounded-full border-2 border-emerald-500/30 animate-ping"></div>
        </div>
      </div>

      <h3 class="text-2xl font-bold text-white mb-4">{t("comingSoon.title")}</h3>
      <p class="text-gray-400 mb-6 leading-relaxed">
        {t("flashcards.comingSoonDesc")}
      </p>

      <!-- Features Preview -->
      <div class="space-y-3 text-left mb-8">
        <div class="flex items-center gap-3 p-3 bg-white/5 rounded-lg">
          <svg class="w-5 h-5 text-emerald-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-gray-300 text-sm">{t("flashcards.feature1")}</span>
        </div>
        <div class="flex items-center gap-3 p-3 bg-white/5 rounded-lg">
          <svg class="w-5 h-5 text-emerald-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-gray-300 text-sm">{t("flashcards.feature2")}</span>
        </div>
        <div class="flex items-center gap-3 p-3 bg-white/5 rounded-lg">
          <svg class="w-5 h-5 text-emerald-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-gray-300 text-sm">{t("flashcards.feature3")}</span>
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
