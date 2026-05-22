import { invoke } from '@tauri-apps/api/core';
import { useScanStore } from '@/stores/scan';
import { useToast } from './useToast';
import { formatError } from './useInvoke';
import type { ScanKind, ScanResult } from '@/types';

export function useScan() {
  const store = useScanStore();
  const toast = useToast();

  async function run(kind: ScanKind): Promise<ScanResult | null> {
    store.beginScan(kind);
    try {
      const cmd = kind === 'quick' ? 'quick_scan' : 'deep_scan';
      const result = await invoke<ScanResult>(cmd);
      store.completeScan(result);
      return result;
    } catch (e) {
      const msg = formatError(e);
      store.failScan(msg);
      toast.error('Tarama başarısız', msg);
      return null;
    }
  }

  return { run };
}
