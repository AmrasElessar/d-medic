<script setup lang="ts">
interface Props {
  value: number;
  max?: number;
  variant?: 'accent' | 'success' | 'warning' | 'danger';
  showLabel?: boolean;
  indeterminate?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  max: 100,
  variant: 'accent',
  showLabel: false,
  indeterminate: false,
});

const pct = () => Math.max(0, Math.min(100, (props.value / props.max) * 100));
</script>

<template>
  <div class="w-full">
    <div class="flex items-center justify-between mb-1" v-if="showLabel">
      <span class="text-xs text-fg-muted">{{ Math.round(pct()) }}%</span>
    </div>
    <div class="w-full h-2 rounded-full bg-bg-subtle overflow-hidden">
      <div
        :class="[
          'h-full transition-all duration-300',
          variant === 'accent'  && 'bg-accent',
          variant === 'success' && 'bg-priority-low',
          variant === 'warning' && 'bg-priority-medium',
          variant === 'danger'  && 'bg-priority-critical',
          indeterminate && 'animate-pulse',
        ]"
        :style="{ width: indeterminate ? '40%' : pct() + '%' }"
      />
    </div>
  </div>
</template>
