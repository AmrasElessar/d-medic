<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { ShieldCheck, DatabaseBackup, TrendingUp } from 'lucide-vue-next';
import type { EstimatedGain } from '@/types';
import { computed } from 'vue';

interface Props {
  gain: EstimatedGain;
}
const props = defineProps<Props>();
const { t } = useI18n();

const text = computed(() => {
  switch (props.gain.kind) {
    case 'ram_mb':   return `~${props.gain.value} MB RAM`;
    case 'boot_pct': return `~${props.gain.value}% ${t('gain.boot')}`;
    case 'cpu_pct':  return `~${props.gain.value}% CPU`;
    case 'disk_mb':  return `~${props.gain.value} MB ${t('gain.disk')}`;
    case 'stability':   return t('gain.stability');
    case 'data_safety': return t('gain.data_safety');
    case 'none':     return '—';
  }
});
</script>

<template>
  <span class="inline-flex items-center gap-1 text-xs text-fg-muted">
    <ShieldCheck v-if="gain.kind === 'stability'" class="w-3.5 h-3.5" />
    <DatabaseBackup v-else-if="gain.kind === 'data_safety'" class="w-3.5 h-3.5" />
    <TrendingUp v-else-if="gain.kind !== 'none'" class="w-3.5 h-3.5" />
    {{ text }}
  </span>
</template>
