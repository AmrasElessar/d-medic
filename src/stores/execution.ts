import { defineStore } from 'pinia';
import type { ExecutionPlan, PlanItem, PlanItemStatus } from '@/types';

type ExecStatus = 'idle' | 'preparing' | 'running' | 'completed' | 'failed';

interface ExecutionState {
  status: ExecStatus;
  plan: ExecutionPlan | null;
  currentIndex: number;
  log: string[];
  error: string | null;
}

export const useExecutionStore = defineStore('execution', {
  state: (): ExecutionState => ({
    status: 'idle',
    plan: null,
    currentIndex: -1,
    log: [],
    error: null,
  }),
  getters: {
    currentItem(state): PlanItem | null {
      if (!state.plan || state.currentIndex < 0) return null;
      return state.plan.items[state.currentIndex] ?? null;
    },
    progress(state): number {
      if (!state.plan || state.plan.items.length === 0) return 0;
      const done = state.plan.items.filter(
        (i) => i.status === 'success' || i.status === 'failed' || i.status === 'skipped',
      ).length;
      return Math.round((done / state.plan.items.length) * 100);
    },
    failedItems(state): PlanItem[] {
      return state.plan?.items.filter((i) => i.status === 'failed') ?? [];
    },
    successItems(state): PlanItem[] {
      return state.plan?.items.filter((i) => i.status === 'success') ?? [];
    },
  },
  actions: {
    begin(plan: ExecutionPlan): void {
      this.status = 'preparing';
      this.plan = plan;
      this.currentIndex = -1;
      this.log = [];
      this.error = null;
    },
    advance(index: number): void {
      this.status = 'running';
      this.currentIndex = index;
    },
    setItemStatus(findingId: string, status: PlanItemStatus, message?: string): void {
      if (!this.plan) return;
      const item = this.plan.items.find((i) => i.finding_id === findingId);
      if (!item) return;
      item.status = status;
      if (message) item.message = message;
      if (status === 'running') item.started_at = new Date().toISOString();
      if (status === 'success' || status === 'failed' || status === 'skipped') {
        item.finished_at = new Date().toISOString();
      }
    },
    appendLog(line: string): void {
      this.log.push(`[${new Date().toLocaleTimeString()}] ${line}`);
      if (this.log.length > 500) this.log.shift();
    },
    complete(): void {
      this.status = this.failedItems.length > 0 ? 'failed' : 'completed';
    },
    fail(error: string): void {
      this.status = 'failed';
      this.error = error;
    },
    reset(): void {
      this.status = 'idle';
      this.plan = null;
      this.currentIndex = -1;
      this.log = [];
      this.error = null;
    },
  },
});
