<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { File, Folder, KeyRound, ListTree } from 'lucide-vue-next';
import BaseBadge from '@/components/common/BaseBadge.vue';
import { formatBytes } from '@/utils/format';
import type { LeftoverConfidence, LeftoverItem, LeftoverKind } from '@/types';

const props = defineProps<{
  items: LeftoverItem[];
  selected: string[];
}>();

const emit = defineEmits<{ (e: 'update:selected', ids: string[]): void }>();

const { t } = useI18n();

const fileItems = computed(() =>
  props.items.filter((i) => i.kind === 'file' || i.kind === 'folder'),
);
const regItems = computed(() =>
  props.items.filter((i) => i.kind === 'reg_key' || i.kind === 'reg_value'),
);

const selectedSet = computed(() => new Set(props.selected));

function toggle(id: string): void {
  const next = new Set(props.selected);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  emit('update:selected', [...next]);
}

function setAll(ids: string[], on: boolean): void {
  const next = new Set(props.selected);
  for (const id of ids) {
    if (on) next.add(id);
    else next.delete(id);
  }
  emit('update:selected', [...next]);
}

function kindIcon(kind: LeftoverKind) {
  if (kind === 'folder') return Folder;
  if (kind === 'file') return File;
  return KeyRound;
}

function confidenceVariant(c: LeftoverConfidence): 'success' | 'warning' | 'danger' {
  if (c === 'safe') return 'success';
  if (c === 'probable') return 'warning';
  return 'danger';
}
</script>

<template>
  <div class="space-y-4">
    <!-- Dosya/klasör grubu -->
    <section v-if="fileItems.length">
      <div class="flex items-center justify-between mb-1.5">
        <h4 class="text-sm font-semibold text-fg flex items-center gap-2">
          <Folder class="w-4 h-4 text-fg-muted" />
          {{ t('uninstall.group_files', { n: fileItems.length }) }}
        </h4>
        <div class="flex gap-2 text-xs">
          <button class="text-accent hover:underline" @click="setAll(fileItems.map((i) => i.id), true)">
            {{ t('uninstall.select_all') }}
          </button>
          <button class="text-fg-muted hover:underline" @click="setAll(fileItems.map((i) => i.id), false)">
            {{ t('uninstall.select_none') }}
          </button>
        </div>
      </div>
      <ul class="rounded border border-border divide-y divide-border">
        <li
          v-for="item in fileItems"
          :key="item.id"
          class="flex items-center gap-3 px-3 py-2 hover:bg-bg-subtle"
        >
          <input
            type="checkbox"
            :checked="selectedSet.has(item.id)"
            class="accent-accent w-4 h-4 shrink-0"
            @change="toggle(item.id)"
          />
          <component :is="kindIcon(item.kind)" class="w-4 h-4 shrink-0 text-fg-muted" />
          <div class="flex-1 min-w-0">
            <div class="text-sm text-fg truncate" :title="item.path">{{ item.path }}</div>
            <div class="text-xs text-fg-subtle truncate">{{ item.reason }}</div>
          </div>
          <span v-if="item.size_bytes" class="text-xs text-fg-muted shrink-0">
            {{ formatBytes(item.size_bytes) }}
          </span>
          <BaseBadge :variant="confidenceVariant(item.confidence)">
            {{ t(`uninstall.confidence.${item.confidence}`) }}
          </BaseBadge>
        </li>
      </ul>
    </section>

    <!-- Registry grubu -->
    <section v-if="regItems.length">
      <div class="flex items-center justify-between mb-1.5">
        <h4 class="text-sm font-semibold text-fg flex items-center gap-2">
          <ListTree class="w-4 h-4 text-fg-muted" />
          {{ t('uninstall.group_registry', { n: regItems.length }) }}
        </h4>
        <div class="flex gap-2 text-xs">
          <button class="text-accent hover:underline" @click="setAll(regItems.map((i) => i.id), true)">
            {{ t('uninstall.select_all') }}
          </button>
          <button class="text-fg-muted hover:underline" @click="setAll(regItems.map((i) => i.id), false)">
            {{ t('uninstall.select_none') }}
          </button>
        </div>
      </div>
      <ul class="rounded border border-border divide-y divide-border">
        <li
          v-for="item in regItems"
          :key="item.id"
          class="flex items-center gap-3 px-3 py-2 hover:bg-bg-subtle"
        >
          <input
            type="checkbox"
            :checked="selectedSet.has(item.id)"
            class="accent-accent w-4 h-4 shrink-0"
            @change="toggle(item.id)"
          />
          <KeyRound class="w-4 h-4 shrink-0 text-fg-muted" />
          <div class="flex-1 min-w-0">
            <div class="text-sm text-fg truncate font-mono" :title="item.path">
              {{ item.path }}<span v-if="item.value_name" class="text-fg-muted"> → {{ item.value_name }}</span>
            </div>
            <div class="text-xs text-fg-subtle truncate">{{ item.reason }}</div>
          </div>
          <BaseBadge :variant="confidenceVariant(item.confidence)">
            {{ t(`uninstall.confidence.${item.confidence}`) }}
          </BaseBadge>
        </li>
      </ul>
    </section>

    <p v-if="!items.length" class="text-sm text-fg-muted text-center py-6">
      {{ t('uninstall.no_leftovers') }}
    </p>
  </div>
</template>
