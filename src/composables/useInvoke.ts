import { invoke as tauriInvoke, type InvokeArgs } from '@tauri-apps/api/core';
import { ref, type Ref } from 'vue';

export interface InvokeState<T> {
  data: Ref<T | null>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  call: (args?: InvokeArgs) => Promise<T>;
}

/**
 * Tauri komut çağrısını reaktif state ile sarar.
 * Hata gövdesi { code, message } şeklinde dönerse `code: message` formatlanır.
 */
export function useInvoke<T>(command: string): InvokeState<T> {
  const data = ref<T | null>(null) as Ref<T | null>;
  const loading = ref(false);
  const error = ref<string | null>(null);

  const call = async (args?: InvokeArgs): Promise<T> => {
    loading.value = true;
    error.value = null;
    try {
      const result = await tauriInvoke<T>(command, args);
      data.value = result;
      return result;
    } catch (e: unknown) {
      const formatted = formatError(e);
      error.value = formatted;
      throw new Error(formatted);
    } finally {
      loading.value = false;
    }
  };

  return { data, loading, error, call };
}

export function formatError(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e && typeof e === 'object') {
    const obj = e as { code?: string; message?: string };
    if (obj.code && obj.message) return `${obj.code}: ${obj.message}`;
    if (obj.message) return obj.message;
  }
  return String(e);
}
