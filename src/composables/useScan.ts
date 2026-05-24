import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useScanStore } from '@/stores/scan';
import { usePlanStore } from '@/stores/plan';
import { useToast } from './useToast';
import { formatError } from './useInvoke';
import type { ScanKind, ScanResult } from '@/types';

interface ScanProgressPayload {
  scan_id: string;
  kind: 'quick' | 'deep';
  index: number;
  total: number;
  check_id: string;
  status: 'started' | 'finished';
  success?: boolean | null;
  finding_count?: number | null;
}

export function useScan() {
  const store = useScanStore();
  const plan = usePlanStore();
  const toast = useToast();
  const { t } = useI18n();

  async function run(kind: ScanKind): Promise<ScanResult | null> {
    store.beginScan(kind);

    // Yeni tarama → eski seçim orphan kalır (finding UUID'leri değişir).
    plan.clear();

    // Backend her check öncesi/sonrası emit("scan-progress", ...) atıyor;
    // tarama bittiğinde unlisten ediyoruz.
    let unlisten: UnlistenFn | null = null;
    try {
      unlisten = await listen<ScanProgressPayload>('scan-progress', (e) => {
        store.handleProgressEvent(e.payload);
      });
    } catch {
      // listen kurulamazsa progress kaybolur ama scan yine de çalışır.
    }

    try {
      const cmd = kind === 'quick' ? 'quick_scan' : 'deep_scan';
      const result = await invoke<ScanResult>(cmd);
      store.completeScan(result);
      return result;
    } catch (e) {
      const msg = formatError(e);
      store.failScan(msg);
      toast.error(t('scan.fail_title'), msg);
      return null;
    } finally {
      if (unlisten) unlisten();
    }
  }

  return { run };
}
