<script setup lang="ts">
import { CheckCircle2, XCircle, Circle, Loader2 } from 'lucide-vue-next';
import { useExecutionStore } from '@/stores/execution';
import { useI18n } from 'vue-i18n';
import ProgressBar from '../common/ProgressBar.vue';

const exec = useExecutionStore();
const { t, te } = useI18n();

function actionLabel(id: string): string {
  const key = `action.${id}.title`;
  return te(key) ? t(key) : id;
}
</script>

<template>
  <div v-if="exec.plan" class="space-y-4">
    <ProgressBar
      :value="exec.progress"
      :variant="exec.status === 'failed' ? 'danger' : 'accent'"
      show-label
    />

    <div class="border border-border rounded-lg divide-y divide-border max-h-80 overflow-y-auto">
      <div
        v-for="item in exec.plan.items"
        :key="item.finding_id"
        class="flex items-center gap-3 px-4 py-2.5"
      >
        <CheckCircle2 v-if="item.status === 'success'" class="w-4 h-4 text-priority-low" />
        <XCircle v-else-if="item.status === 'failed'" class="w-4 h-4 text-priority-critical" />
        <Loader2 v-else-if="item.status === 'running'" class="w-4 h-4 text-accent animate-spin" />
        <Circle v-else class="w-4 h-4 text-fg-subtle" />

        <div class="flex-1 min-w-0">
          <div class="text-sm text-fg">{{ actionLabel(item.action_id) }}</div>
          <div v-if="item.message" class="text-xs text-fg-muted truncate">
            {{ item.message }}
          </div>
        </div>
        <span class="text-xs text-fg-subtle">
          {{ t(`exec.status_${item.status}`) }}
        </span>
      </div>
    </div>

    <details v-if="exec.log.length" class="border border-border rounded-lg">
      <summary class="cursor-pointer px-4 py-2 text-sm text-fg-muted hover:text-fg">
        {{ t('exec.show_log') }} ({{ exec.log.length }})
      </summary>
      <pre
        class="px-4 py-3 text-[11px] leading-relaxed font-mono bg-bg-subtle max-h-60 overflow-auto"
      >{{ exec.log.join('\n') }}</pre>
    </details>
  </div>
</template>
