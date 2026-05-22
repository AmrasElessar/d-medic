<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Finding } from '@/types';
import PriorityBadge from '../diagnostic/PriorityBadge.vue';
import GainBadge from '../diagnostic/GainBadge.vue';

interface Props {
  finding: Finding;
  selected: boolean;
}
const props = defineProps<Props>();
defineEmits<{ (e: 'toggle', id: string): void }>();
const { locale } = useI18n();
const title = computed(() => props.finding.title[locale.value as 'tr' | 'en']);
</script>

<template>
  <label
    class="flex items-center gap-3 px-4 py-2.5 cursor-pointer transition-colors"
    :class="selected ? 'bg-accent/10' : 'hover:bg-bg-subtle'"
  >
    <input
      type="checkbox"
      :checked="selected"
      class="accent-accent"
      @change="$emit('toggle', finding.id)"
    />
    <div class="flex-1 min-w-0">
      <div class="text-sm text-fg truncate">{{ title }}</div>
    </div>
    <GainBadge :gain="finding.estimated_gain" />
    <PriorityBadge :priority="finding.priority" />
  </label>
</template>
