<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useScanStore } from '@/stores/scan';
import { useSystemStore } from '@/stores/system';
import Spinner from '../common/Spinner.vue';

const { t } = useI18n();
const scan = useScanStore();
const sys = useSystemStore();
</script>

<template>
  <footer
    class="h-7 shrink-0 border-t border-border bg-bg-elevated text-xs text-fg-muted flex items-center justify-between px-3"
  >
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1.5">
        <Spinner v-if="scan.status === 'running'" size="sm" />
        <span
          v-else
          class="w-2 h-2 rounded-full"
          :class="{
            'bg-priority-low': scan.status === 'completed',
            'bg-priority-critical': scan.status === 'failed',
            'bg-fg-subtle': scan.status === 'idle',
          }"
        />
        <span>{{ t(`status.${scan.status}`) }}</span>
      </span>
      <span v-if="scan.lastResult">
        {{ t('status.last_findings', { n: scan.lastResult.findings.length }) }}
      </span>
    </div>

    <div class="flex items-center gap-3">
      <span>{{ sys.info?.os ?? '—' }}</span>
      <span>v{{ sys.version }}</span>
    </div>
  </footer>
</template>
