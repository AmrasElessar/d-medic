<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Search, Package, AppWindow } from 'lucide-vue-next';
import BaseBadge from '@/components/common/BaseBadge.vue';
import { formatBytes, formatInstallDate } from '@/utils/format';
import type { InstalledProgram } from '@/types';

const props = defineProps<{
  programs: InstalledProgram[];
  selectedId: string | null;
}>();

const emit = defineEmits<{ (e: 'select', program: InstalledProgram): void }>();

const { t } = useI18n();
const query = ref('');

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  const base = q
    ? props.programs.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          (p.publisher ?? '').toLowerCase().includes(q),
      )
    : props.programs;
  return base;
});
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="relative mb-2">
      <Search class="w-4 h-4 absolute left-2.5 top-1/2 -translate-y-1/2 text-fg-subtle" />
      <input
        v-model="query"
        :placeholder="t('uninstall.search')"
        class="w-full pl-8 pr-3 py-2 text-sm bg-bg-subtle border border-border rounded
               focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent"
      />
    </div>

    <div class="text-xs text-fg-subtle px-1 pb-1">
      {{ t('uninstall.program_count', { n: filtered.length }) }}
    </div>

    <div class="flex-1 overflow-y-auto divide-y divide-border rounded border border-border">
      <button
        v-for="p in filtered"
        :key="p.id"
        class="w-full text-left px-3 py-2 flex items-center gap-3 transition-colors"
        :class="
          selectedId === p.id ? 'bg-accent/15' : 'hover:bg-bg-subtle'
        "
        @click="emit('select', p)"
      >
        <component
          :is="p.kind === 'uwp' ? AppWindow : Package"
          class="w-4 h-4 shrink-0 text-fg-muted"
        />
        <div class="flex-1 min-w-0">
          <div class="text-sm text-fg truncate">{{ p.name }}</div>
          <div class="text-xs text-fg-muted truncate">
            {{ p.publisher || t('uninstall.unknown_publisher') }}
            <span v-if="p.version"> · {{ p.version }}</span>
          </div>
        </div>
        <div class="text-right shrink-0">
          <div class="text-xs text-fg-muted">{{ formatBytes(p.size_bytes) }}</div>
          <div class="text-[10px] text-fg-subtle">{{ formatInstallDate(p.install_date) }}</div>
        </div>
        <BaseBadge v-if="p.kind === 'uwp'" variant="info">UWP</BaseBadge>
      </button>

      <div v-if="!filtered.length" class="px-3 py-8 text-center text-sm text-fg-muted">
        {{ t('uninstall.no_match') }}
      </div>
    </div>
  </div>
</template>
