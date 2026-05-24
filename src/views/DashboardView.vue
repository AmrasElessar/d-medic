<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Stethoscope, Play, BookOpen, History, Activity, Cpu, HardDrive, MemoryStick } from 'lucide-vue-next';
import { useScanStore } from '@/stores/scan';
import { useNavStore } from '@/stores/nav';
import { useSystemStore } from '@/stores/system';
import { useScan } from '@/composables/useScan';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseButton from '@/components/common/BaseButton.vue';
import StatPill from '@/components/common/StatPill.vue';
import ScanSummary from '@/components/diagnostic/ScanSummary.vue';
import QuickTools from '@/components/dashboard/QuickTools.vue';

const scan = useScanStore();
const nav = useNavStore();
const sys = useSystemStore();
const { run } = useScan();
const { t } = useI18n();

const hasResults = computed(() => !!scan.lastResult);

async function runQuick() {
  await run('quick');
  if (scan.lastResult) nav.go('scan');
}
</script>

<template>
  <div class="p-6 space-y-6 max-w-6xl mx-auto">
    <header class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-fg">{{ t('view.dashboard.title') }}</h1>
        <p class="text-sm text-fg-muted mt-1">{{ t('view.dashboard.subtitle') }}</p>
      </div>
      <BaseButton :icon="Stethoscope" :loading="scan.status === 'running'" @click="runQuick">
        {{ t('scan.quick') }}
      </BaseButton>
    </header>

    <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
      <StatPill :icon="Cpu" label="CPU" :value="sys.cpuText" />
      <StatPill :icon="MemoryStick" label="RAM" :value="sys.ramText" />
      <StatPill
        :icon="HardDrive"
        :label="`Disk ${sys.stats?.primary_disk_type || ''}`.trim()"
        :value="sys.diskText"
      />
      <StatPill
        :icon="Activity"
        :label="t('status.elevated')"
        :value="sys.isElevated ? 'ADMIN' : 'USER'"
        :tone="sys.isElevated ? 'success' : 'danger'"
      />
    </div>

    <BaseCard :title="t('view.dashboard.last_scan')">
      <div v-if="hasResults">
        <ScanSummary />
        <div class="flex gap-2 mt-4">
          <BaseButton variant="secondary" :icon="Play" @click="nav.go('plan')">
            {{ t('view.dashboard.go_plan') }}
          </BaseButton>
          <BaseButton variant="ghost" @click="nav.go('scan')">
            {{ t('view.dashboard.view_findings') }}
          </BaseButton>
        </div>
      </div>
      <div v-else class="text-center py-8">
        <Activity class="w-10 h-10 text-fg-subtle mx-auto mb-3" />
        <p class="text-sm text-fg-muted">{{ t('view.dashboard.no_scan') }}</p>
        <BaseButton class="mt-3" :icon="Stethoscope" @click="runQuick">
          {{ t('scan.quick') }}
        </BaseButton>
      </div>
    </BaseCard>

    <QuickTools />

    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
      <BaseCard :title="t('view.dashboard.quick_actions')">
        <div class="space-y-2">
          <BaseButton full-width variant="secondary" :icon="Stethoscope" @click="nav.go('scan')">
            {{ t('view.dashboard.run_scan') }}
          </BaseButton>
          <BaseButton full-width variant="secondary" :icon="BookOpen" @click="nav.go('guides')">
            {{ t('view.dashboard.browse_guides') }}
          </BaseButton>
          <BaseButton full-width variant="secondary" :icon="History" @click="nav.go('history')">
            {{ t('view.dashboard.view_history') }}
          </BaseButton>
        </div>
      </BaseCard>

      <BaseCard :title="t('view.dashboard.tips')" class="md:col-span-2">
        <ul class="text-sm text-fg-muted space-y-2 leading-relaxed">
          <li>• {{ t('view.dashboard.tip_1') }}</li>
          <li>• {{ t('view.dashboard.tip_2') }}</li>
          <li>• {{ t('view.dashboard.tip_3') }}</li>
        </ul>
      </BaseCard>
    </div>
  </div>
</template>
