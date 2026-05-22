<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Zap, BookOpen, RotateCw, Ban } from 'lucide-vue-next';
import type { ActionType } from '@/types';

interface Props {
  actionType: ActionType;
}
defineProps<Props>();
const { t } = useI18n();
</script>

<template>
  <span
    class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium border"
    :class="{
      'bg-accent/15 text-accent border-accent/30': actionType === 'automatic',
      'bg-priority-medium/15 text-priority-medium border-priority-medium/30': actionType === 'guided',
      'bg-priority-high/15 text-priority-high border-priority-high/30': actionType === 'reboot',
      'bg-fg-subtle/20 text-fg-subtle border-fg-subtle/30': actionType === 'not_possible',
    }"
  >
    <Zap v-if="actionType === 'automatic'" class="w-3 h-3" />
    <BookOpen v-if="actionType === 'guided'" class="w-3 h-3" />
    <RotateCw v-if="actionType === 'reboot'" class="w-3 h-3" />
    <Ban v-if="actionType === 'not_possible'" class="w-3 h-3" />
    {{ t(`action_type.${actionType}`) }}
  </span>
</template>
