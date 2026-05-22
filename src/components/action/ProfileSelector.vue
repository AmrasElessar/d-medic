<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { CheckCircle2 } from 'lucide-vue-next';
import { PROFILES } from '@/constants/profiles';
import type { ProfileKind } from '@/types';

interface Props {
  modelValue: ProfileKind;
}
defineProps<Props>();
defineEmits<{ (e: 'update:modelValue', value: ProfileKind): void }>();

const { t } = useI18n();
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
    <button
      v-for="profile in PROFILES"
      :key="profile.kind"
      class="text-left p-4 rounded-lg border transition-all"
      :class="
        modelValue === profile.kind
          ? 'border-accent bg-accent/10'
          : 'border-border bg-bg-subtle hover:border-border-strong'
      "
      @click="$emit('update:modelValue', profile.kind)"
    >
      <div class="flex items-start justify-between mb-2">
        <h4 class="text-sm font-semibold text-fg">{{ t(profile.labelKey) }}</h4>
        <CheckCircle2
          v-if="modelValue === profile.kind"
          class="w-4 h-4 text-accent shrink-0"
        />
      </div>
      <p class="text-xs text-fg-muted mb-2 leading-relaxed">
        {{ t(profile.descKey) }}
      </p>
      <p class="text-[10px] text-fg-subtle">{{ profile.recommendedFor }}</p>
    </button>
  </div>
</template>
