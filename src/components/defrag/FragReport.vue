<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { FileWarning } from 'lucide-vue-next';
import { formatBytes } from '@/utils/format';
import type { FragmentationReport } from '@/types';

const props = defineProps<{ report: FragmentationReport | null }>();
const { t } = useI18n();

const pctColor = computed(() => {
  const p = props.report?.fragmentation_percent ?? 0;
  if (p >= 10) return 'text-priority-critical';
  if (p >= 3) return 'text-priority-medium';
  return 'text-priority-low';
});
</script>

<template>
  <div v-if="report">
    <div class="flex items-baseline gap-2">
      <span :class="['text-4xl font-bold', pctColor]">{{ report.fragmentation_percent }}%</span>
      <span class="text-sm text-fg-muted">{{ t('defrag.fragmented') }}</span>
    </div>

    <div class="grid grid-cols-2 gap-3 mt-3 text-sm">
      <div>
        <div class="text-fg-subtle text-xs">{{ t('defrag.total_files') }}</div>
        <div class="text-fg font-medium">{{ report.total_files.toLocaleString() }}</div>
      </div>
      <div>
        <div class="text-fg-subtle text-xs">{{ t('defrag.fragmented_files') }}</div>
        <div class="text-fg font-medium">{{ report.fragmented_files.toLocaleString() }}</div>
      </div>
    </div>

    <p class="text-xs text-fg-muted mt-3 px-2 py-1.5 rounded bg-bg-subtle">{{ report.recommendation }}</p>

    <div v-if="report.most_fragmented.length" class="mt-4">
      <h4 class="text-xs font-semibold text-fg-muted mb-1.5 flex items-center gap-1.5">
        <FileWarning class="w-3.5 h-3.5" />
        {{ t('defrag.most_fragmented') }}
      </h4>
      <ul class="space-y-0.5 max-h-48 overflow-y-auto">
        <li
          v-for="f in report.most_fragmented.slice(0, 15)"
          :key="f.path"
          class="flex items-center gap-2 text-xs py-1 border-b border-border/50"
        >
          <span class="flex-1 min-w-0 truncate text-fg-muted" :title="f.path">{{ f.path }}</span>
          <span class="text-fg-subtle shrink-0">{{ formatBytes(f.size_bytes) }}</span>
          <span class="text-priority-medium shrink-0 tabular-nums">
            {{ t('defrag.fragments_n', { n: f.fragments }) }}
          </span>
        </li>
      </ul>
    </div>
  </div>
</template>
