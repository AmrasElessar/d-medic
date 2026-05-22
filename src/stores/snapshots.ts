import { defineStore } from 'pinia';
import type { Snapshot } from '@/types';

interface SnapshotsState {
  items: Snapshot[];
  loading: boolean;
  error: string | null;
}

export const useSnapshotsStore = defineStore('snapshots', {
  state: (): SnapshotsState => ({
    items: [],
    loading: false,
    error: null,
  }),
  getters: {
    byId: (s) => (id: string): Snapshot | undefined =>
      s.items.find((x) => x.id === id),
  },
  actions: {
    setLoading(value: boolean): void {
      this.loading = value;
    },
    replaceAll(items: Snapshot[]): void {
      this.items = items;
    },
    add(snapshot: Snapshot): void {
      this.items.unshift(snapshot);
    },
    remove(id: string): void {
      this.items = this.items.filter((s) => s.id !== id);
    },
    setError(error: string | null): void {
      this.error = error;
    },
  },
});
