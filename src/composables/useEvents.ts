import { listen, type UnlistenFn, type EventCallback } from '@tauri-apps/api/event';
import { onBeforeUnmount, onMounted } from 'vue';

/**
 * Tauri event'ine component lifecycle'a bağlı dinleyici kur.
 * Mount'ta dinler, unmount'ta otomatik temizler.
 */
export function useEvent<T>(event: string, handler: EventCallback<T>): void {
  let unlisten: UnlistenFn | null = null;

  onMounted(async () => {
    unlisten = await listen<T>(event, handler);
  });

  onBeforeUnmount(() => {
    unlisten?.();
  });
}
