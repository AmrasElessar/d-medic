<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Sun, Moon, ShieldCheck, ShieldAlert, Languages } from 'lucide-vue-next';
import { useNavStore } from '@/stores/nav';
import { useSettingsStore } from '@/stores/settings';
import { useSystemStore } from '@/stores/system';
import { useTheme } from '@/composables/useTheme';
import { NAV_ENTRIES } from '@/constants/nav';
import BaseTooltip from '../common/BaseTooltip.vue';

const nav = useNavStore();
const settings = useSettingsStore();
const sys = useSystemStore();
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
      <BaseTooltip
        :content="sys.isElevated ? t('topbar.elevated') : t('topbar.not_elevated')"
      >
        <span
          class="inline-flex items-center gap-1.5 px-2 py-1 rounded text-xs"
          :class="
            sys.isElevated
              ? 'bg-priority-low/15 text-priority-low'
              : 'bg-priority-critical/15 text-priority-critical'
          "
        >
          <ShieldCheck v-if="sys.isElevated" class="w-3.5 h-3.5" />
          <ShieldAlert v-else class="w-3.5 h-3.5" />
          {{ sys.isElevated ? 'ADMIN' : 'USER' }}
        </span>
      </BaseTooltip>

      <BaseTooltip :content="t('topbar.toggle_language')">
        <button
          class="p-1.5 rounded text-fg-muted hover:text-fg hover:bg-bg-subtle"
          @click="toggleLocale"
        >
          <Languages class="w-4 h-4" />
        </button>
      </BaseTooltip>

      <BaseTooltip :content="t('topbar.toggle_theme')">
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
