<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { RotateCw } from 'lucide-vue-next';
import BaseButton from '../common/BaseButton.vue';
import BaseModal from '../common/BaseModal.vue';

interface Props {
  open: boolean;
}
defineProps<Props>();
defineEmits<{
  (e: 'close'): void;
  (e: 'reboot'): void;
  (e: 'postpone'): void;
}>();

const { t } = useI18n();
</script>

<template>
  <BaseModal
    :open="open"
    :title="t('reboot.title')"
    size="md"
    @close="$emit('close')"
  >
    <div class="flex items-start gap-3">
      <div class="w-10 h-10 rounded-full bg-priority-high/15 grid place-items-center shrink-0">
        <RotateCw class="w-5 h-5 text-priority-high" />
      </div>
      <div>
        <p class="text-sm text-fg leading-relaxed">{{ t('reboot.description') }}</p>
        <p class="text-xs text-fg-muted mt-2">{{ t('reboot.hint') }}</p>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="secondary" @click="$emit('postpone')">
        {{ t('reboot.postpone') }}
      </BaseButton>
      <BaseButton variant="primary" :icon="RotateCw" @click="$emit('reboot')">
        {{ t('reboot.now') }}
      </BaseButton>
    </template>
  </BaseModal>
</template>
