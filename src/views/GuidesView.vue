<script setup lang="ts">
import { onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useGuidesStore } from '@/stores/guides';
import { useGuide } from '@/composables/useGuide';
import GuideLibrary from '@/components/guide/GuideLibrary.vue';
import GuideViewer from '@/components/guide/GuideViewer.vue';

const store = useGuidesStore();
const { loadAll, loadOne } = useGuide();
const { t } = useI18n();

onMounted(() => {
  if (store.items.length === 0) {
    void loadAll();
  }
});

async function openGuide(id: string) {
  if (!store.byId(id)) await loadOne(id);
  store.open(id);
}
</script>

<template>
  <div class="p-6 space-y-4 max-w-6xl mx-auto">
    <header>
      <h1 class="text-2xl font-bold text-fg">{{ t('view.guides.title') }}</h1>
      <p class="text-sm text-fg-muted mt-1">{{ t('view.guides.subtitle') }}</p>
    </header>

    <GuideLibrary :guides="store.items" @open="openGuide" />
    <GuideViewer />
  </div>
</template>
