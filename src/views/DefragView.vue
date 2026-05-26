<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { HardDrive, Cpu } from 'lucide-vue-next';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseBadge from '@/components/common/BaseBadge.vue';
import ClusterMap from '@/components/defrag/ClusterMap.vue';
import FragReport from '@/components/defrag/FragReport.vue';
import DefragControls from '@/components/defrag/DefragControls.vue';
import { useDefragStore } from '@/stores/defrag';
import { useDefrag } from '@/composables/useDefrag';
import { formatBytes } from '@/utils/format';
import type { DefragMode } from '@/types';

const { t } = useI18n();
const store = useDefragStore();
const { loadVolumes, analyze, startDefrag, cancel } = useDefrag();

onMounted(loadVolumes);

const vol = computed(() => store.selectedVolume);

function usedPct(): number {
  const v = vol.value;
  if (!v || !v.total_bytes) return 0;
  return Math.round(((v.total_bytes - v.free_bytes) / v.total_bytes) * 100);
}

function onAnalyze(): void {
  if (store.selected) void analyze(store.selected);
}
function onStart(mode: DefragMode): void {
  if (store.selected) void startDefrag(store.selected, mode);
}
</script>

<template>
  <div class="p-6 space-y-4 max-w-6xl mx-auto">
    <header>
      <h1 class="text-2xl font-bold text-fg">{{ t('view.defrag.title') }}</h1>
      <p class="text-sm text-fg-muted mt-1">{{ t('view.defrag.subtitle') }}</p>
    </header>

    <!-- Birim seçici -->
    <div class="flex flex-wrap gap-2">
      <button
        v-for="v in store.volumes"
        :key="v.letter"
        class="flex items-center gap-2.5 px-3 py-2 rounded-lg border transition-colors"
        :class="
          store.selected === v.letter
            ? 'border-accent bg-accent/10 text-fg'
            : 'border-border bg-bg-subtle text-fg-muted hover:text-fg'
        "
        :disabled="store.status === 'defragging'"
        @click="store.select(v.letter)"
      >
        <HardDrive class="w-4 h-4" />
        <div class="text-left">
          <div class="text-sm font-medium">{{ v.letter }}:</div>
          <div class="text-[10px] text-fg-subtle">{{ formatBytes(v.free_bytes) }} {{ t('defrag.free') }}</div>
        </div>
        <BaseBadge :variant="v.media_type === 'SSD' ? 'info' : 'neutral'">{{ v.media_type }}</BaseBadge>
      </button>
      <div v-if="!store.volumes.length" class="text-sm text-fg-muted py-2">
        {{ t('defrag.no_volumes') }}
      </div>
    </div>

    <div v-if="vol" class="grid grid-cols-1 lg:grid-cols-3 gap-4">
      <!-- Sol kolon: rapor + kontroller -->
      <div class="space-y-4 lg:col-span-1">
        <BaseCard :title="t('defrag.volume_title', { letter: vol.letter })">
          <div class="space-y-1.5 text-sm">
            <div class="flex justify-between">
              <span class="text-fg-muted">{{ t('defrag.filesystem') }}</span>
              <span class="text-fg">{{ vol.file_system }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-fg-muted flex items-center gap-1"><Cpu class="w-3.5 h-3.5" />{{ t('defrag.media') }}</span>
              <span class="text-fg">{{ vol.media_type }}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-fg-muted">{{ t('defrag.usage') }}</span>
              <span class="text-fg">{{ usedPct() }}% ({{ formatBytes(vol.total_bytes - vol.free_bytes) }} / {{ formatBytes(vol.total_bytes) }})</span>
            </div>
          </div>
        </BaseCard>

        <BaseCard :title="t('defrag.controls_title')">
          <DefragControls
            :volume="vol"
            :status="store.status"
            :progress="store.progress"
            @analyze="onAnalyze"
            @start="onStart"
            @cancel="cancel"
          />
        </BaseCard>

        <BaseCard v-if="store.report" :title="t('defrag.report_title')">
          <FragReport :report="store.report" />
        </BaseCard>
      </div>

      <!-- Sağ kolon: cluster haritası -->
      <div class="lg:col-span-2">
        <BaseCard :title="t('defrag.map_title')" :subtitle="t('defrag.map_subtitle')">
          <ClusterMap :map="store.map" :cluster-bytes="vol.cluster_bytes" />
        </BaseCard>
      </div>
    </div>
  </div>
</template>
