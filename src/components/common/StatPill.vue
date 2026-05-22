<script setup lang="ts">
import type { Component } from 'vue';

interface Props {
  icon?: Component;
  label: string;
  value: string | number;
  tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger';
}
withDefaults(defineProps<Props>(), { tone: 'neutral' });
</script>

<template>
  <div
    :class="[
      'flex items-center gap-3 px-4 py-3 rounded-lg border',
      tone === 'neutral' && 'border-border bg-bg-subtle',
      tone === 'accent'  && 'border-accent/30 bg-accent/10',
      tone === 'success' && 'border-priority-low/30 bg-priority-low/10',
      tone === 'warning' && 'border-priority-medium/30 bg-priority-medium/10',
      tone === 'danger'  && 'border-priority-critical/30 bg-priority-critical/10',
    ]"
  >
    <component
      :is="icon"
      v-if="icon"
      :class="[
        'w-5 h-5',
        tone === 'neutral' && 'text-fg-muted',
        tone === 'accent'  && 'text-accent',
        tone === 'success' && 'text-priority-low',
        tone === 'warning' && 'text-priority-medium',
        tone === 'danger'  && 'text-priority-critical',
      ]"
    />
    <div>
      <div class="text-xs text-fg-muted">{{ label }}</div>
      <div class="text-base font-semibold text-fg">{{ value }}</div>
    </div>
  </div>
</template>
