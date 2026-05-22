<script setup lang="ts">
interface Props {
  title?: string;
  subtitle?: string;
  padding?: 'none' | 'sm' | 'md' | 'lg';
  variant?: 'default' | 'elevated' | 'outlined';
}

withDefaults(defineProps<Props>(), {
  padding: 'md',
  variant: 'default',
});
</script>

<template>
  <section
    :class="[
      'rounded-lg',
      variant === 'default' && 'bg-bg-subtle border border-border',
      variant === 'elevated' && 'bg-bg-elevated border border-border shadow-sm',
      variant === 'outlined' && 'border border-border-strong',
    ]"
  >
    <header
      v-if="title || $slots.header"
      class="flex items-center justify-between px-4 py-3 border-b border-border"
    >
      <div>
        <h3 v-if="title" class="text-sm font-semibold text-fg">{{ title }}</h3>
        <p v-if="subtitle" class="text-xs text-fg-muted mt-0.5">{{ subtitle }}</p>
      </div>
      <slot name="header" />
    </header>
    <div
      :class="[
        padding === 'none' && 'p-0',
        padding === 'sm' && 'p-3',
        padding === 'md' && 'p-4',
        padding === 'lg' && 'p-6',
      ]"
    >
      <slot />
    </div>
    <footer
      v-if="$slots.footer"
      class="px-4 py-3 border-t border-border bg-bg/50"
    >
      <slot name="footer" />
    </footer>
  </section>
</template>
