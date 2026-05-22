<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { Priority } from '@/types';

interface Props {
  modelValue: Priority[];
  counts?: Record<Priority, number>;
}
defineProps<Props>();
defineEmits<{ (e: 'update:modelValue', value: Priority[]): void }>();

const ALL: Priority[] = ['critical', 'high', 'medium', 'low'];
const { t } = useI18n();

function toggle(p: Priority, current: Priority[], emit: (value: Priority[]) => void) {
  const idx = current.indexOf(p);
  if (idx === -1) emit([...current, p]);
  else emit(current.filter((x) => x !== p));
}
</script>

<template>
  <div class="flex items-center gap-2 flex-wrap">
    <button
      v-for="p in ALL"
      :key="p"
      class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs border transition-colors"
      :class="
        modelValue.includes(p)
          ? 'bg-bg-elevated border-border-strong text-fg'
          : 'bg-bg border-border text-fg-muted hover:text-fg'
      "
      @click="toggle(p, modelValue, (v) => $emit('update:modelValue', v))"
    >
      <span
        class="w-1.5 h-1.5 rounded-full"
        :class="{
          'bg-priority-critical': p === 'critical',
          'bg-priority-high': p === 'high',
          'bg-priority-medium': p === 'medium',
          'bg-priority-low': p === 'low',
        }"
      />
      {{ t(`priority.${p}`) }}
      <span v-if="counts" class="text-fg-subtle">({{ counts[p] ?? 0 }})</span>
    </button>
  </div>
</template>
