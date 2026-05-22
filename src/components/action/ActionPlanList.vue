<script setup lang="ts">
import type { Finding } from '@/types';
import PlanItemRow from './PlanItemRow.vue';

interface Props {
  findings: Finding[];
  selectedIds: string[];
}
defineProps<Props>();
defineEmits<{ (e: 'toggle', id: string): void }>();
</script>

<template>
  <div class="border border-border rounded-lg overflow-hidden divide-y divide-border">
    <PlanItemRow
      v-for="finding in findings"
      :key="finding.id"
      :finding="finding"
      :selected="selectedIds.includes(finding.id)"
      @toggle="(id) => $emit('toggle', id)"
    />
  </div>
</template>
