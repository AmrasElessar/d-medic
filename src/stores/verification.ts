import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { VerificationRecord } from '@/types';

interface VerificationState {
  records: Record<string, VerificationRecord>;
  loaded: boolean;
}

/**
 * Verification kayıtları tek seferde yüklenir, sonra in-memory. Backend
 * include_str! ile gömdüğü için disk I/O yok, sadece IPC bir kez çalışır.
 */
export const useVerificationStore = defineStore('verification', {
  state: (): VerificationState => ({
    records: {},
    loaded: false,
  }),
  getters: {
    byId: (s) => (id: string): VerificationRecord | undefined => s.records[id],
    has: (s) => (id: string): boolean => id in s.records,
  },
  actions: {
    async loadOnce(): Promise<void> {
      if (this.loaded) return;
      try {
        const data = await invoke<Record<string, VerificationRecord>>('list_verifications');
        this.records = data;
      } catch (e) {
        // Sessiz fail — UI rozet göstermez, kötü değil.
        // eslint-disable-next-line no-console
        console.warn('list_verifications failed:', e);
      } finally {
        this.loaded = true;
      }
    },
  },
});
