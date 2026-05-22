<script setup lang="ts">
import type { Component } from 'vue';

interface Props {
  icon?: Component;
  title: string;
  description?: string;
}
defineProps<Props>();
</script>

<template>
  <div class="grid place-items-center text-center py-12 px-6">
    <component
      :is="icon"
      v-if="icon"
      class="w-10 h-10 text-fg-subtle mb-3"
    />
    <h3 class="text-base font-semibold text-fg">{{ title }}</h3>
    <p v-if="description" class="text-sm text-fg-muted mt-1 max-w-md">{{ description }}</p>
    <div v-if="$slots.actions" class="mt-4">
      <slot name="actions" />
    </div>
  </div>
</template>
