<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ChevronRight, ChevronDown } from 'lucide-vue-next';
import type { Finding } from '@/types';
import { useVerificationStore } from '@/stores/verification';
import { useGuidesStore } from '@/stores/guides';
import PriorityBadge from './PriorityBadge.vue';
import ActionTypeBadge from './ActionTypeBadge.vue';
import CategoryIcon from './CategoryIcon.vue';
import GainBadge from './GainBadge.vue';
import FindingDetails from './FindingDetails.vue';
import SourceBadge from '@/components/common/SourceBadge.vue';
import SourceList from '@/components/common/SourceList.vue';

interface Props {
  finding: Finding;
  selected?: boolean;
  selectable?: boolean;
}
const props = withDefaults(defineProps<Props>(), {
  selected: false,
  selectable: false,
});

defineEmits<{
  (e: 'toggle', id: string): void;
  (e: 'open', id: string): void;
}>();

const { locale } = useI18n();
const title = computed(() => props.finding.title[locale.value as 'tr' | 'en']);
const desc  = computed(() => props.finding.description[locale.value as 'tr' | 'en']);

// Verification rozeti: action_id > check_id öncelikli. Action önerisi varsa
// onun kaynakları daha alakalı; yoksa check'in kendi kaydı.
const verif = useVerificationStore();
const verifId = computed(() => props.finding.action_id ?? props.finding.id);
const verifRecord = computed(() => verif.byId(verifId.value));
const showSources = ref(false);

// "Detayları Göster" expand state. Evidence verisi veya guide_id varsa
// expander gösterilir; aksi halde tek satır kart yeterli.
const expanded = ref(false);
const hasEvidence = computed(() => {
  const ev = props.finding.evidence;
  return ev && typeof ev === 'object' && Object.keys(ev as object).length > 0;
});
const hasDetails = computed(() => hasEvidence.value || !!props.finding.guide_id);

const guidesStore = useGuidesStore();
function openGuide(id: string) {
  guidesStore.open(id);
}
</script>

<template>
  <article
    class="border rounded-lg p-4 transition-colors hover:border-border-strong"
    :class="selected ? 'border-accent bg-accent/5' : 'border-border bg-bg-subtle'"
  >
    <div class="flex items-start gap-3">
      <input
        v-if="selectable"
        type="checkbox"
        :checked="selected"
        class="mt-1 accent-accent"
        @change="$emit('toggle', finding.id)"
      />
      <div class="flex-1 min-w-0">
        <div class="flex items-center flex-wrap gap-2 mb-1">
          <CategoryIcon :category="finding.category" />
          <h4 class="text-sm font-medium text-fg truncate">{{ title }}</h4>
          <PriorityBadge :priority="finding.priority" />
          <ActionTypeBadge :action-type="finding.action_type" />
        </div>
        <p class="text-xs text-fg-muted line-clamp-2">{{ desc }}</p>
        <div class="flex items-center flex-wrap gap-3 mt-2">
          <GainBadge :gain="finding.estimated_gain" />
          <span
            v-if="finding.reboot_required"
            class="text-xs text-priority-high"
          >
            ⟳ Reboot
          </span>
          <SourceBadge
            v-if="verifRecord"
            :level="verifRecord.verification_level"
            @click="showSources = true"
          />
        </div>
      </div>
      <button
        v-if="hasDetails"
        class="text-fg-subtle hover:text-fg p-1"
        :title="expanded ? '' : ''"
        @click="expanded = !expanded"
      >
        <ChevronDown v-if="expanded" class="w-4 h-4" />
        <ChevronRight v-else class="w-4 h-4" />
      </button>
      <button
        v-else
        class="text-fg-subtle hover:text-fg p-1"
        @click="$emit('open', finding.id)"
      >
        <ChevronRight class="w-4 h-4" />
      </button>
    </div>

    <FindingDetails
      v-if="expanded"
      :finding="finding"
      @open-guide="openGuide"
    />

    <SourceList
      :open="showSources"
      :id="verifId"
      :record="verifRecord ?? null"
      @close="showSources = false"
    />
  </article>
</template>
