import { defineStore } from 'pinia';
import type {
  ClusterMap,
  DefragProgress,
  FragmentationReport,
  VolumeInfo,
} from '@/types';

type DefragStatus = 'idle' | 'analyzing' | 'defragging' | 'done' | 'error';

interface DefragState {
  volumes: VolumeInfo[];
  selected: string | null;
  report: FragmentationReport | null;
  map: ClusterMap | null;
  progress: DefragProgress | null;
  status: DefragStatus;
}

export const useDefragStore = defineStore('defrag', {
  state: (): DefragState => ({
    volumes: [],
    selected: null,
    report: null,
    map: null,
    progress: null,
    status: 'idle',
  }),
  getters: {
    selectedVolume(): VolumeInfo | null {
      return this.volumes.find((v) => v.letter === this.selected) ?? null;
    },
  },
  actions: {
    setVolumes(list: VolumeInfo[]): void {
      this.volumes = list;
      if (!this.selected && list.length) {
        // Varsayılan: sistem diski (C) varsa onu, yoksa ilkini seç.
        this.selected = (list.find((v) => v.letter === 'C') ?? list[0]!).letter;
      }
    },
    select(letter: string): void {
      this.selected = letter;
      this.report = null;
      this.map = null;
      this.progress = null;
      this.status = 'idle';
    },
    setReport(r: FragmentationReport): void {
      this.report = r;
    },
    setMap(m: ClusterMap): void {
      this.map = m;
    },
    setProgress(p: DefragProgress): void {
      this.progress = p;
      if (p.phase === 'done') this.status = 'done';
      else if (p.phase === 'error') this.status = 'error';
      else if (p.phase === 'cancelled') this.status = 'idle';
    },
    setStatus(s: DefragStatus): void {
      this.status = s;
    },
  },
});
