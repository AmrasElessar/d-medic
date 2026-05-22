<script setup lang="ts">
interface Props {
  modelValue: boolean;
  label?: string;
  hint?: string;
  disabled?: boolean;
}

defineProps<Props>();
defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();
</script>

<template>
  <label
    class="flex items-start gap-3 cursor-pointer select-none"
    :class="disabled && 'opacity-50 cursor-not-allowed'"
  >
    <button
      type="button"
      role="switch"
      :aria-checked="modelValue"
      :disabled="disabled"
      class="relative inline-flex h-5 w-9 shrink-0 mt-0.5 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-accent/40"
      :class="modelValue ? 'bg-accent' : 'bg-border-strong'"
      @click="$emit('update:modelValue', !modelValue)"
    >
      <span
        class="inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform"
        :class="modelValue ? 'translate-x-5' : 'translate-x-1'"
      />
    </button>
    <div v-if="label || hint" class="flex-1">
      <span v-if="label" class="block text-sm text-fg">{{ label }}</span>
      <span v-if="hint" class="block text-xs text-fg-muted mt-0.5">{{ hint }}</span>
    </div>
  </label>
</template>
