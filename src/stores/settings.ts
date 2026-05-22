import { defineStore } from 'pinia';
import type { ThemeKind } from '@/composables/useTheme';
import type { SupportedLocale } from '@/i18n';

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface SettingsState {
  theme: ThemeKind;
  locale: SupportedLocale;
  logLevel: LogLevel;
  autoScanOnStart: boolean;
  confirmBeforeExecute: boolean;
  createRestorePoint: boolean;
  exportRegistry: boolean;
}

const STORAGE_KEY = 'd-medic.settings.v1';

function load(): Partial<SettingsState> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as Partial<SettingsState>) : {};
  } catch {
    return {};
  }
}

export const useSettingsStore = defineStore('settings', {
  state: (): SettingsState => {
    const stored = load();
    return {
      theme: stored.theme ?? 'dark',
      locale: stored.locale ?? 'tr',
      logLevel: stored.logLevel ?? 'info',
      autoScanOnStart: stored.autoScanOnStart ?? true,
      confirmBeforeExecute: stored.confirmBeforeExecute ?? true,
      createRestorePoint: stored.createRestorePoint ?? true,
      exportRegistry: stored.exportRegistry ?? true,
    };
  },
  actions: {
    persist(): void {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.$state));
      } catch {
        // sessizce yok say — quota dolmuş olabilir
      }
    },
  },
});
