<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Zap, BookOpen, RotateCw, Ban } from 'lucide-vue-next';
import { useScanStore } from '@/stores/scan';
import StatPill from '../common/StatPill.vue';

const scan = useScanStore();
const { t } = useI18n();

const counts = computed(() => {
  const byType = scan.findingsByActionType;
  return {
    automatic: byType.automatic?.length ?? 0,
    guided: byType.guided?.length ?? 0,
    reboot: byType.reboot?.length ?? 0,
    not_possible: byType.not_possible?.length ?? 0,
  };
});
</script>

<template>
  <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
    <StatPill
      :icon="Zap"
      :label="t('action_type.automatic')"
      :value="counts.automatic"
      tone="accent"
    />
    <StatPill
      :icon="BookOpen"
      :label="t('action_type.guided')"
      :value="counts.guided"
      tone="warning"
    />
    <StatPill
      :icon="RotateCw"
      :label="t('action_type.reboot')"
      :value="counts.reboot"
      tone="warning"
    />
    <StatPill
      :icon="Ban"
      :label="t('action_type.not_possible')"
      :value="counts.not_possible"
      tone="neutral"
    />
  </div>
</template>
