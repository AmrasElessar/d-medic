<script setup lang="ts">
import { computed, onMounted, reactive, ref, type Component } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import {
  Trash2,
  HardDrive,
  ShieldCheck,
  FileCheck2,
  RefreshCw,
  CheckCircle2,
  XCircle,
  Info,
} from 'lucide-vue-next';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseButton from '@/components/common/BaseButton.vue';
import { useSnapshotsStore } from '@/stores/snapshots';
import { useToast } from '@/composables/useToast';
import { formatError } from '@/composables/useInvoke';
import type { ActionResult, PhysicalDiskInfo, Snapshot } from '@/types';

interface Tool {
  id: string;
  icon: Component;
  /** Uzun süren işlemler (dakikalar). true → confirm dialog gösterilir. */
  longRunning?: boolean;
}

// Tarama bağımsız çalışabilen, "gerekli olduğunda zarar vermez" işlemler.
// Bloatware kaldırma / VBS kapatma / telemetri kapatma gibi geri alınması
// görece zor olanlar burada YOK — onlar tarama → plan → yürüt akışında.
const TOOLS: Tool[] = [
  { id: 'clean-temp', icon: Trash2 },
  { id: 'defrag-system', icon: HardDrive, longRunning: true },
  { id: 'sfc-repair', icon: FileCheck2, longRunning: true },
  { id: 'dism-restore-health', icon: ShieldCheck, longRunning: true },
  { id: 'reset-windows-update', icon: RefreshCw, longRunning: true },
];

type State = 'idle' | 'running' | 'success' | 'failed';
const status = reactive<Record<string, State>>(
  Object.fromEntries(TOOLS.map((t) => [t.id, 'idle'])),
);
const messages = reactive<Record<string, string>>({});

const { t } = useI18n();
const snaps = useSnapshotsStore();
const toast = useToast();

// Disk listesini bir kez yükle — defrag butonu SSD-only sistemlerde disabled.
// WMI sorgusu sessiz fail edebilir; o durumda eski davranış (her disk için
// çalıştır) korunur. mediaType "Unknown" → "HDD" sayılır (varsayılan güvenli).
const disks = ref<PhysicalDiskInfo[]>([]);
onMounted(async () => {
  try {
    disks.value = await invoke<PhysicalDiskInfo[]>('list_disks');
  } catch {
    // WMI fail; defrag butonu varsayılan davranışla görünür
  }
});

const onlySsd = computed(
  () => disks.value.length > 0 && disks.value.every((d) => d.media_type === 'SSD'),
);
const ssdCount = computed(() => disks.value.filter((d) => d.media_type === 'SSD').length);
const hddCount = computed(() => disks.value.filter((d) => d.media_type === 'HDD').length);

function isDisabled(tool: Tool): boolean {
  if (status[tool.id] === 'running') return true;
  // SSD-only sistemde defrag disabled
  if (tool.id === 'defrag-system' && onlySsd.value) return true;
  return false;
}

function disabledReason(tool: Tool): string {
  if (tool.id === 'defrag-system' && onlySsd.value) {
    return t('view.dashboard.tools_ssd_only', { n: ssdCount.value });
  }
  return '';
}

function diskMixNote(tool: Tool): string {
  if (tool.id === 'defrag-system' && ssdCount.value > 0 && hddCount.value > 0) {
    return t('view.dashboard.tools_mixed_disks', {
      ssd: ssdCount.value,
      hdd: hddCount.value,
    });
  }
  return '';
}

async function run(tool: Tool) {
  if (status[tool.id] === 'running') return;
  if (tool.longRunning && !confirm(t('view.dashboard.tools_confirm_long'))) {
    return;
  }

  status[tool.id] = 'running';
  delete messages[tool.id];

  try {
    const result = await invoke<ActionResult>('apply_action', { actionId: tool.id });
    status[tool.id] = result.outcome.success ? 'success' : 'failed';
    messages[tool.id] = result.outcome.message;

    // Snapshot listesini yenile — UI'daki "Geçmiş" bölümü güncel kalsın.
    if (result.snapshot_id) {
      try {
        const items = await invoke<Snapshot[]>('list_snapshots');
        snaps.replaceAll(items);
      } catch {
        // best-effort
      }
    }

    if (result.outcome.success) {
      toast.success(
        t(`action.${tool.id}.title`),
        result.outcome.message || t('view.dashboard.tools_success'),
      );
    } else {
      toast.error(
        t(`action.${tool.id}.title`),
        result.outcome.message || t('view.dashboard.tools_failed'),
      );
    }
  } catch (e) {
    status[tool.id] = 'failed';
    const msg = formatError(e);
    messages[tool.id] = msg;
    toast.error(t(`action.${tool.id}.title`), msg);
  }
}
</script>

<template>
  <BaseCard :title="t('view.dashboard.tools_title')">
    <p class="text-xs text-fg-muted mb-3">{{ t('view.dashboard.tools_desc') }}</p>
    <div class="divide-y divide-border">
      <div
        v-for="tool in TOOLS"
        :key="tool.id"
        class="flex items-center gap-3 py-2.5"
      >
        <component
          :is="tool.icon"
          :class="[
            'w-4 h-4 shrink-0',
            isDisabled(tool) && tool.id === 'defrag-system' ? 'text-fg-subtle' : 'text-fg-muted',
          ]"
        />
        <div class="flex-1 min-w-0">
          <div
            :class="[
              'text-sm truncate',
              isDisabled(tool) && tool.id === 'defrag-system' ? 'text-fg-muted' : 'text-fg',
            ]"
          >
            {{ t(`action.${tool.id}.title`) }}
          </div>
          <!-- Disabled sebep (SSD-only) -->
          <div
            v-if="disabledReason(tool)"
            class="text-xs text-priority-medium truncate flex items-center gap-1"
            :title="disabledReason(tool)"
          >
            <Info class="w-3 h-3 shrink-0" />
            {{ disabledReason(tool) }}
          </div>
          <!-- Karışık disk uyarısı (HDD + SSD birlikte) -->
          <div
            v-else-if="diskMixNote(tool)"
            class="text-xs text-priority-medium truncate flex items-center gap-1"
            :title="diskMixNote(tool)"
          >
            <Info class="w-3 h-3 shrink-0" />
            {{ diskMixNote(tool) }}
          </div>
          <div
            v-else-if="messages[tool.id]"
            class="text-xs text-fg-muted truncate"
            :title="messages[tool.id]"
          >
            {{ messages[tool.id] }}
          </div>
        </div>

        <CheckCircle2
          v-if="status[tool.id] === 'success'"
          class="w-4 h-4 text-priority-low shrink-0"
        />
        <XCircle
          v-else-if="status[tool.id] === 'failed'"
          class="w-4 h-4 text-priority-critical shrink-0"
        />

        <BaseButton
          size="sm"
          variant="secondary"
          :loading="status[tool.id] === 'running'"
          :disabled="isDisabled(tool)"
          :title="disabledReason(tool)"
          @click="run(tool)"
        >
          {{ t('view.dashboard.tools_run') }}
        </BaseButton>
      </div>
    </div>
  </BaseCard>
</template>
