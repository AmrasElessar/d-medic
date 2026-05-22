<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { useNavStore } from '@/stores/nav';
import { useSettingsStore } from '@/stores/settings';
import { useSystemStore } from '@/stores/system';
import { useTheme } from '@/composables/useTheme';
import { useToast } from '@/composables/useToast';
import { formatError } from '@/composables/useInvoke';

import SideNav from './SideNav.vue';
import TopBar from './TopBar.vue';
import StatusBar from './StatusBar.vue';
import ToastHost from './ToastHost.vue';

import DashboardView from '@/views/DashboardView.vue';
import ScanView from '@/views/ScanView.vue';
import PlanView from '@/views/PlanView.vue';
import ExecuteView from '@/views/ExecuteView.vue';
import GuidesView from '@/views/GuidesView.vue';
import HistoryView from '@/views/HistoryView.vue';
import SettingsView from '@/views/SettingsView.vue';
import AboutView from '@/views/AboutView.vue';

const nav = useNavStore();
const settings = useSettingsStore();
const sys = useSystemStore();
const toast = useToast();
const theme = useTheme();
const { locale } = useI18n();

const views = {
  dashboard: DashboardView,
  scan: ScanView,
  plan: PlanView,
  execute: ExecuteView,
  guides: GuidesView,
  history: HistoryView,
  settings: SettingsView,
  about: AboutView,
};

const currentView = computed(() => views[nav.current]);

onMounted(async () => {
  theme.apply();
  locale.value = settings.locale;

  try {
    const info = await invoke<{
      name: string;
      version: string;
      os: string;
      elevated: boolean;
    }>('app_info');
    sys.setInfo(info);
    if (!info.elevated) {
      toast.warning('Yetki Uyarısı', 'D-Medic admin yetkisi olmadan çalışıyor. Bazı işlemler başarısız olabilir.');
    }
  } catch (e) {
    toast.error('Sistem bilgisi alınamadı', formatError(e));
  }

  window.addEventListener('online', () => sys.setOnline(true));
  window.addEventListener('offline', () => sys.setOnline(false));
});
</script>

<template>
  <div class="h-screen flex flex-col bg-bg text-fg">
    <div class="flex flex-1 overflow-hidden">
      <SideNav />
      <div class="flex-1 flex flex-col overflow-hidden">
        <TopBar />
        <main class="flex-1 overflow-y-auto">
          <component :is="currentView" />
        </main>
      </div>
    </div>
    <StatusBar />
    <ToastHost />
  </div>
</template>
