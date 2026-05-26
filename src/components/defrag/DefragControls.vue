<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ScanSearch, Play, Square, AlertTriangle } from 'lucide-vue-next';
import BaseButton from '@/components/common/BaseButton.vue';
import ProgressBar from '@/components/common/ProgressBar.vue';
import type { DefragMode, DefragProgress, VolumeInfo } from '@/types';

defineProps<{
  volume: VolumeInfo | null;
  status: 'idle' | 'analyzing' | 'defragging' | 'done' | 'error';
  progress: DefragProgress | null;
}>();

const emit = defineEmits<{
  (e: 'analyze'): void;
  (e: 'start', mode: DefragMode): void;
  (e: 'cancel'): void;
}>();

const { t } = useI18n();
const mode = ref<DefragMode>('quick');

const MODES: DefragMode[] = ['quick', 'full', 'free_space_consolidate'];
</script>

<template>
  <div class="space-y-3">
    <!-- SSD / desteklenmeyen uyarısı -->
    <div
      v-if="volume && !volume.defrag_supported"
      class="flex items-start gap-2 text-xs text-priority-medium px-2.5 py-2 rounded bg-priority-medium/10 border border-priority-medium/30"
    >
      <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
      <span>
        {{ volume.media_type === 'SSD' ? t('defrag.ssd_warn') : t('defrag.unsupported_warn', { fs: volume.file_system }) }}
      </span>
    </div>

    <!-- Çalışma sırasında ilerleme -->
    <div v-if="status === 'defragging'" class="space-y-2">
      <ProgressBar :value="progress?.percent ?? 0" variant="accent" show-label />
      <div class="text-xs text-fg-muted truncate" :title="progress?.current_file ?? ''">
        {{ progress?.current_file || t('defrag.preparing') }}
      </div>
      <div class="text-xs text-fg-subtle">
        {{ t('defrag.progress_stats', {
          files: progress?.files_processed ?? 0,
          moved: progress?.clusters_moved ?? 0,
        }) }}
      </div>
      <BaseButton variant="secondary" size="sm" :icon="Square" full-width @click="emit('cancel')">
        {{ t('defrag.cancel') }}
      </BaseButton>
    </div>

    <!-- Idle kontroller -->
    <template v-else>
      <BaseButton
        variant="secondary"
        :icon="ScanSearch"
        full-width
        :loading="status === 'analyzing'"
        :disabled="!volume"
        @click="emit('analyze')"
      >
        {{ t('defrag.analyze') }}
      </BaseButton>

      <!-- Analiz ilerlemesi (toplam dosya bilinmediği için belirsiz çubuk) -->
      <div v-if="status === 'analyzing'" class="space-y-1">
        <ProgressBar :value="40" variant="accent" indeterminate />
        <div class="text-xs text-fg-muted">
          {{ t('defrag.analyzing_files', { n: (progress?.files_processed ?? 0).toLocaleString() }) }}
        </div>
        <div v-if="progress?.current_file" class="text-[10px] text-fg-subtle truncate" :title="progress.current_file">
          {{ progress.current_file }}
        </div>
      </div>

      <div>
        <label class="text-xs text-fg-muted block mb-1">{{ t('defrag.mode') }}</label>
        <select
          v-model="mode"
          class="w-full px-3 py-2 text-sm bg-bg-subtle border border-border rounded
                 focus:outline-none focus:border-accent"
        >
          <option v-for="m in MODES" :key="m" :value="m">{{ t(`defrag.mode_${m}`) }}</option>
        </select>
      </div>

      <BaseButton
        variant="primary"
        :icon="Play"
        full-width
        :disabled="!volume || !volume.defrag_supported"
        @click="emit('start', mode)"
      >
        {{ t('defrag.start') }}
      </BaseButton>
    </template>
  </div>
</template>
