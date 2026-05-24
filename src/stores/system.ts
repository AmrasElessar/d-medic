import { defineStore } from 'pinia';
import type { SystemStats } from '@/types';

interface AppInfo {
  name: string;
  version: string;
  os: string;
  elevated: boolean;
  git_rev: string;
  build_date: string;
}

interface SystemState {
  info: AppInfo | null;
  stats: SystemStats | null;
  online: boolean;
}

export const useSystemStore = defineStore('system', {
  state: (): SystemState => ({
    info: null,
    stats: null,
    online: navigator.onLine,
  }),
  getters: {
    isElevated: (s): boolean => s.info?.elevated ?? false,
    version: (s): string => s.info?.version ?? '0.0.0',
    ramText: (s): string => {
      if (!s.stats) return '—';
      const total = s.stats.total_ram_gb;
      const used = Math.max(0, total - s.stats.free_ram_gb);
      return `${used.toFixed(1)} / ${total.toFixed(0)} GB`;
    },
    diskText: (s): string => {
      if (!s.stats || s.stats.primary_disk_size_gb === 0) return '—';
      const total = s.stats.primary_disk_size_gb;
      const free = s.stats.primary_disk_free_gb;
      return `${free.toFixed(0)} / ${total.toFixed(0)} GB`;
    },
    cpuText: (s): string => s.stats?.cpu_name || '—',
  },
  actions: {
    setInfo(info: AppInfo): void {
      this.info = info;
    },
    setStats(stats: SystemStats): void {
      this.stats = stats;
    },
    setOnline(online: boolean): void {
      this.online = online;
    },
  },
});
