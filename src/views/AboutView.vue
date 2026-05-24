<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Stethoscope, Github, FileText, ExternalLink } from 'lucide-vue-next';
import { useSystemStore } from '@/stores/system';
import BaseCard from '@/components/common/BaseCard.vue';

const sys = useSystemStore();
const { t } = useI18n();
</script>

<template>
  <div class="p-6 space-y-4 max-w-2xl mx-auto">
    <header class="flex items-center gap-4">
      <div class="w-14 h-14 rounded-xl bg-accent/15 grid place-items-center">
        <Stethoscope class="w-8 h-8 text-accent" />
      </div>
      <div>
        <h1 class="text-2xl font-bold text-fg">D-Medic</h1>
        <p class="text-sm text-fg-muted">
          v{{ sys.version }} · {{ sys.info?.git_rev ?? '—' }} ·
          {{ sys.info?.build_date ?? '—' }} · MIT License
        </p>
      </div>
    </header>

    <BaseCard :title="t('view.about.title')">
      <p class="text-sm text-fg leading-relaxed">{{ t('view.about.description') }}</p>
    </BaseCard>

    <BaseCard :title="t('view.about.philosophy')">
      <ul class="text-sm text-fg-muted space-y-2 leading-relaxed">
        <li>• {{ t('view.about.p1') }}</li>
        <li>• {{ t('view.about.p2') }}</li>
        <li>• {{ t('view.about.p3') }}</li>
        <li>• {{ t('view.about.p4') }}</li>
        <li>• {{ t('view.about.p5') }}</li>
      </ul>
    </BaseCard>

    <BaseCard :title="t('view.about.links')">
      <div class="space-y-2 text-sm">
        <a
          href="https://github.com/"
          target="_blank"
          rel="noopener"
          class="flex items-center gap-2 text-accent hover:underline"
        >
          <Github class="w-4 h-4" />
          GitHub Repo
          <ExternalLink class="w-3 h-3" />
        </a>
        <a
          href="#"
          class="flex items-center gap-2 text-accent hover:underline"
        >
          <FileText class="w-4 h-4" />
          {{ t('view.about.docs') }}
          <ExternalLink class="w-3 h-3" />
        </a>
      </div>
    </BaseCard>

    <BaseCard :title="t('view.about.environment')">
      <dl class="text-sm grid grid-cols-2 gap-y-2">
        <dt class="text-fg-muted">OS</dt>
        <dd class="text-fg">{{ sys.info?.os ?? '—' }}</dd>
        <dt class="text-fg-muted">{{ t('view.about.privilege') }}</dt>
        <dd class="text-fg">
          {{ sys.isElevated ? t('view.about.administrator') : t('view.about.standard_user') }}
        </dd>
        <dt class="text-fg-muted">Stack</dt>
        <dd class="text-fg">Tauri 2 · Vue 3 · Rust</dd>
      </dl>
    </BaseCard>
  </div>
</template>
