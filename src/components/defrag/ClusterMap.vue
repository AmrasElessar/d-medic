<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { HardDrive } from 'lucide-vue-next';
import { formatBytes } from '@/utils/format';
import type { CellState, ClusterMap } from '@/types';

const props = defineProps<{
  map: ClusterMap | null;
  clusterBytes?: number;
}>();

const { t } = useI18n();

const gridStyle = computed(() => ({
  gridTemplateColumns: `repeat(${props.map?.cols ?? 64}, minmax(0, 1fr))`,
}));

const CELL_CLASS: Record<CellState, string> = {
  free: 'bg-bg-elevated',
  used: 'bg-accent',
  fragmented: 'bg-priority-medium',
  unmovable: 'bg-fg-subtle',
  moving: 'bg-priority-low animate-pulse',
};

const legend: { state: CellState; key: string }[] = [
  { state: 'used', key: 'defrag.legend_used' },
  { state: 'fragmented', key: 'defrag.legend_fragmented' },
  { state: 'free', key: 'defrag.legend_free' },
  { state: 'unmovable', key: 'defrag.legend_unmovable' },
  { state: 'moving', key: 'defrag.legend_moving' },
];

const perCellLabel = computed(() => {
  if (!props.map || !props.clusterBytes) return '';
  return formatBytes(props.map.clusters_per_cell * props.clusterBytes);
});
</script>

<template>
  <div>
    <div
      v-if="map && map.cells.length"
      class="grid gap-px p-2 rounded border border-border bg-border"
      :style="gridStyle"
    >
      <div
        v-for="(cell, i) in map.cells"
        :key="i"
        :class="['aspect-square rounded-[1px]', CELL_CLASS[cell]]"
      />
    </div>
    <div v-else class="grid place-items-center py-12 text-fg-subtle border border-dashed border-border rounded">
      <div class="text-center">
        <HardDrive class="w-8 h-8 mx-auto mb-2" />
        <p class="text-sm">{{ t('defrag.map_empty') }}</p>
      </div>
    </div>

    <!-- Lejant -->
    <div class="flex flex-wrap items-center gap-x-4 gap-y-1.5 mt-3 text-xs text-fg-muted">
      <div v-for="l in legend" :key="l.state" class="flex items-center gap-1.5">
        <span :class="['w-3 h-3 rounded-[2px] inline-block', CELL_CLASS[l.state]]" />
        {{ t(l.key) }}
      </div>
      <span v-if="perCellLabel" class="ml-auto text-fg-subtle">
        {{ t('defrag.per_cell', { size: perCellLabel }) }}
      </span>
    </div>
  </div>
</template>
