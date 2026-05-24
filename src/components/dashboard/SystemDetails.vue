<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Cpu, MemoryStick, HardDrive, ShieldCheck, AppWindow, Package, Server } from 'lucide-vue-next';
import { useSystemStore } from '@/stores/system';
import BaseCard from '@/components/common/BaseCard.vue';

const sys = useSystemStore();
const { t } = useI18n();

const ramDetail = computed(() => {
  const s = sys.stats;
  if (!s) return '—';
  const used = Math.max(0, s.total_ram_gb - s.free_ram_gb);
  const pct = s.total_ram_gb > 0 ? Math.round((used / s.total_ram_gb) * 100) : 0;
  return `${used.toFixed(1)} / ${s.total_ram_gb.toFixed(0)} GB · ${pct}%`;
});

const diskDetail = computed(() => {
  const s = sys.stats;
  if (!s || s.primary_disk_size_gb === 0) return '—';
  const used = s.primary_disk_size_gb - s.primary_disk_free_gb;
  return `${used.toFixed(0)} / ${s.primary_disk_size_gb.toFixed(0)} GB · ${s.primary_disk_type || '?'}`;
});

const efiDetail = computed(() => {
  const s = sys.stats;
  if (!s || s.efi_size_mb === 0) return t('view.dashboard.efi_legacy');
  return `${s.efi_size_mb} MB`;
});

const vbsDetail = computed(() => {
  const s = sys.stats;
  if (!s) return '—';
  return s.vbs_running ? t('view.dashboard.vbs_on') : t('view.dashboard.vbs_off');
});
</script>

<template>
  <BaseCard :title="t('view.dashboard.sys_details')">
    <dl class="text-sm grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-2.5">
      <div class="flex items-center gap-2">
        <Cpu class="w-4 h-4 text-fg-muted shrink-0" />
        <dt class="text-fg-muted shrink-0">CPU</dt>
        <dd class="text-fg truncate ml-auto text-right" :title="sys.cpuText">{{ sys.cpuText }}</dd>
      </div>
      <div class="flex items-center gap-2">
        <MemoryStick class="w-4 h-4 text-fg-muted shrink-0" />
        <dt class="text-fg-muted shrink-0">RAM</dt>
        <dd class="text-fg ml-auto">{{ ramDetail }}</dd>
      </div>
      <div class="flex items-center gap-2">
        <HardDrive class="w-4 h-4 text-fg-muted shrink-0" />
        <dt class="text-fg-muted shrink-0">{{ t('view.dashboard.primary_disk') }}</dt>
        <dd class="text-fg ml-auto">{{ diskDetail }}</dd>
      </div>
      <div class="flex items-center gap-2">
        <Server class="w-4 h-4 text-fg-muted shrink-0" />
        <dt class="text-fg-muted shrink-0">{{ t('view.dashboard.efi_partition') }}</dt>
        <dd class="text-fg ml-auto">{{ efiDetail }}</dd>
      </div>
      <div class="flex items-center gap-2">
        <ShieldCheck class="w-4 h-4 text-fg-muted shrink-0" />
        <dt class="text-fg-muted shrink-0">VBS / HVCI</dt>
        <dd class="text-fg ml-auto">{{ vbsDetail }}</dd>
      </div>
      <div class="flex items-center gap-2">
        <Package class="w-4 h-4 text-fg-muted shrink-0" />
        <dt class="text-fg-muted shrink-0">{{ t('view.dashboard.uwp_packages') }}</dt>
        <dd class="text-fg ml-auto">{{ sys.stats?.uwp_package_count ?? '—' }}</dd>
      </div>
      <div class="flex items-center gap-2">
        <AppWindow class="w-4 h-4 text-fg-muted shrink-0" />
        <dt class="text-fg-muted shrink-0">{{ t('view.dashboard.installed_apps') }}</dt>
        <dd class="text-fg ml-auto">{{ sys.stats?.installed_app_count ?? '—' }}</dd>
      </div>
    </dl>
  </BaseCard>
</template>
