<script setup lang="ts" generic="T extends string | number">
interface Option {
  value: T;
  label: string;
}

interface Props {
  modelValue: T;
  options: Option[];
  label?: string;
  disabled?: boolean;
}

defineProps<Props>();
defineEmits<{
  (e: 'update:modelValue', value: T): void;
}>();
</script>

<template>
  <label class="block">
    <span v-if="label" class="block text-sm font-medium text-fg mb-1">{{ label }}</span>
    <select
      :value="modelValue"
      :disabled="disabled"
      class="w-full px-3 py-2 text-sm bg-bg-subtle border border-border rounded
             focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent
             disabled:opacity-50"
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value as T)"
    >
      <option v-for="opt in options" :key="String(opt.value)" :value="opt.value">
        {{ opt.label }}
      </option>
    </select>
  </label>
</template>
