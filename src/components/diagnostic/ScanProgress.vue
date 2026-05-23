<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useScanStore } from '@/stores/scan';
import ProgressBar from '../common/ProgressBar.vue';
import Spinner from '../common/Spinner.vue';

const scan = useScanStore();
const { t, te } = useI18n();

/** check_id için i18n label varsa onu, yoksa ham id'yi göster. */
function checkLabel(id: string | null): string {
  if (!id) return '';
  const key = `check.${id}.title`;
  return te(key) ? t(key) : id;
}

const stepText = computed(() => {
  if (scan.totalChecks === 0) return t('scan.starting');
  // Started event'i geldiğinde currentCheckId set, finished'da lastFinished.
  // Kullanıcı en güncel "şu an çalışan" bilgiyi görsün.
  const running = scan.currentCheckId
    ? `${scan.currentIndex + 1} / ${scan.totalChecks} — ${checkLabel(scan.currentCheckId)}`
    : `${scan.totalChecks} / ${scan.totalChecks} — ${t('scan.finalizing')}`;
  return running;
});
</script>

<template>
  <div v-if="scan.status === 'running'" class="space-y-3">
    <div class="flex items-center gap-2 text-sm">
      <Spinner size="sm" />
      <span class="font-medium">{{ stepText }}</span>
      <span class="text-fg-muted">— {{ t(`scan.${scan.kind}`) }}</span>
    </div>
    <ProgressBar
      :value="scan.progress"
      :indeterminate="scan.totalChecks === 0"
      show-label
    />
    <div
      v-if="scan.lastFinishedCheckId"
      class="text-xs text-fg-muted"
    >
      ✓ {{ checkLabel(scan.lastFinishedCheckId) }}
    </div>
  </div>
</template>
