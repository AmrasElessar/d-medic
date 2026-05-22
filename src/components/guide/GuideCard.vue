<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { BookOpen, ExternalLink } from 'lucide-vue-next';
import type { Guide } from '@/types';

interface Props {
  guide: Guide;
}
const props = defineProps<Props>();
defineEmits<{ (e: 'open', id: string): void }>();
const { locale } = useI18n();
const title = computed(() => props.guide.title[locale.value as 'tr' | 'en']);
</script>

<template>
  <button
    class="text-left p-4 border border-border rounded-lg bg-bg-subtle hover:border-border-strong transition-colors w-full"
    @click="$emit('open', guide.id)"
  >
    <div class="flex items-start justify-between mb-2">
      <div class="flex items-center gap-2">
        <BookOpen class="w-4 h-4 text-accent" />
        <h4 class="text-sm font-semibold text-fg">{{ title }}</h4>
      </div>
      <ExternalLink class="w-3.5 h-3.5 text-fg-subtle" />
    </div>
    <div class="flex items-center gap-3 text-xs text-fg-muted">
      <span>⏱ {{ guide.estimated_time }}</span>
      <span>· {{ guide.priority }}</span>
      <span>· Risk: {{ guide.risk }}</span>
    </div>
  </button>
</template>
