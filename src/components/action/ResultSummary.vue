<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { CheckCircle2, XCircle, MinusCircle } from 'lucide-vue-next';
import { useExecutionStore } from '@/stores/execution';
import StatPill from '../common/StatPill.vue';

const exec = useExecutionStore();
const { t } = useI18n();

const stats = computed(() => {
  const items = exec.plan?.items ?? [];
  return {
    success: items.filter((i) => i.status === 'success').length,
    failed: items.filter((i) => i.status === 'failed').length,
    skipped: items.filter((i) => i.status === 'skipped').length,
  };
});
</script>

<template>
  <div class="grid grid-cols-3 gap-3">
    <StatPill
      :icon="CheckCircle2"
      :label="t('exec.success')"
      :value="stats.success"
      tone="success"
    />
    <StatPill
      :icon="XCircle"
      :label="t('exec.failed')"
      :value="stats.failed"
      tone="danger"
    />
    <StatPill
      :icon="MinusCircle"
      :label="t('exec.skipped')"
      :value="stats.skipped"
      tone="neutral"
    />
  </div>
</template>
