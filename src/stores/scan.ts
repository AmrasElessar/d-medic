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
  /** Backend `scan-progress` event'lerinden gelen anlık adım sayısı. */
  currentIndex: number;
  totalChecks: number;
  /** Şu an çalışan / az önce biten check kimliği — UI alt yazısında gösterilir. */
  currentCheckId: string | null;
  lastFinishedCheckId: string | null;
}

export const useScanStore = defineStore('scan', {
  state: (): ScanState => ({
    status: 'idle',
    kind: null,
    lastResult: null,
    history: [],
    error: null,
    progress: 0,
    currentIndex: 0,
    totalChecks: 0,
    currentCheckId: null,
    lastFinishedCheckId: null,
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
      this.currentIndex = 0;
      this.totalChecks = 0;
      this.currentCheckId = null;
      this.lastFinishedCheckId = null;
    },
    setProgress(pct: number): void {
      this.progress = Math.max(0, Math.min(100, pct));
    },
    /** Backend `scan-progress` event'inden gelen tick. */
    handleProgressEvent(payload: {
      index: number;
      total: number;
      check_id: string;
      status: 'started' | 'finished';
    }): void {
      this.totalChecks = payload.total;
      if (payload.status === 'started') {
        this.currentIndex = payload.index;
        this.currentCheckId = payload.check_id;
      } else {
        this.lastFinishedCheckId = payload.check_id;
        // index+1 biten check sayısı; total > 0 garanti.
        const done = payload.index + 1;
        this.progress = Math.round((done / Math.max(1, payload.total)) * 100);
      }
    },
    completeScan(result: ScanResult): void {
      this.status = 'completed';
      this.lastResult = result;
      this.history.unshift(result);
      this.history = this.history.slice(0, 20);
      this.progress = 100;
      this.currentCheckId = null;
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
      this.currentIndex = 0;
      this.totalChecks = 0;
      this.currentCheckId = null;
      this.lastFinishedCheckId = null;
    },
  },
});
