<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Play, CheckSquare, Square } from 'lucide-vue-next';
import { useScanStore } from '@/stores/scan';
import { usePlanStore } from '@/stores/plan';
import { useNavStore } from '@/stores/nav';
import BaseButton from '@/components/common/BaseButton.vue';
import BaseCard from '@/components/common/BaseCard.vue';
import EmptyState from '@/components/common/EmptyState.vue';
import ProfileSelector from '@/components/action/ProfileSelector.vue';
import ActionPlanList from '@/components/action/ActionPlanList.vue';

const scan = useScanStore();
const plan = usePlanStore();
const nav = useNavStore();
const { t } = useI18n();

const eligible = computed(() =>
  scan.findings.filter(
    (f) => f.action_type === 'automatic' || f.action_type === 'reboot',
  ),
);

const selectedCount = computed(() => plan.selectedFindingIds.length);
const needsReboot = computed(() =>
  eligible.value.some(
    (f) => plan.selectedFindingIds.includes(f.id) && f.reboot_required,
  ),
);

function selectAll() {
  plan.selectAll(eligible.value);
}
function clear() {
  plan.clear();
}
function proceed() {
  nav.go('execute');
}
</script>

<template>
  <div class="p-6 space-y-4 max-w-6xl mx-auto">
    <header>
      <h1 class="text-2xl font-bold text-fg">{{ t('view.plan.title') }}</h1>
      <p class="text-sm text-fg-muted mt-1">{{ t('view.plan.subtitle') }}</p>
    </header>

    <BaseCard :title="t('view.plan.profile')">
      <ProfileSelector v-model="plan.profile" />
    </BaseCard>

    <BaseCard :title="t('view.plan.selected')">
      <template #header>
        <div class="flex items-center gap-2">
          <BaseButton size="sm" variant="ghost" :icon="CheckSquare" @click="selectAll">
            {{ t('view.plan.select_all') }}
          </BaseButton>
          <BaseButton size="sm" variant="ghost" :icon="Square" @click="clear">
            {{ t('view.plan.clear') }}
          </BaseButton>
        </div>
      </template>

      <EmptyState
        v-if="eligible.length === 0"
        :title="t('view.plan.no_eligible_title')"
        :description="t('view.plan.no_eligible_desc')"
      />
      <ActionPlanList
        v-else
        :findings="eligible"
        :selected-ids="plan.selectedFindingIds"
        @toggle="(id) => plan.toggle(id)"
      />

      <template #footer>
        <div class="flex items-center justify-between w-full">
          <div class="text-sm text-fg-muted">
            {{ t('view.plan.count', { n: selectedCount }) }}
            <span v-if="needsReboot" class="text-priority-high ml-2">
              · {{ t('view.plan.needs_reboot') }}
            </span>
          </div>
          <BaseButton
            :icon="Play"
            :disabled="selectedCount === 0"
            @click="proceed"
          >
            {{ t('view.plan.continue') }}
          </BaseButton>
        </div>
      </template>
    </BaseCard>
  </div>
</template>
