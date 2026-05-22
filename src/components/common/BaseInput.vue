<script setup lang="ts">
interface Props {
  modelValue: string | number;
  type?: 'text' | 'number' | 'password' | 'email';
  placeholder?: string;
  label?: string;
  hint?: string;
  error?: string;
  disabled?: boolean;
}

defineProps<Props>();
defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();
</script>

<template>
  <label class="block">
    <span v-if="label" class="block text-sm font-medium text-fg mb-1">{{ label }}</span>
    <input
      :value="modelValue"
      :type="type ?? 'text'"
      :placeholder="placeholder"
      :disabled="disabled"
      class="w-full px-3 py-2 text-sm bg-bg-subtle border border-border rounded
             focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent
             disabled:opacity-50"
      :class="error && 'border-priority-critical focus:border-priority-critical focus:ring-priority-critical'"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <span v-if="error" class="block text-xs text-priority-critical mt-1">{{ error }}</span>
    <span v-else-if="hint" class="block text-xs text-fg-muted mt-1">{{ hint }}</span>
  </label>
</template>
