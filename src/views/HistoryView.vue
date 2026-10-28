<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { History, RotateCcw, Trash2, ShieldAlert } from 'lucide-vue-next';
import { useSnapshotsStore } from '@/stores/snapshots';
import { useToast } from '@/composables/useToast';
import { formatError } from '@/composables/useInvoke';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseButton from '@/components/common/BaseButton.vue';
import EmptyState from '@/components/common/EmptyState.vue';
import type { RollbackReport, Snapshot } from '@/types';

const snaps = useSnapshotsStore();
const toast = useToast();
const { t, locale } = useI18n();

// id → bool: rollback/delete sırasında çift tıklamayı engelle
const busy = ref<Record<string, boolean>>({});

async function refresh() {
  snaps.setLoading(true);
  try {
    const items = await invoke<Snapshot[]>('list_snapshots');
    snaps.replaceAll(items);
  } catch (e) {
    toast.error(t('view.history.list_fail'), formatError(e));
  } finally {
    snaps.setLoading(false);
  }
}

onMounted(refresh);

async function rollback(snap: Snapshot) {
  if (busy.value[snap.id]) return;
  const msg = t('view.history.rollback_confirm', {
    desc: snap.description,
    regs: snap.registry_export_paths.length,
    svcs: snap.service_states.length,
  });
  if (!confirm(msg)) return;

  busy.value = { ...busy.value, [snap.id]: true };
  try {
    const report = await invoke<RollbackReport>('rollback_snapshot', { id: snap.id });
    const okReg = report.registry_imports.filter(([, ok]) => ok).length;
    const failReg = report.registry_imports.length - okReg;
    toast.success(
      t('view.history.rollback_done_title'),
      t('view.history.rollback_done_desc', {
        ok: okReg,
        fail: failReg,
        svcs: report.services_restored,
      }),
    );
  } catch (e) {
    toast.error(t('view.history.rollback_fail_title'), formatError(e));
  } finally {
    busy.value = { ...busy.value, [snap.id]: false };
  }
}

async function remove(snap: Snapshot) {
  if (busy.value[snap.id]) return;
  if (!confirm(t('view.history.delete_confirm', { desc: snap.description }))) return;
  busy.value = { ...busy.value, [snap.id]: true };
  try {
    await invoke('delete_snapshot', { id: snap.id });
    snaps.remove(snap.id);
    toast.info(t('view.history.delete_done_title'), snap.description);
  } catch (e) {
    toast.error(t('view.history.delete_fail_title'), formatError(e));
  } finally {
    busy.value = { ...busy.value, [snap.id]: false };
  }
}

// En yeni snapshot'ta restore point oluşmadıysa System Protection'ı açma önerisi.
const enablingProtection = ref(false);

async function enableProtection() {
  if (!confirm(t('view.history.enable_protection_confirm'))) return;
  enablingProtection.value = true;
  try {
    await invoke('enable_system_protection');
    toast.success(t('view.history.enable_protection_done'), t('view.history.enable_protection_done_desc'));
  } catch (e) {
    toast.error(t('view.history.enable_protection_fail'), formatError(e));
  } finally {
    enablingProtection.value = false;
  }
}

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
            <span
              v-else
              class="flex items-center gap-1 text-priority-medium"
              :title="snap.restore_point_note ?? ''"
            >
              <ShieldAlert class="w-3.5 h-3.5" />
              {{ t('view.history.no_restore_point') }}
            </span>
            <span>{{ t('view.history.svc_count', { n: snap.service_states.length }) }}</span>
            <span>{{ t('view.history.reg_count', { n: snap.registry_export_paths.length }) }}</span>
          </div>
          <p v-if="!snap.restore_point_created && snap.restore_point_note" class="text-xs text-fg-subtle mt-1">
            {{ snap.restore_point_note }}
          </p>
          <BaseButton
            v-if="!snap.restore_point_created && snap.id === snaps.items[0]?.id"
            class="mt-2"
            size="sm"
            variant="secondary"
            :icon="ShieldAlert"
            :loading="enablingProtection"
            @click="enableProtection"
          >
            {{ t('view.history.enable_protection') }}
          </BaseButton>
        </div>
        <div class="flex gap-2">
          <BaseButton
            size="sm"
            variant="secondary"
            :icon="RotateCcw"
            :disabled="busy[snap.id]"
            @click="rollback(snap)"
          >
            {{ t('view.history.rollback') }}
          </BaseButton>
          <BaseButton
            size="sm"
            variant="ghost"
            :icon="Trash2"
            :disabled="busy[snap.id]"
            @click="remove(snap)"
          >
            {{ t('common.delete') }}
          </BaseButton>
        </div>
      </div>
    </BaseCard>
  </div>
</template>
