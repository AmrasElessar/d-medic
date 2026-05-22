<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Terminal, Cpu, Info, ExternalLink, MousePointerClick } from 'lucide-vue-next';
import type { GuideStep } from '@/types';

interface Props {
  step: GuideStep;
  done: boolean;
}
defineProps<Props>();
defineEmits<{ (e: 'toggle', done: boolean): void }>();

const { locale } = useI18n();
const lang = computed(() => locale.value as 'tr' | 'en');
</script>

<template>
  <div
    class="flex items-start gap-3 p-4 border border-border rounded-lg"
    :class="done ? 'bg-priority-low/5 border-priority-low/30' : 'bg-bg-subtle'"
  >
    <input
      type="checkbox"
      :checked="done"
      class="mt-1 accent-priority-low"
      @change="$emit('toggle', !done)"
    />
    <div class="flex-1">
      <div class="flex items-center gap-2 mb-1">
        <Terminal v-if="step.type === 'cmd'" class="w-4 h-4 text-accent" />
        <Cpu v-if="step.type === 'bios'" class="w-4 h-4 text-priority-medium" />
        <Info v-if="step.type === 'info'" class="w-4 h-4 text-fg-muted" />
        <MousePointerClick v-if="step.type === 'manual'" class="w-4 h-4 text-priority-high" />
        <ExternalLink v-if="step.type === 'link'" class="w-4 h-4 text-accent" />
        <h5 class="text-sm font-medium text-fg">
          {{ step.id }}. {{ step.title[lang] }}
        </h5>
      </div>
      <p v-if="step.body" class="text-xs text-fg-muted mb-2">
        {{ step.body[lang] }}
      </p>
      <pre
        v-if="step.command"
        class="text-xs font-mono bg-bg-elevated border border-border rounded p-2 overflow-x-auto"
      >{{ step.command }}</pre>
      <a
        v-if="step.guide_link"
        :href="step.guide_link"
        target="_blank"
        rel="noopener"
        class="inline-flex items-center gap-1 text-xs text-accent hover:underline mt-2"
      >
        <ExternalLink class="w-3 h-3" /> {{ step.guide_link }}
      </a>
    </div>
  </div>
</template>
