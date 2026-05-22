<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Stethoscope } from 'lucide-vue-next';
import { useNavStore } from '@/stores/nav';
import { NAV_ENTRIES } from '@/constants/nav';

const nav = useNavStore();
const { t } = useI18n();

const mainEntries = computed(() => NAV_ENTRIES.filter((n) => n.group === 'main'));
const systemEntries = computed(() => NAV_ENTRIES.filter((n) => n.group === 'system'));
</script>

<template>
  <aside class="w-56 shrink-0 border-r border-border bg-bg-elevated flex flex-col">
    <div class="px-4 py-4 flex items-center gap-2 border-b border-border">
      <Stethoscope class="w-5 h-5 text-accent" />
      <span class="font-semibold text-fg">D-Medic</span>
    </div>

    <nav class="flex-1 overflow-y-auto p-2 space-y-0.5">
      <div class="text-[10px] uppercase tracking-wider text-fg-subtle px-3 py-2">
        {{ t('nav.section_main') }}
      </div>
      <button
        v-for="entry in mainEntries"
        :key="entry.key"
        class="w-full flex items-center gap-3 px-3 py-2 rounded text-sm transition-colors"
        :class="
          nav.current === entry.key
            ? 'bg-accent/15 text-fg font-medium'
            : 'text-fg-muted hover:text-fg hover:bg-bg-subtle'
        "
        @click="nav.go(entry.key)"
      >
        <component :is="entry.icon" class="w-4 h-4" />
        <span>{{ t(entry.labelKey) }}</span>
      </button>

      <div class="text-[10px] uppercase tracking-wider text-fg-subtle px-3 py-2 mt-3">
        {{ t('nav.section_system') }}
      </div>
      <button
        v-for="entry in systemEntries"
        :key="entry.key"
        class="w-full flex items-center gap-3 px-3 py-2 rounded text-sm transition-colors"
        :class="
          nav.current === entry.key
            ? 'bg-accent/15 text-fg font-medium'
            : 'text-fg-muted hover:text-fg hover:bg-bg-subtle'
        "
        @click="nav.go(entry.key)"
      >
        <component :is="entry.icon" class="w-4 h-4" />
        <span>{{ t(entry.labelKey) }}</span>
      </button>
    </nav>
  </aside>
</template>
