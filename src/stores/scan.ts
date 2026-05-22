import { defineStore } from 'pinia';
import type { Finding, ScanKind, ScanResult } from '@/types';

type ScanStatus = 'idle' | 'running' | 'completed' | 'failed';

interface ScanState {
  status: ScanStatus;
  kind: ScanKind | null;
  lastResult: ScanResult | null;
  history: ScanResult[];
  error: string | null;
  progress: number;
}

export const useScanStore = defineStore('scan', {
  state: (): ScanState => ({
    status: 'idle',
    kind: null,
    lastResult: null,
    history: [],
    error: null,
    progress: 0,
  }),
  getters: {
    findings: (s): Finding[] => s.lastResult?.findings ?? [],
    findingsByPriority(): Record<string, Finding[]> {
      const map: Record<string, Finding[]> = {
        critical: [],
        high: [],
        medium: [],
        low: [],
      };
      for (const f of this.findings) {
        const bucket = map[f.priority];
        if (bucket) bucket.push(f);
      }
      return map;
    },
    findingsByActionType(): Record<string, Finding[]> {
      const map: Record<string, Finding[]> = {
        automatic: [],
        guided: [],
        reboot: [],
        not_possible: [],
      };
      for (const f of this.findings) {
        const bucket = map[f.action_type];
        if (bucket) bucket.push(f);
      }
      return map;
    },
    totalCount(): number {
      return this.findings.length;
    },
  },
  actions: {
    beginScan(kind: ScanKind): void {
      this.status = 'running';
      this.kind = kind;
      this.error = null;
      this.progress = 0;
    },
    setProgress(pct: number): void {
      this.progress = Math.max(0, Math.min(100, pct));
    },
    completeScan(result: ScanResult): void {
      this.status = 'completed';
      this.lastResult = result;
      this.history.unshift(result);
      this.history = this.history.slice(0, 20);
      this.progress = 100;
    },
    failScan(error: string): void {
      this.status = 'failed';
      this.error = error;
    },
    reset(): void {
      this.status = 'idle';
      this.kind = null;
      this.error = null;
      this.progress = 0;
    },
  },
});
