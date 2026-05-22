<script setup lang="ts">
import { onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { History, RotateCcw, Trash2 } from 'lucide-vue-next';
import { useSnapshotsStore } from '@/stores/snapshots';
import { useToast } from '@/composables/useToast';
import { formatError } from '@/composables/useInvoke';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseButton from '@/components/common/BaseButton.vue';
import EmptyState from '@/components/common/EmptyState.vue';
import type { Snapshot } from '@/types';

const snaps = useSnapshotsStore();
const toast = useToast();
const { t, locale } = useI18n();

onMounted(async () => {
  snaps.setLoading(true);
  try {
    const items = await invoke<Snapshot[]>('list_snapshots');
    snaps.replaceAll(items);
  } catch (e) {
    toast.error('Snapshot listesi alınamadı', formatError(e));
  } finally {
    snaps.setLoading(false);
  }
});

function formatDate(iso: string): string {
  try {
    return new Date(iso).toLocaleString(locale.value === 'tr' ? 'tr-TR' : 'en-US', {
      dateStyle: 'medium',
      timeStyle: 'short',
    });
  } catch {
    return iso;
  }
}
</script>

<template>
  <div class="p-6 space-y-4 max-w-5xl mx-auto">
    <header>
      <h1 class="text-2xl font-bold text-fg">{{ t('view.history.title') }}</h1>
      <p class="text-sm text-fg-muted mt-1">{{ t('view.history.subtitle') }}</p>
    </header>

    <EmptyState
      v-if="snaps.items.length === 0 && !snaps.loading"
      :icon="History"
      :title="t('view.history.empty_title')"
      :description="t('view.history.empty_desc')"
    />

    <BaseCard
      v-for="snap in snaps.items"
      :key="snap.id"
      variant="default"
      padding="md"
    >
      <div class="flex items-start justify-between">
        <div>
          <h4 class="text-sm font-semibold text-fg">{{ snap.description }}</h4>
          <p class="text-xs text-fg-muted mt-1">{{ formatDate(snap.timestamp) }}</p>
          <div class="flex items-center gap-3 text-xs text-fg-muted mt-2">
            <span v-if="snap.restore_point_created">✓ System Restore</span>
            <span>{{ snap.service_states.length }} servis</span>
            <span>{{ snap.registry_export_paths.length }} reg export</span>
          </div>
        </div>
        <div class="flex gap-2">
          <BaseButton size="sm" variant="secondary" :icon="RotateCcw">
            {{ t('view.history.rollback') }}
          </BaseButton>
          <BaseButton size="sm" variant="ghost" :icon="Trash2">
            {{ t('common.delete') }}
          </BaseButton>
        </div>
      </div>
    </BaseCard>
  </div>
</template>
