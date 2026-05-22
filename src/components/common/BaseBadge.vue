<script setup lang="ts">
interface Props {
  variant?: 'neutral' | 'success' | 'warning' | 'danger' | 'info' | 'accent';
  size?: 'sm' | 'md';
}

withDefaults(defineProps<Props>(), {
  variant: 'neutral',
  size: 'sm',
});
</script>

<template>
  <span
    :class="[
      'inline-flex items-center gap-1 rounded-full font-medium',
      size === 'sm' && 'px-2 py-0.5 text-xs',
      size === 'md' && 'px-2.5 py-1 text-sm',
      variant === 'neutral' && 'bg-bg-elevated text-fg-muted border border-border',
      variant === 'success' && 'bg-priority-low/15 text-priority-low border border-priority-low/30',
      variant === 'warning' && 'bg-priority-medium/15 text-priority-medium border border-priority-medium/30',
      variant === 'danger'  && 'bg-priority-critical/15 text-priority-critical border border-priority-critical/30',
      variant === 'info'    && 'bg-accent/15 text-accent border border-accent/30',
      variant === 'accent'  && 'bg-priority-high/15 text-priority-high border border-priority-high/30',
    ]"
  >
    <slot />
  </span>
</template>
