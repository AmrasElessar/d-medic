<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useSettingsStore } from '@/stores/settings';
import { useTheme } from '@/composables/useTheme';
import BaseCard from '@/components/common/BaseCard.vue';
import BaseSelect from '@/components/common/BaseSelect.vue';
import BaseToggle from '@/components/common/BaseToggle.vue';
import TabBar from '@/components/common/TabBar.vue';

const settings = useSettingsStore();
const { apply } = useTheme();
const { t, locale } = useI18n();

type Tab = 'general' | 'appearance' | 'logs' | 'safety';
const tab = ref<Tab>('general');

const tabs = [
  { key: 'general' as Tab,    label: 'Genel' },
  { key: 'appearance' as Tab, label: 'Görünüm' },
  { key: 'safety' as Tab,     label: 'Güvenlik' },
  { key: 'logs' as Tab,       label: 'Loglar' },
];

const themeOptions = [
  { value: 'dark',   label: t('settings.theme_dark') },
  { value: 'light',  label: t('settings.theme_light') },
  { value: 'system', label: t('settings.theme_system') },
];
const localeOptions = [
  { value: 'tr', label: 'Türkçe' },
  { value: 'en', label: 'English' },
];
const logOptions = [
  { value: 'debug', label: 'debug' },
  { value: 'info',  label: 'info' },
  { value: 'warn',  label: 'warn' },
  { value: 'error', label: 'error' },
];

watch(
  () => settings.locale,
  (v) => {
    locale.value = v;
    settings.persist();
  },
);

watch(
  () => settings.theme,
  () => {
    apply();
    settings.persist();
  },
);

watch(
  () => [
    settings.autoScanOnStart,
    settings.confirmBeforeExecute,
    settings.createRestorePoint,
    settings.exportRegistry,
    settings.logLevel,
  ],
  () => settings.persist(),
);
</script>

<template>
  <div class="p-6 space-y-4 max-w-3xl mx-auto">
    <header>
      <h1 class="text-2xl font-bold text-fg">{{ t('view.settings.title') }}</h1>
      <p class="text-sm text-fg-muted mt-1">{{ t('view.settings.subtitle') }}</p>
    </header>

    <TabBar v-model="tab" :tabs="tabs" />

    <BaseCard v-if="tab === 'general'">
      <div class="space-y-4">
        <BaseSelect
          v-model="settings.locale"
          :options="localeOptions"
          :label="t('settings.language')"
        />
        <BaseToggle
          v-model="settings.autoScanOnStart"
          :label="t('settings.auto_scan')"
          :hint="t('settings.auto_scan_hint')"
        />
      </div>
    </BaseCard>

    <BaseCard v-if="tab === 'appearance'">
      <BaseSelect
        v-model="settings.theme"
        :options="themeOptions"
        :label="t('settings.theme')"
      />
    </BaseCard>

    <BaseCard v-if="tab === 'safety'">
      <div class="space-y-4">
        <BaseToggle
          v-model="settings.createRestorePoint"
          :label="t('settings.restore_point')"
          :hint="t('settings.restore_point_hint')"
        />
        <BaseToggle
          v-model="settings.exportRegistry"
          :label="t('settings.reg_export')"
          :hint="t('settings.reg_export_hint')"
        />
        <BaseToggle
          v-model="settings.confirmBeforeExecute"
          :label="t('settings.confirm_execute')"
        />
      </div>
    </BaseCard>

    <BaseCard v-if="tab === 'logs'">
      <BaseSelect
        v-model="settings.logLevel"
        :options="logOptions"
        :label="t('settings.log_level')"
      />
      <p class="text-xs text-fg-muted mt-2">
        {{ t('settings.log_path_hint') }}
        <code class="font-mono">%APPDATA%\D-Medic\logs\</code>
      </p>
    </BaseCard>
  </div>
</template>
