import { invoke } from '@tauri-apps/api/core';
import { useGuidesStore } from '@/stores/guides';
import { useToast } from './useToast';
import { formatError } from './useInvoke';
import type { Guide } from '@/types';

export function useGuide() {
  const store = useGuidesStore();
  const toast = useToast();

  async function loadAll(): Promise<Guide[]> {
    try {
      const list = await invoke<Guide[]>('list_guides');
      store.replaceAll(list);
      return list;
    } catch (e) {
      toast.error('Kılavuzlar yüklenemedi', formatError(e));
      return [];
    }
  }

  async function loadOne(id: string): Promise<Guide | null> {
    try {
      const guide = await invoke<Guide>('get_guide', { id });
      store.upsert(guide);
      return guide;
    } catch (e) {
      toast.error('Kılavuz açılamadı', formatError(e));
      return null;
    }
  }

  return { loadAll, loadOne };
}
