<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { ShieldCheck, ShieldAlert, ShieldQuestion, ShieldX } from 'lucide-vue-next';
import type { VerificationLevel } from '@/types';

interface Props {
  level: VerificationLevel;
  /** Mini sürüm — sadece ikon + renk dot; küçük kartlarda */
  mini?: boolean;
}
const props = withDefaults(defineProps<Props>(), { mini: false });

defineEmits<{ (e: 'click'): void }>();

const { t } = useI18n();

const config = computed(() => {
  switch (props.level) {
    case 'safe':
      return {
        label: t('verification.safe'),
        icon: ShieldCheck,
        tone: 'success',
        ring: 'ring-priority-low/30',
        bg: 'bg-priority-low/10',
        fg: 'text-priority-low',
        border: 'border-priority-low/40',
      };
    case 'documented_alternative':
      return {
        label: t('verification.documented_alternative'),
        icon: ShieldQuestion,
        tone: 'warning',
        ring: 'ring-priority-medium/30',
        bg: 'bg-priority-medium/10',
        fg: 'text-priority-medium',
        border: 'border-priority-medium/40',
      };
    case 'tried_not_official':
      return {
        label: t('verification.tried_not_official'),
        icon: ShieldAlert,
        tone: 'warning',
        ring: 'ring-priority-high/30',
        bg: 'bg-priority-high/10',
        fg: 'text-priority-high',
        border: 'border-priority-high/40',
      };
    case 'not_recommended':
    default:
      return {
        label: t('verification.not_recommended'),
        icon: ShieldX,
        tone: 'danger',
        ring: 'ring-priority-critical/30',
        bg: 'bg-priority-critical/10',
        fg: 'text-priority-critical',
        border: 'border-priority-critical/40',
      };
  }
});
</script>

<template>
  <button
    type="button"
    :class="[
      'inline-flex items-center gap-1.5 rounded-full border transition-colors',
      mini ? 'px-1.5 py-0.5' : 'px-2 py-0.5 text-xs',
      config.bg, config.fg, config.border,
      'hover:ring-2', config.ring,
      'focus:outline-none focus:ring-2',
    ]"
    :title="config.label"
    @click.stop="$emit('click')"
  >
    <component :is="config.icon" class="w-3.5 h-3.5 shrink-0" />
    <span v-if="!mini" class="font-medium">{{ config.label }}</span>
  </button>
</template>
