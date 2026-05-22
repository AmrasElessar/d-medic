<script setup lang="ts" generic="T extends string">
interface Tab {
  key: T;
  label: string;
  badge?: number | string;
}
interface Props {
  modelValue: T;
  tabs: Tab[];
}
defineProps<Props>();
defineEmits<{ (e: 'update:modelValue', value: T): void }>();
</script>

<template>
  <div class="border-b border-border">
    <nav class="flex gap-1 -mb-px">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        :class="[
          'px-4 py-2 text-sm font-medium border-b-2 transition-colors flex items-center gap-2',
          modelValue === tab.key
            ? 'border-accent text-fg'
            : 'border-transparent text-fg-muted hover:text-fg hover:border-border-strong',
        ]"
        @click="$emit('update:modelValue', tab.key)"
      >
        {{ tab.label }}
        <span
          v-if="tab.badge !== undefined"
          class="inline-flex items-center justify-center min-w-[1.25rem] h-5 px-1.5 text-xs rounded-full bg-bg-subtle text-fg-muted"
        >
          {{ tab.badge }}
        </span>
      </button>
    </nav>
  </div>
</template>
