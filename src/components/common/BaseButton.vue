<script setup lang="ts">
import type { Component } from 'vue';

interface Props {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  icon?: Component;
  iconRight?: Component;
  fullWidth?: boolean;
}

withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
  fullWidth: false,
});

defineEmits<{
  (e: 'click', evt: MouseEvent): void;
}>();
</script>

<template>
  <button
    :disabled="disabled || loading"
    :class="[
      'inline-flex items-center justify-center gap-2 font-medium rounded transition-colors',
      'focus:outline-none focus:ring-2 focus:ring-accent/40 focus:ring-offset-2 focus:ring-offset-bg',
      'disabled:opacity-50 disabled:cursor-not-allowed',
      fullWidth && 'w-full',
      size === 'sm' && 'px-2.5 py-1.5 text-sm',
      size === 'md' && 'px-4 py-2 text-sm',
      size === 'lg' && 'px-5 py-3 text-base',
      variant === 'primary' && 'bg-accent text-white hover:bg-accent-hover',
      variant === 'secondary' && 'bg-bg-subtle text-fg hover:bg-bg-elevated border border-border',
      variant === 'ghost' && 'text-fg-muted hover:text-fg hover:bg-bg-subtle',
      variant === 'danger' && 'bg-priority-critical text-white hover:opacity-90',
    ]"
    @click="$emit('click', $event)"
  >
    <span
      v-if="loading"
      class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"
    />
    <component :is="icon" v-else-if="icon" class="w-4 h-4" />
    <slot />
    <component :is="iconRight" v-if="iconRight" class="w-4 h-4" />
  </button>
</template>
