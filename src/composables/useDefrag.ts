import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useDefragStore } from '@/stores/defrag';
import { useToast } from './useToast';
import { formatError } from './useInvoke';
import type {
  ClusterMap,
  DefragMode,
  DefragProgress,
  FragmentationReport,
  VolumeInfo,
} from '@/types';

/** İstenen yaklaşık hücre sayısı (cluster haritası çözünürlüğü). */
const MAP_RESOLUTION = 64 * 32;

export function useDefrag() {
  const store = useDefragStore();
  const toast = useToast();
  const { t } = useI18n();

  async function loadVolumes(): Promise<void> {
    try {
      const list = await invoke<VolumeInfo[]>('list_volumes');
      store.setVolumes(list);
    } catch (e) {
      toast.error(t('defrag.volumes_fail'), formatError(e));
    }
  }

  async function loadMap(letter: string): Promise<void> {
    try {
      const map = await invoke<ClusterMap>('get_cluster_map', {
        letter,
        resolution: MAP_RESOLUTION,
      });
      store.setMap(map);
    } catch {
      // harita best-effort
    }
  }

  async function analyze(letter: string): Promise<void> {
    let unlisten: UnlistenFn | null = null;
    store.setStatus('analyzing');
    store.setProgress({
      job_id: 'analyze',
      phase: 'analyzing',
      current_file: null,
      clusters_moved: 0,
      clusters_total: 0,
      files_processed: 0,
      percent: 0,
    });
    try {
      unlisten = await listen<DefragProgress>('defrag-progress', (e) => {
        if (e.payload.phase === 'analyzing') store.setProgress(e.payload);
      });
      const report = await invoke<FragmentationReport>('analyze_volume', { letter });
      store.setReport(report);
      await loadMap(letter);
      store.setStatus('idle');
    } catch (e) {
      store.setStatus('error');
      toast.error(t('defrag.analyze_fail'), formatError(e));
    } finally {
      unlisten?.();
    }
  }

  async function startDefrag(letter: string, mode: DefragMode): Promise<void> {
    let unlisten: UnlistenFn | null = null;
    store.setStatus('defragging');
    try {
      unlisten = await listen<DefragProgress>('defrag-progress', (e) => {
        store.setProgress(e.payload);
        // Haritayı arada bir tazele (her ~25 dosyada).
        if (e.payload.files_processed > 0 && e.payload.files_processed % 25 === 0) {
          void loadMap(letter);
        }
      });
      await invoke('start_defrag', { letter, mode });
      await loadMap(letter);
      // Defrag sonrası raporu da tazele.
      const report = await invoke<FragmentationReport>('analyze_volume', { letter });
      store.setReport(report);
    } catch (e) {
      store.setStatus('error');
      toast.error(t('defrag.defrag_fail'), formatError(e));
    } finally {
      unlisten?.();
    }
  }

  async function cancel(): Promise<void> {
    try {
      await invoke('cancel_defrag');
    } catch {
      // best-effort
    }
  }

  return { loadVolumes, loadMap, analyze, startDefrag, cancel };
}
