<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Undo2, Trash2, ShieldCheck } from 'lucide-vue-next';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseBadge from '@/components/common/BaseBadge.vue';
import { formatBytes } from '@/utils/format';
import type { QuarantineEntry } from '@/types';

defineProps<{ entries: QuarantineEntry[] }>();
const emit = defineEmits<{
  (e: 'restore', id: string): void;
  (e: 'purge', id: string): void;
}>();

const { t } = useI18n();
</script>

<template>
  <div>
    <div v-if="!entries.length" class="text-sm text-fg-muted text-center py-6 flex flex-col items-center gap-2">
      <ShieldCheck class="w-8 h-8 text-fg-subtle" />
      {{ t('uninstall.quarantine_empty') }}
    </div>

    <ul v-else class="space-y-2">
      <li
        v-for="e in entries"
        :key="e.id"
        class="flex items-center gap-3 px-3 py-2.5 rounded border border-border bg-bg-subtle"
      >
        <div class="flex-1 min-w-0">
          <div class="text-sm text-fg truncate">{{ e.program_label }}</div>
          <div class="text-xs text-fg-muted">
            {{ t('uninstall.quarantine_meta', { files: e.file_count, regs: e.reg_export_count }) }}
            · {{ formatBytes(e.total_bytes) }}
          </div>
        </div>
        <BaseBadge :variant="e.expires_in_days <= 2 ? 'warning' : 'neutral'">
          {{ t('uninstall.expires_in', { n: Math.max(0, e.expires_in_days) }) }}
        </BaseBadge>
        <BaseButton size="sm" variant="secondary" :icon="Undo2" @click="emit('restore', e.id)">
          {{ t('uninstall.restore') }}
        </BaseButton>
        <BaseButton size="sm" variant="ghost" :icon="Trash2" @click="emit('purge', e.id)">
          {{ t('uninstall.purge') }}
        </BaseButton>
      </li>
    </ul>
  </div>
</template>
