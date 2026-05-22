<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { Priority } from '@/types';

interface Props {
  priority: Priority;
}
defineProps<Props>();
const { t } = useI18n();
</script>

<template>
  <span
    class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium border"
    :class="{
      'bg-priority-critical/15 text-priority-critical border-priority-critical/30': priority === 'critical',
      'bg-priority-high/15 text-priority-high border-priority-high/30': priority === 'high',
      'bg-priority-medium/15 text-priority-medium border-priority-medium/30': priority === 'medium',
      'bg-priority-low/15 text-priority-low border-priority-low/30': priority === 'low',
    }"
  >
    <span
      class="w-1.5 h-1.5 rounded-full"
      :class="{
        'bg-priority-critical': priority === 'critical',
        'bg-priority-high': priority === 'high',
        'bg-priority-medium': priority === 'medium',
        'bg-priority-low': priority === 'low',
      }"
    />
    {{ t(`priority.${priority}`) }}
  </span>
</template>
