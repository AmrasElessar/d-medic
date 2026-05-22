import { defineStore } from 'pinia';

interface AppInfo {
  name: string;
  version: string;
  os: string;
  elevated: boolean;
}

interface SystemState {
  info: AppInfo | null;
  online: boolean;
}

export const useSystemStore = defineStore('system', {
  state: (): SystemState => ({
    info: null,
    online: navigator.onLine,
  }),
  getters: {
    isElevated: (s): boolean => s.info?.elevated ?? false,
    version: (s): string => s.info?.version ?? '0.0.0',
  },
  actions: {
    setInfo(info: AppInfo): void {
      this.info = info;
    },
    setOnline(online: boolean): void {
      this.online = online;
    },
  },
});
