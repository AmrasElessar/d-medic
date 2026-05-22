<script setup lang="ts">
interface Props {
  size?: 'sm' | 'md' | 'lg';
}
withDefaults(defineProps<Props>(), { size: 'md' });
</script>

<template>
  <span
    :class="[
      'inline-block border-2 border-current border-t-transparent rounded-full animate-spin',
      size === 'sm' && 'w-3 h-3',
      size === 'md' && 'w-4 h-4',
      size === 'lg' && 'w-6 h-6',
    ]"
  />
</template>
