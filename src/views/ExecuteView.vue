<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Play, RotateCw, FileCheck, AlertTriangle } from 'lucide-vue-next';
import { useScanStore } from '@/stores/scan';
import { usePlanStore } from '@/stores/plan';
import { useExecutionStore } from '@/stores/execution';
import { useSettingsStore } from '@/stores/settings';
import { useToast } from '@/composables/useToast';
import { useNavStore } from '@/stores/nav';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseToggle from '@/components/common/BaseToggle.vue';
import EmptyState from '@/components/common/EmptyState.vue';
import ExecutionMonitor from '@/components/action/ExecutionMonitor.vue';
import ResultSummary from '@/components/action/ResultSummary.vue';
import RebootPrompt from '@/components/action/RebootPrompt.vue';

const scan = useScanStore();
const plan = usePlanStore();
const exec = useExecutionStore();
const settings = useSettingsStore();
const nav = useNavStore();
const toast = useToast();
const { t } = useI18n();

const showReboot = ref(false);

const selectedFindings = computed(() =>
  scan.findings.filter((f) => plan.selectedFindingIds.includes(f.id)),
);

const isFinished = computed(
  () => exec.status === 'completed' || exec.status === 'failed',
);

async function start() {
  if (selectedFindings.value.length === 0) {
    toast.warning(t('exec.empty_selection_title'), t('exec.empty_selection_desc'));
    return;
  }
  // TODO Faz 2: gerçek plan oluştur + Tauri invoke('execute_plan', { ... })
  toast.info(t('exec.stub_title'), t('exec.stub_desc'));
}

function reset() {
  exec.reset();
}
function backToPlan() {
  nav.go('plan');
}
</script>

<template>
  <div class="p-6 space-y-4 max-w-6xl mx-auto">
    <header>
      <h1 class="text-2xl font-bold text-fg">{{ t('view.execute.title') }}</h1>
      <p class="text-sm text-fg-muted mt-1">{{ t('view.execute.subtitle') }}</p>
    </header>

    <EmptyState
      v-if="selectedFindings.length === 0 && exec.status === 'idle'"
      :icon="FileCheck"
      :title="t('view.execute.empty_title')"
      :description="t('view.execute.empty_desc')"
    >
      <template #actions>
        <BaseButton @click="backToPlan">{{ t('view.execute.go_plan') }}</BaseButton>
      </template>
    </EmptyState>

    <BaseCard
      v-else-if="exec.status === 'idle'"
      :title="t('view.execute.preflight')"
    >
      <div class="space-y-3">
        <p class="text-sm text-fg-muted">
          {{ t('view.execute.preflight_desc', { n: selectedFindings.length }) }}
        </p>
        <BaseToggle
          v-model="settings.createRestorePoint"
          :label="t('view.execute.opt_restore_point')"
          :hint="t('view.execute.opt_restore_point_hint')"
        />
        <BaseToggle
          v-model="settings.exportRegistry"
          :label="t('view.execute.opt_reg_export')"
          :hint="t('view.execute.opt_reg_export_hint')"
        />
        <BaseToggle
          v-model="settings.confirmBeforeExecute"
          :label="t('view.execute.opt_confirm')"
        />

        <div class="flex items-start gap-2 p-3 bg-priority-medium/10 border border-priority-medium/30 rounded text-sm">
          <AlertTriangle class="w-4 h-4 text-priority-medium shrink-0 mt-0.5" />
          <span>{{ t('view.execute.warning_uwp') }}</span>
        </div>
      </div>

      <template #footer>
        <div class="flex justify-end gap-2 w-full">
          <BaseButton variant="secondary" @click="backToPlan">
            {{ t('common.back') }}
          </BaseButton>
          <BaseButton :icon="Play" @click="start">
            {{ t('view.execute.start') }}
          </BaseButton>
        </div>
      </template>
    </BaseCard>

    <BaseCard
      v-else
      :title="t('view.execute.in_progress')"
    >
      <ExecutionMonitor />
    </BaseCard>

    <BaseCard
      v-if="isFinished"
      :title="t('view.execute.result')"
    >
      <ResultSummary />
      <template #footer>
        <div class="flex justify-end gap-2 w-full">
          <BaseButton variant="secondary" @click="reset">
            {{ t('view.execute.new_plan') }}
          </BaseButton>
          <BaseButton :icon="RotateCw" variant="primary" @click="showReboot = true">
            {{ t('reboot.now') }}
          </BaseButton>
        </div>
      </template>
    </BaseCard>

    <RebootPrompt
      :open="showReboot"
      @close="showReboot = false"
      @postpone="showReboot = false"
      @reboot="showReboot = false"
    />
  </div>
</template>
