<script setup lang="ts">
import { Search } from 'lucide-vue-next';
import type { Finding } from '@/types';
import DiagnosticCard from './DiagnosticCard.vue';
import EmptyState from '../common/EmptyState.vue';
import { useI18n } from 'vue-i18n';

interface Props {
  findings: Finding[];
  selectable?: boolean;
  selectedIds?: string[];
}
withDefaults(defineProps<Props>(), {
  selectable: false,
  selectedIds: () => [],
});

defineEmits<{
  (e: 'toggle', id: string): void;
  (e: 'open', id: string): void;
}>();

const { t } = useI18n();
</script>

<template>
  <div>
    <div v-if="findings.length === 0">
      <EmptyState
        :icon="Search"
        :title="t('diagnostic.empty_title')"
        :description="t('diagnostic.empty_desc')"
      />
    </div>
    <div v-else class="space-y-2">
      <DiagnosticCard
        v-for="finding in findings"
        :key="finding.id"
        :finding="finding"
        :selectable="selectable"
        :selected="selectedIds.includes(finding.id)"
        @toggle="(id) => $emit('toggle', id)"
        @open="(id) => $emit('open', id)"
      />
    </div>
  </div>
</template>
