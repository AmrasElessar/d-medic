<script setup lang="ts">
import { CATEGORY_META } from '@/constants/categories';
import type { Category } from '@/types';
import { computed } from 'vue';

interface Props {
  category: Category;
  size?: 'sm' | 'md';
}
const props = withDefaults(defineProps<Props>(), { size: 'md' });
const meta = computed(() => CATEGORY_META[props.category]);
</script>

<template>
  <component
    :is="meta.icon"
    :class="[size === 'sm' ? 'w-3.5 h-3.5' : 'w-4 h-4', 'text-fg-muted']"
  />
</template>
