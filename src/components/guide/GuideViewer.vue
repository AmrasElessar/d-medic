<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Clock, AlertTriangle, ExternalLink, X } from 'lucide-vue-next';
import { useGuidesStore } from '@/stores/guides';
import GuideStep from './GuideStep.vue';
import BaseButton from '../common/BaseButton.vue';

const store = useGuidesStore();
const { t, locale } = useI18n();

const guide = computed(() => store.active);
const lang = computed(() => locale.value as 'tr' | 'en');

const completedIds = computed(() =>
  guide.value ? store.completedStepIds[guide.value.id] ?? [] : [],
);
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200"
      enter-from-class="opacity-0"
      leave-active-class="transition duration-150"
      leave-to-class="opacity-0"
    >
      <div
        v-if="guide"
        class="fixed inset-0 z-40 bg-bg/95 backdrop-blur-sm overflow-y-auto"
      >
        <div class="max-w-3xl mx-auto p-6">
          <header class="flex items-start justify-between mb-6">
            <div>
              <h2 class="text-xl font-bold text-fg">{{ guide.title[lang] }}</h2>
              <div class="flex items-center gap-3 text-xs text-fg-muted mt-2">
                <span class="flex items-center gap-1">
                  <Clock class="w-3.5 h-3.5" /> {{ guide.estimated_time }}
                </span>
                <span>{{ guide.priority }}</span>
                <span>Risk: {{ guide.risk }}</span>
              </div>
            </div>
            <button
              class="p-2 text-fg-muted hover:text-fg"
              @click="store.close()"
            >
              <X class="w-5 h-5" />
            </button>
          </header>

          <div
            v-if="guide.risk_note"
            class="flex items-start gap-2 p-3 mb-4 bg-priority-medium/10 border border-priority-medium/30 rounded"
          >
            <AlertTriangle class="w-4 h-4 text-priority-medium shrink-0 mt-0.5" />
            <p class="text-sm text-fg">{{ guide.risk_note[lang] }}</p>
          </div>

          <div v-if="guide.prerequisites.length" class="mb-4">
            <h3 class="text-xs uppercase tracking-wider text-fg-subtle mb-2">
              {{ t('guide.prerequisites') }}
            </h3>
            <ul class="text-sm text-fg-muted list-disc list-inside space-y-1">
              <li v-for="p in guide.prerequisites" :key="p">{{ p }}</li>
            </ul>
          </div>

          <h3 class="text-xs uppercase tracking-wider text-fg-subtle mb-2">
            {{ t('guide.steps') }}
          </h3>
          <div class="space-y-3">
            <GuideStep
              v-for="step in guide.steps"
              :key="step.id"
              :step="step"
              :done="completedIds.includes(step.id)"
              @toggle="(done) => store.markStep(guide!.id, step.id, done)"
            />
          </div>

          <footer class="mt-6 flex items-center justify-between">
            <a
              v-if="guide.microsoft_doc"
              :href="guide.microsoft_doc"
              target="_blank"
              rel="noopener"
              class="inline-flex items-center gap-1.5 text-sm text-accent hover:underline"
            >
              <ExternalLink class="w-3.5 h-3.5" />
              {{ t('guide.official_doc') }}
            </a>
            <BaseButton variant="secondary" @click="store.close()">
              {{ t('common.close') }}
            </BaseButton>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
