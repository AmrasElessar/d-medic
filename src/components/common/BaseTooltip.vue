<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  content: string;
  side?: 'top' | 'bottom' | 'left' | 'right';
}

withDefaults(defineProps<Props>(), { side: 'top' });
const open = ref(false);
</script>

<template>
  <span
    class="relative inline-flex"
    @mouseenter="open = true"
    @mouseleave="open = false"
    @focusin="open = true"
    @focusout="open = false"
  >
    <slot />
    <span
      v-if="open"
      class="absolute z-30 px-2 py-1 text-xs whitespace-nowrap bg-bg-elevated border border-border rounded shadow-md text-fg pointer-events-none"
      :class="[
        side === 'top'    && 'bottom-full left-1/2 -translate-x-1/2 mb-1',
        side === 'bottom' && 'top-full left-1/2 -translate-x-1/2 mt-1',
        side === 'left'   && 'right-full top-1/2 -translate-y-1/2 mr-1',
        side === 'right'  && 'left-full top-1/2 -translate-y-1/2 ml-1',
      ]"
    >
      {{ content }}
    </span>
  </span>
</template>
