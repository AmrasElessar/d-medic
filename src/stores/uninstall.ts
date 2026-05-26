import { defineStore } from 'pinia';
import type { InstalledProgram, QuarantineEntry } from '@/types';

interface UninstallState {
  programs: InstalledProgram[];
  loaded: boolean;
  quarantine: QuarantineEntry[];
}

export const useUninstallStore = defineStore('uninstall', {
  state: (): UninstallState => ({
    programs: [],
    loaded: false,
    quarantine: [],
  }),
  getters: {
    /** UWP olmayan + sistem bileşeni olmayan toplam (UI özeti). */
    userProgramCount: (s): number =>
      s.programs.filter((p) => !p.is_system_component).length,
  },
  actions: {
    setPrograms(list: InstalledProgram[]): void {
      this.programs = list;
      this.loaded = true;
    },
    removeProgram(id: string): void {
      this.programs = this.programs.filter((p) => p.id !== id);
    },
    setQuarantine(list: QuarantineEntry[]): void {
      this.quarantine = list;
    },
  },
});
