<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { Play, RotateCw, FileCheck, AlertTriangle } from 'lucide-vue-next';
import { useScanStore } from '@/stores/scan';
import { usePlanStore } from '@/stores/plan';
import { useExecutionStore } from '@/stores/execution';
import { useSettingsStore } from '@/stores/settings';
import { useSnapshotsStore } from '@/stores/snapshots';
import { useToast } from '@/composables/useToast';
import { useNavStore } from '@/stores/nav';
import { formatError } from '@/composables/useInvoke';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseToggle from '@/components/common/BaseToggle.vue';
import EmptyState from '@/components/common/EmptyState.vue';
import ExecutionMonitor from '@/components/action/ExecutionMonitor.vue';
import ResultSummary from '@/components/action/ResultSummary.vue';
import RebootPrompt from '@/components/action/RebootPrompt.vue';
import type { ExecutionPlan, PlanItem, PlanResult, Snapshot } from '@/types';

const scan = useScanStore();
const plan = usePlanStore();
const exec = useExecutionStore();
const settings = useSettingsStore();
const snaps = useSnapshotsStore();
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
  // Sadece action_id'si olan finding'leri al — Guided'lar kullanıcı manuel.
  const executable = selectedFindings.value.filter((f) => !!f.action_id);
  if (executable.length === 0) {
    toast.info(
      t('exec.empty_selection_title'),
      'Seçili bulguların hiçbiri otomatik düzeltilebilir değil (yalnız kılavuz).',
    );
    return;
  }

  const planObj: ExecutionPlan = {
    plan_id: `plan-${Date.now()}`,
    profile: plan.profile,
    selected_findings: executable,
    items: executable.map<PlanItem>((f) => ({
      finding_id: f.id,
      action_id: f.action_id as string,
      status: 'pending',
      started_at: null,
      finished_at: null,
      message: null,
    })),
    snapshot_id: null,
    reboot_required: executable.some((f) => f.reboot_required),
  };

  exec.begin(planObj);
  exec.appendLog(`Plan oluşturuldu: ${executable.length} action`);

  try {
    exec.appendLog('Pre-action snapshot alınıyor...');
    const result = await invoke<PlanResult>('apply_plan', {
      actionIds: executable.map((f) => f.action_id),
    });

    if (result.snapshot_id) {
      planObj.snapshot_id = result.snapshot_id;
      exec.appendLog(`Snapshot oluşturuldu: ${result.snapshot_id.slice(0, 8)}...`);
      // Geçmiş listesini güncel tutmak için snapshot'ları yeniden yükle.
      try {
        const items = await invoke<Snapshot[]>('list_snapshots');
        snaps.replaceAll(items);
      } catch {
        // best-effort, sessiz geç
      }
    } else {
      exec.appendLog('Snapshot oluşturulamadı, action snapshotsiz devam etti.');
    }

    // outcomes sıralı geldiği için item'larla index-by-index eşleşir.
    result.outcomes.forEach((outcome, idx) => {
      exec.advance(idx);
      const item = planObj.items[idx];
      if (!item) return;
      exec.setItemStatus(
        item.finding_id,
        outcome.success ? 'success' : 'failed',
        outcome.message,
      );
      exec.appendLog(
        `[${outcome.success ? 'OK' : 'FAIL'}] ${outcome.action_id}: ${outcome.message}`,
      );
    });

    exec.complete();
    const ok = result.outcomes.filter((o) => o.success).length;
    const fail = result.outcomes.length - ok;
    toast.success(
      'Plan tamamlandı',
      `${ok} başarılı, ${fail} başarısız.`,
    );
  } catch (e) {
    const msg = formatError(e);
    exec.fail(msg);
    toast.error('Plan başarısız', msg);
  }
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
