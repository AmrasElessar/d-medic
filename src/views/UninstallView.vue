<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  Trash2,
  ScanSearch,
  ShieldCheck,
  CheckCircle2,
  Loader2,
  Package,
  ArrowLeft,
  Eraser,
} from 'lucide-vue-next';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseBadge from '@/components/common/BaseBadge.vue';
import ProgramList from '@/components/uninstall/ProgramList.vue';
import LeftoverTree from '@/components/uninstall/LeftoverTree.vue';
import QuarantinePanel from '@/components/uninstall/QuarantinePanel.vue';
import { useUninstallStore } from '@/stores/uninstall';
import { useUninstall } from '@/composables/useUninstall';
import { useToast } from '@/composables/useToast';
import { formatBytes, formatInstallDate } from '@/utils/format';
import type {
  InstalledProgram,
  LeftoverScanResult,
  RemovalReport,
  UninstallReport,
} from '@/types';

const { t } = useI18n();
const store = useUninstallStore();
const toast = useToast();
const {
  loadPrograms,
  runUninstaller,
  scanLeftovers,
  removeLeftovers,
  loadQuarantine,
  restoreQuarantine,
  purgeQuarantine,
} = useUninstall();

type Panel = 'programs' | 'quarantine';
type Flow = 'idle' | 'uninstalling' | 'scanning' | 'review' | 'done';

const panel = ref<Panel>('programs');
const flow = ref<Flow>('idle');
const loading = ref(true);

const selected = ref<InstalledProgram | null>(null);
const uninstallReport = ref<UninstallReport | null>(null);
const scanResult = ref<LeftoverScanResult | null>(null);
const selectedLeftovers = ref<string[]>([]);
const removalReport = ref<RemovalReport | null>(null);

onMounted(async () => {
  await Promise.all([loadPrograms(), loadQuarantine()]);
  loading.value = false;
});

function pick(p: InstalledProgram): void {
  if (flow.value !== 'idle' && flow.value !== 'done') return;
  selected.value = p;
  resetFlow();
}

function resetFlow(): void {
  flow.value = 'idle';
  uninstallReport.value = null;
  scanResult.value = null;
  selectedLeftovers.value = [];
  removalReport.value = null;
}

async function uninstallAndClean(): Promise<void> {
  const p = selected.value;
  if (!p) return;
  if (!confirm(t('uninstall.confirm_uninstall', { name: p.name }))) return;

  flow.value = 'uninstalling';
  uninstallReport.value = await runUninstaller(p);
  await doScan(p, true);
}

async function scanOnly(): Promise<void> {
  const p = selected.value;
  if (!p) return;
  await doScan(p, false);
}

async function doScan(p: InstalledProgram, removeFromList: boolean): Promise<void> {
  flow.value = 'scanning';
  const res = await scanLeftovers(p);
  if (!res) {
    flow.value = 'idle';
    return;
  }
  scanResult.value = res;
  selectedLeftovers.value = res.items.filter((i) => i.default_selected).map((i) => i.id);
  if (removeFromList) store.removeProgram(p.id);
  flow.value = 'review';
}

async function doRemove(): Promise<void> {
  const res = scanResult.value;
  const p = selected.value;
  if (!res || !p) return;
  const items = res.items.filter((i) => selectedLeftovers.value.includes(i.id));
  if (!items.length) {
    finish();
    return;
  }
  const rep = await removeLeftovers(p.name, items);
  if (rep) {
    removalReport.value = rep;
    flow.value = 'done';
    toast.success(
      t('uninstall.cleaned_title'),
      t('uninstall.cleaned_desc', { removed: rep.removed, freed: formatBytes(rep.freed_bytes) }),
    );
  }
}

function finish(): void {
  resetFlow();
  selected.value = null;
}

const selectedCount = computed(() => selectedLeftovers.value.length);
const freedEstimate = computed(() => {
  const res = scanResult.value;
  if (!res) return 0;
  return res.items
    .filter((i) => selectedLeftovers.value.includes(i.id))
    .reduce((sum, i) => sum + (i.size_bytes ?? 0), 0);
});
</script>

<template>
  <div class="p-6 space-y-4 max-w-6xl mx-auto">
    <header class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-fg">{{ t('view.uninstall.title') }}</h1>
        <p class="text-sm text-fg-muted mt-1">{{ t('view.uninstall.subtitle') }}</p>
      </div>
      <div class="flex gap-1 rounded-lg border border-border p-1 bg-bg-subtle">
        <button
          class="px-3 py-1.5 text-sm rounded transition-colors"
          :class="panel === 'programs' ? 'bg-accent/15 text-fg font-medium' : 'text-fg-muted hover:text-fg'"
          @click="panel = 'programs'"
        >
          {{ t('uninstall.tab_programs') }}
        </button>
        <button
          class="px-3 py-1.5 text-sm rounded transition-colors flex items-center gap-1.5"
          :class="panel === 'quarantine' ? 'bg-accent/15 text-fg font-medium' : 'text-fg-muted hover:text-fg'"
          @click="panel = 'quarantine'"
        >
          {{ t('uninstall.tab_quarantine') }}
          <BaseBadge v-if="store.quarantine.length" variant="neutral">{{ store.quarantine.length }}</BaseBadge>
        </button>
      </div>
    </header>

    <!-- KARANTINA -->
    <BaseCard v-if="panel === 'quarantine'" :title="t('uninstall.quarantine_title')" :subtitle="t('uninstall.quarantine_subtitle')">
      <QuarantinePanel
        :entries="store.quarantine"
        @restore="restoreQuarantine"
        @purge="purgeQuarantine"
      />
    </BaseCard>

    <!-- PROGRAMLAR -->
    <div v-else class="grid grid-cols-1 lg:grid-cols-2 gap-4" style="min-height: 28rem">
      <!-- Sol: program listesi -->
      <BaseCard padding="sm" class="flex flex-col">
        <div v-if="loading" class="flex-1 grid place-items-center text-fg-muted">
          <Loader2 class="w-6 h-6 animate-spin" />
        </div>
        <ProgramList
          v-else
          :programs="store.programs"
          :selected-id="selected?.id ?? null"
          class="flex-1"
          @select="pick"
        />
      </BaseCard>

      <!-- Sağ: detay / sihirbaz -->
      <BaseCard padding="md" class="flex flex-col">
        <!-- Hiç seçim yok -->
        <div v-if="!selected" class="flex-1 grid place-items-center text-center text-fg-muted px-6">
          <div>
            <Package class="w-10 h-10 mx-auto mb-3 text-fg-subtle" />
            <p class="text-sm">{{ t('uninstall.pick_hint') }}</p>
          </div>
        </div>

        <template v-else>
          <!-- Program başlığı -->
          <div class="flex items-start gap-3 pb-3 border-b border-border">
            <Package class="w-5 h-5 text-fg-muted mt-0.5" />
            <div class="flex-1 min-w-0">
              <h3 class="text-base font-semibold text-fg truncate">{{ selected.name }}</h3>
              <p class="text-xs text-fg-muted">
                {{ selected.publisher || t('uninstall.unknown_publisher') }}
                <span v-if="selected.version"> · {{ selected.version }}</span>
              </p>
              <p class="text-xs text-fg-subtle mt-0.5">
                {{ formatBytes(selected.size_bytes) }} · {{ formatInstallDate(selected.install_date) }}
              </p>
            </div>
          </div>

          <!-- IDLE: aksiyonlar -->
          <div v-if="flow === 'idle'" class="flex-1 flex flex-col justify-center gap-3 py-6">
            <BaseButton :icon="Trash2" variant="danger" full-width @click="uninstallAndClean">
              {{ t('uninstall.action_uninstall') }}
            </BaseButton>
            <BaseButton :icon="ScanSearch" variant="secondary" full-width @click="scanOnly">
              {{ t('uninstall.action_scan_only') }}
            </BaseButton>
            <p class="text-xs text-fg-subtle text-center px-4">{{ t('uninstall.action_hint') }}</p>
          </div>

          <!-- ÇALIŞIYOR -->
          <div v-else-if="flow === 'uninstalling' || flow === 'scanning'" class="flex-1 grid place-items-center text-center">
            <div>
              <Loader2 class="w-8 h-8 animate-spin mx-auto mb-3 text-accent" />
              <p class="text-sm text-fg">
                {{ flow === 'uninstalling' ? t('uninstall.busy_uninstall') : t('uninstall.busy_scan') }}
              </p>
            </div>
          </div>

          <!-- REVIEW: kalıntılar -->
          <div v-else-if="flow === 'review' && scanResult" class="flex-1 flex flex-col min-h-0">
            <div class="flex items-center gap-2 py-2 text-xs text-fg-muted">
              <BaseBadge variant="info">{{ t('uninstall.found_files', { n: scanResult.file_hits }) }}</BaseBadge>
              <BaseBadge variant="info">{{ t('uninstall.found_regs', { n: scanResult.registry_hits }) }}</BaseBadge>
            </div>
            <div class="flex-1 overflow-y-auto pr-1">
              <LeftoverTree
                :items="scanResult.items"
                v-model:selected="selectedLeftovers"
              />
            </div>
            <div class="pt-3 mt-2 border-t border-border flex items-center justify-between gap-2">
              <span class="text-xs text-fg-muted">
                {{ t('uninstall.selected_summary', { n: selectedCount, size: formatBytes(freedEstimate) }) }}
              </span>
              <div class="flex gap-2">
                <BaseButton variant="ghost" size="sm" :icon="ArrowLeft" @click="finish">
                  {{ t('common.cancel') }}
                </BaseButton>
                <BaseButton variant="danger" size="sm" :icon="Eraser" :disabled="!selectedCount" @click="doRemove">
                  {{ t('uninstall.remove_selected', { n: selectedCount }) }}
                </BaseButton>
              </div>
            </div>
          </div>

          <!-- DONE -->
          <div v-else-if="flow === 'done' && removalReport" class="flex-1 grid place-items-center text-center">
            <div>
              <CheckCircle2 class="w-10 h-10 mx-auto mb-3 text-priority-low" />
              <p class="text-base font-semibold text-fg">{{ t('uninstall.done_title') }}</p>
              <p class="text-sm text-fg-muted mt-1">
                {{ t('uninstall.done_desc', { removed: removalReport.removed, freed: formatBytes(removalReport.freed_bytes) }) }}
              </p>
              <p v-if="removalReport.failed" class="text-xs text-priority-medium mt-1">
                {{ t('uninstall.done_failed', { n: removalReport.failed }) }}
              </p>
              <div class="mt-4 flex items-center justify-center gap-2 text-xs text-fg-subtle">
                <ShieldCheck class="w-4 h-4" />
                {{ t('uninstall.done_quarantine_hint') }}
              </div>
              <BaseButton class="mt-4" @click="finish">{{ t('common.done') }}</BaseButton>
            </div>
          </div>
        </template>
      </BaseCard>
    </div>
  </div>
</template>
