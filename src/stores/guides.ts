import { defineStore } from 'pinia';
import type { Guide } from '@/types';

interface GuidesState {
  items: Guide[];
  activeId: string | null;
  completedStepIds: Record<string, number[]>;
}

export const useGuidesStore = defineStore('guides', {
  state: (): GuidesState => ({
    items: [],
    activeId: null,
    completedStepIds: {},
  }),
  getters: {
    active: (s): Guide | null =>
      s.activeId ? s.items.find((g) => g.id === s.activeId) ?? null : null,
    byId: (s) => (id: string): Guide | undefined =>
      s.items.find((g) => g.id === id),
  },
  actions: {
    replaceAll(items: Guide[]): void {
      this.items = items;
    },
    upsert(guide: Guide): void {
      const idx = this.items.findIndex((g) => g.id === guide.id);
      if (idx === -1) this.items.push(guide);
      else this.items[idx] = guide;
    },
    open(id: string): void {
      this.activeId = id;
    },
    close(): void {
      this.activeId = null;
    },
    markStep(guideId: string, stepId: number, done: boolean): void {
      const list = this.completedStepIds[guideId] ?? [];
      if (done && !list.includes(stepId)) list.push(stepId);
      if (!done) {
        const idx = list.indexOf(stepId);
        if (idx !== -1) list.splice(idx, 1);
      }
      this.completedStepIds[guideId] = list;
    },
  },
});
