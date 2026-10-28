<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { computed } from 'vue';
import { Undo2, Trash2, ShieldCheck, AlertTriangle } from 'lucide-vue-next';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseBadge from '@/components/common/BaseBadge.vue';
import { formatBytes } from '@/utils/format';
import type { QuarantineEntry } from '@/types';

const props = defineProps<{ entries: QuarantineEntry[] }>();
const emit = defineEmits<{
  (e: 'restore', id: string): void;
  (e: 'purge', id: string): void;
  (e: 'purge-all'): void;
}>();

const { t } = useI18n();

/** Bu boyutun üzerinde karantina diskte belirgin yer kaplıyor — kullanıcıyı uyar. */
const LARGE_QUARANTINE_BYTES = 5 * 1024 ** 3;

const totalBytes = computed(() => props.entries.reduce((sum, e) => sum + e.total_bytes, 0));
</script>

<template>
  <div>
    <div v-if="!entries.length" class="text-sm text-fg-muted text-center py-6 flex flex-col items-center gap-2">
      <ShieldCheck class="w-8 h-8 text-fg-subtle" />
      {{ t('uninstall.quarantine_empty') }}
    </div>

    <template v-else>
    <div class="flex items-center justify-between gap-3 mb-3">
      <span class="text-xs text-fg-muted">
        {{ t('uninstall.quarantine_total', { n: entries.length, size: formatBytes(totalBytes) }) }}
      </span>
      <BaseButton size="sm" variant="ghost" :icon="Trash2" @click="emit('purge-all')">
        {{ t('uninstall.purge_all') }}
      </BaseButton>
    </div>
    <div
      v-if="totalBytes > LARGE_QUARANTINE_BYTES"
      class="flex items-start gap-2 text-xs text-priority-medium px-2.5 py-2 mb-3 rounded bg-priority-medium/10 border border-priority-medium/30"
    >
      <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
      <span>{{ t('uninstall.quarantine_large_warn', { size: formatBytes(totalBytes) }) }}</span>
    </div>
    <ul class="space-y-2">
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
    </template>
  </div>
</template>
