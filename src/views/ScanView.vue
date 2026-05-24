<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Stethoscope, Zap, Play } from 'lucide-vue-next';
import { useScanStore } from '@/stores/scan';
import { useNavStore } from '@/stores/nav';
import { useScan } from '@/composables/useScan';
import type { Priority } from '@/types';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseCard from '@/components/common/BaseCard.vue';
import TabBar from '@/components/common/TabBar.vue';
import ScanProgress from '@/components/diagnostic/ScanProgress.vue';
import ScanSummary from '@/components/diagnostic/ScanSummary.vue';
import DiagnosticList from '@/components/diagnostic/DiagnosticList.vue';
import PriorityFilter from '@/components/diagnostic/PriorityFilter.vue';

const scan = useScanStore();
const nav = useNavStore();
const { run } = useScan();
const { t } = useI18n();

const eligibleCount = computed(
  () =>
    scan.findings.filter(
      (f) => f.action_type === 'automatic' || f.action_type === 'reboot',
    ).length,
);

type Tab = 'all' | 'automatic' | 'guided' | 'reboot' | 'not_possible';
const tab = ref<Tab>('all');
const filters = ref<Priority[]>(['critical', 'high', 'medium', 'low']);

const tabs = computed(() => [
  { key: 'all' as Tab, label: t('scan.tab_all'), badge: scan.totalCount },
  {
    key: 'automatic' as Tab,
    label: t('action_type.automatic'),
    badge: scan.findingsByActionType.automatic?.length ?? 0,
  },
  {
    key: 'guided' as Tab,
    label: t('action_type.guided'),
    badge: scan.findingsByActionType.guided?.length ?? 0,
  },
  {
    key: 'reboot' as Tab,
    label: t('action_type.reboot'),
    badge: scan.findingsByActionType.reboot?.length ?? 0,
  },
  {
    key: 'not_possible' as Tab,
    label: t('action_type.not_possible'),
    badge: scan.findingsByActionType.not_possible?.length ?? 0,
  },
]);

const visible = computed(() => {
  const base =
    tab.value === 'all'
      ? scan.findings
      : scan.findings.filter((f) => f.action_type === tab.value);
  return base.filter((f) => filters.value.includes(f.priority));
});

const priorityCounts = computed(() => ({
  critical: scan.findingsByPriority.critical?.length ?? 0,
  high: scan.findingsByPriority.high?.length ?? 0,
  medium: scan.findingsByPriority.medium?.length ?? 0,
  low: scan.findingsByPriority.low?.length ?? 0,
}));
</script>

<template>
  <div class="p-6 space-y-4 max-w-6xl mx-auto">
    <header class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-fg">{{ t('view.scan.title') }}</h1>
        <p class="text-sm text-fg-muted mt-1">{{ t('view.scan.subtitle') }}</p>
      </div>
      <div class="flex gap-2">
        <BaseButton variant="secondary" :icon="Zap" :loading="scan.status === 'running' && scan.kind === 'quick'" @click="run('quick')">
          {{ t('scan.quick') }}
        </BaseButton>
        <BaseButton :icon="Stethoscope" :loading="scan.status === 'running' && scan.kind === 'deep'" @click="run('deep')">
          {{ t('scan.deep') }}
        </BaseButton>
      </div>
    </header>

    <ScanProgress />

    <BaseCard v-if="scan.lastResult" :title="t('view.scan.summary')">
      <ScanSummary />
      <template v-if="eligibleCount > 0" #footer>
        <div class="flex items-center justify-between w-full">
          <span class="text-sm text-fg-muted">
            {{ t('view.scan.eligible_hint', { n: eligibleCount }) }}
          </span>
          <BaseButton :icon="Play" @click="nav.go('plan')">
            {{ t('view.scan.go_plan') }}
          </BaseButton>
        </div>
      </template>
    </BaseCard>

    <div v-if="scan.lastResult" class="space-y-3">
      <TabBar v-model="tab" :tabs="tabs" />
      <PriorityFilter v-model="filters" :counts="priorityCounts" />
      <DiagnosticList :findings="visible" />
    </div>
  </div>
</template>
