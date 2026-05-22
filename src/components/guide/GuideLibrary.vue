<script setup lang="ts">
import { BookOpen } from 'lucide-vue-next';
import type { Guide } from '@/types';
import GuideCard from './GuideCard.vue';
import EmptyState from '../common/EmptyState.vue';
import { useI18n } from 'vue-i18n';

interface Props {
  guides: Guide[];
}
defineProps<Props>();
defineEmits<{ (e: 'open', id: string): void }>();
const { t } = useI18n();
</script>

<template>
  <div>
    <EmptyState
      v-if="guides.length === 0"
      :icon="BookOpen"
      :title="t('guide.empty_title')"
      :description="t('guide.empty_desc')"
    />
    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <GuideCard
        v-for="g in guides"
        :key="g.id"
        :guide="g"
        @open="(id) => $emit('open', id)"
      />
    </div>
  </div>
</template>
