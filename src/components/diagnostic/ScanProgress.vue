<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useScanStore } from '@/stores/scan';
import ProgressBar from '../common/ProgressBar.vue';
import Spinner from '../common/Spinner.vue';

const scan = useScanStore();
const { t } = useI18n();
</script>

<template>
  <div v-if="scan.status === 'running'" class="space-y-3">
    <div class="flex items-center gap-2 text-sm">
      <Spinner size="sm" />
      <span>{{ t('scan.starting') }}</span>
      <span class="text-fg-muted">— {{ t(`scan.${scan.kind}`) }}</span>
    </div>
    <ProgressBar :value="scan.progress" indeterminate show-label />
  </div>
</template>
