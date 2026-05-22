import { computed } from 'vue';
import { useSettingsStore } from '@/stores/settings';

export type ThemeKind = 'dark' | 'light' | 'system';

export function useTheme() {
  const settings = useSettingsStore();

  const theme = computed<ThemeKind>(() => settings.theme);

  const isDark = computed(() => {
    if (theme.value === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    return theme.value === 'dark';
  });

  function apply(): void {
    const root = document.documentElement;
    if (isDark.value) root.classList.add('dark');
    else root.classList.remove('dark');
  }

  function set(next: ThemeKind): void {
    settings.theme = next;
    apply();
  }

  function toggle(): void {
    set(isDark.value ? 'light' : 'dark');
  }

  return { theme, isDark, apply, set, toggle };
}
