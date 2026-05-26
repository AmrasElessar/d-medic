import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { useUninstallStore } from '@/stores/uninstall';
import { useToast } from './useToast';
import { formatError } from './useInvoke';
import type {
  InstalledProgram,
  LeftoverItem,
  LeftoverScanResult,
  QuarantineEntry,
  RemovalReport,
  UninstallReport,
} from '@/types';

export function useUninstall() {
  const store = useUninstallStore();
  const toast = useToast();
  const { t } = useI18n();

  async function loadPrograms(force = false): Promise<void> {
    if (store.loaded && !force) return;
    try {
      const list = await invoke<InstalledProgram[]>('list_installed_programs');
      store.setPrograms(list);
    } catch (e) {
      toast.error(t('uninstall.load_fail'), formatError(e));
    }
  }

  async function runUninstaller(program: InstalledProgram): Promise<UninstallReport | null> {
    try {
      return await invoke<UninstallReport>('run_uninstaller', { program });
    } catch (e) {
      toast.error(t('uninstall.uninstall_fail'), formatError(e));
      return null;
    }
  }

  async function scanLeftovers(program: InstalledProgram): Promise<LeftoverScanResult | null> {
    try {
      return await invoke<LeftoverScanResult>('scan_leftovers', { program });
    } catch (e) {
      toast.error(t('uninstall.scan_fail'), formatError(e));
      return null;
    }
  }

  async function removeLeftovers(
    programLabel: string,
    items: LeftoverItem[],
  ): Promise<RemovalReport | null> {
    try {
      const report = await invoke<RemovalReport>('remove_leftovers', {
        programLabel,
        items,
      });
      void loadQuarantine();
      return report;
    } catch (e) {
      toast.error(t('uninstall.remove_fail'), formatError(e));
      return null;
    }
  }

  async function loadQuarantine(): Promise<void> {
    try {
      const list = await invoke<QuarantineEntry[]>('list_quarantine');
      store.setQuarantine(list);
    } catch {
      // sessiz — karantina paneli boş kalır
    }
  }

  async function restoreQuarantine(id: string): Promise<void> {
    try {
      await invoke('restore_quarantine', { id });
      toast.success(t('uninstall.restore_ok_title'), t('uninstall.restore_ok_desc'));
      void loadQuarantine();
    } catch (e) {
      toast.error(t('uninstall.restore_fail'), formatError(e));
    }
  }

  async function purgeQuarantine(id: string): Promise<void> {
    try {
      await invoke('purge_quarantine', { id });
      void loadQuarantine();
    } catch (e) {
      toast.error(t('uninstall.purge_fail'), formatError(e));
    }
  }

  return {
    loadPrograms,
    runUninstaller,
    scanLeftovers,
    removeLeftovers,
    loadQuarantine,
    restoreQuarantine,
    purgeQuarantine,
  };
}
