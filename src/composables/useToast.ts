import { reactive, readonly } from 'vue';

export type ToastKind = 'info' | 'success' | 'warning' | 'error';

export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  body?: string;
  timeout: number;
}

interface ToastState {
  items: Toast[];
}

const state = reactive<ToastState>({ items: [] });
let nextId = 1;

function push(kind: ToastKind, title: string, body?: string, timeout = 4000): number {
  const toast: Toast = { id: nextId++, kind, title, body, timeout };
  state.items.push(toast);
  if (timeout > 0) {
    setTimeout(() => dismiss(toast.id), timeout);
  }
  return toast.id;
}

function dismiss(id: number): void {
  const idx = state.items.findIndex((t) => t.id === id);
  if (idx !== -1) state.items.splice(idx, 1);
}

export function useToast() {
  return {
    state: readonly(state),
    info: (title: string, body?: string) => push('info', title, body),
    success: (title: string, body?: string) => push('success', title, body),
    warning: (title: string, body?: string) => push('warning', title, body, 6000),
    error: (title: string, body?: string) => push('error', title, body, 8000),
    dismiss,
  };
}
