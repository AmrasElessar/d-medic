import { defineStore } from 'pinia';
import type { Finding, ProfileKind } from '@/types';

interface PlanState {
  profile: ProfileKind;
  selectedFindingIds: string[];
  rebootAccepted: boolean;
}

export const usePlanStore = defineStore('plan', {
  state: (): PlanState => ({
    profile: 'moderate',
    selectedFindingIds: [],
    rebootAccepted: false,
  }),
  getters: {
    isSelected: (s) => (id: string): boolean => s.selectedFindingIds.includes(id),
    count: (s): number => s.selectedFindingIds.length,
  },
  actions: {
    setProfile(profile: ProfileKind): void {
      this.profile = profile;
    },
    toggle(id: string): void {
      const idx = this.selectedFindingIds.indexOf(id);
      if (idx === -1) this.selectedFindingIds.push(id);
      else this.selectedFindingIds.splice(idx, 1);
    },
    selectAll(findings: Finding[]): void {
      this.selectedFindingIds = findings
        .filter((f) => f.action_type === 'automatic' || f.action_type === 'reboot')
        .map((f) => f.id);
    },
    clear(): void {
      this.selectedFindingIds = [];
    },
    acceptReboot(value: boolean): void {
      this.rebootAccepted = value;
    },
  },
});
