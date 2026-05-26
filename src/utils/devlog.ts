import { invoke } from '@tauri-apps/api/core';
import type { App } from 'vue';

// Frontend → dev terminali log köprüsü. SADECE dev modda kurulur. Yakalanmamış
// hataları (window.onerror, unhandledrejection, Vue errorHandler) ve
// console.error/warn çağrılarını backend `dev_log` komutuna iletir → Rust
// tracing console layer'ında `frontend` target'ı ile dev terminalinde görünür.

let installed = false;
// Köprünün kendi içinde döngüye girmesini engelle (invoke hatası → console.error).
let inSend = false;

function send(level: 'error' | 'warn' | 'info', message: string): void {
  if (inSend) return;
  inSend = true;
  invoke('dev_log', { level, message })
    .catch(() => {})
    .finally(() => {
      inSend = false;
    });
}

function fmt(args: unknown[]): string {
  return args
    .map((a) => {
      if (typeof a === 'string') return a;
      if (a instanceof Error) return a.stack ?? `${a.name}: ${a.message}`;
      try {
        return JSON.stringify(a);
      } catch {
        return String(a);
      }
    })
    .join(' ');
}

export function installDevLogger(app: App): void {
  if (installed || !import.meta.env.DEV) return;
  installed = true;

  const origError = console.error.bind(console);
  const origWarn = console.warn.bind(console);

  console.error = (...args: unknown[]) => {
    send('error', fmt(args));
    origError(...args);
  };
  console.warn = (...args: unknown[]) => {
    send('warn', fmt(args));
    origWarn(...args);
  };

  window.addEventListener('error', (e) => {
    send('error', `window.onerror: ${e.message} @ ${e.filename}:${e.lineno}:${e.colno}`);
  });
  window.addEventListener('unhandledrejection', (e) => {
    const reason = e.reason instanceof Error ? (e.reason.stack ?? e.reason.message) : String(e.reason);
    send('error', `unhandledrejection: ${reason}`);
  });

  const prevHandler = app.config.errorHandler;
  app.config.errorHandler = (err, instance, info) => {
    const msg = err instanceof Error ? (err.stack ?? err.message) : String(err);
    send('error', `[vue] ${msg} (${info})`);
    origError('[vue]', err, info);
    prevHandler?.(err, instance, info);
  };

  send('info', 'Dev logger köprüsü kuruldu — frontend hataları dev terminaline akıyor.');
}
