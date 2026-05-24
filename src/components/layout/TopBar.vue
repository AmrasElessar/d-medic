<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { Sun, Moon, ShieldCheck, ShieldAlert, Languages } from 'lucide-vue-next';
import { useNavStore } from '@/stores/nav';
import { useSettingsStore } from '@/stores/settings';
import { useSystemStore } from '@/stores/system';
import { useTheme } from '@/composables/useTheme';
import { useToast } from '@/composables/useToast';
import { formatError } from '@/composables/useInvoke';
import { NAV_ENTRIES } from '@/constants/nav';
import BaseTooltip from '../common/BaseTooltip.vue';

const nav = useNavStore();
const settings = useSettingsStore();
const sys = useSystemStore();
const toast = useToast();
const { t, locale } = useI18n();
const { isDark, toggle } = useTheme();

const currentLabel = computed(() => {
  const entry = NAV_ENTRIES.find((n) => n.key === nav.current);
  return entry ? t(entry.labelKey) : '';
});

function toggleLocale() {
  const next = settings.locale === 'tr' ? 'en' : 'tr';
  settings.locale = next;
  locale.value = next;
  settings.persist();
}

async function relaunchAdmin() {
  if (sys.isElevated) return;
  if (!confirm(t('topbar.relaunch_confirm'))) return;
  try {
    await invoke('relaunch_as_admin');
    // Backend 500ms sonra kendi process'ini kapatacak; UI bunu beklemeden
    // kullanıcıya "yeniden başlatılıyor" mesajı gösterir.
    toast.info(t('topbar.relaunch_title'), t('topbar.relaunch_desc'));
  } catch (e) {
    toast.error(t('topbar.relaunch_fail'), formatError(e));
  }
}
</script>

<template>
  <header
    class="h-12 shrink-0 border-b border-border bg-bg-elevated flex items-center justify-between px-4"
  >
    <div class="flex items-center gap-2 text-sm">
      <span class="text-fg-muted">D-Medic</span>
      <span class="text-fg-subtle">/</span>
      <span class="text-fg font-medium">{{ currentLabel }}</span>
    </div>

    <div class="flex items-center gap-2">
      <!-- Admin rozeti — USER iken tıklanabilir buton (yönetici olarak yeniden başlat) -->
      <BaseTooltip
        side="bottom"
        :content="sys.isElevated ? t('topbar.elevated') : t('topbar.relaunch_hint')"
      >
        <button
          type="button"
          :disabled="sys.isElevated"
          class="inline-flex items-center gap-1.5 px-2 py-1 rounded text-xs transition-colors"
          :class="
            sys.isElevated
              ? 'bg-priority-low/15 text-priority-low cursor-default'
              : 'bg-priority-critical/15 text-priority-critical hover:bg-priority-critical/25 cursor-pointer'
          "
          @click="relaunchAdmin"
        >
          <ShieldCheck v-if="sys.isElevated" class="w-3.5 h-3.5" />
          <ShieldAlert v-else class="w-3.5 h-3.5" />
          {{ sys.isElevated ? 'ADMIN' : 'USER' }}
        </button>
      </BaseTooltip>

      <BaseTooltip side="bottom" :content="t('topbar.toggle_language')">
        <button
          class="p-1.5 rounded text-fg-muted hover:text-fg hover:bg-bg-subtle"
          @click="toggleLocale"
        >
          <Languages class="w-4 h-4" />
        </button>
      </BaseTooltip>

      <BaseTooltip side="bottom" :content="t('topbar.toggle_theme')">
        <button
          class="p-1.5 rounded text-fg-muted hover:text-fg hover:bg-bg-subtle"
          @click="toggle"
        >
          <Sun v-if="isDark" class="w-4 h-4" />
          <Moon v-else class="w-4 h-4" />
        </button>
      </BaseTooltip>
    </div>
  </header>
</template>
