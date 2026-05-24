<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { ExternalLink, AlertTriangle, Star } from 'lucide-vue-next';
import BaseModal from './BaseModal.vue';
import SourceBadge from './SourceBadge.vue';
import type { VerificationRecord, VerificationSource, SourceType, SourceStance } from '@/types';

interface Props {
  open: boolean;
  /** Aksiyon / check id'si (başlık için) */
  id?: string;
  record: VerificationRecord | null;
}
defineProps<Props>();
defineEmits<{ (e: 'close'): void }>();

const { t } = useI18n();

function typeLabel(type: SourceType): string {
  const map: Record<SourceType, string> = {
    microsoft: 'Microsoft',
    cis: 'CIS Benchmark',
    nist: 'NIST',
    nsa: 'NSA',
    disa: 'DISA STIG',
    github: 'GitHub',
    forum: t('verification.source_forum'),
    mvp: 'MVP / Tech Blog',
    other: t('verification.source_other'),
  };
  return map[type] ?? type;
}

function stanceLabel(stance: SourceStance): string {
  return t(`verification.stance_${stance}`);
}

function stanceColor(stance: SourceStance): string {
  switch (stance) {
    case 'recommends':
    case 'supports':
      return 'text-priority-low';
    case 'documents':
      return 'text-accent';
    case 'silent':
      return 'text-fg-muted';
    case 'discourages':
      return 'text-priority-medium';
    case 'blocks':
      return 'text-priority-critical';
  }
}

const grouped = computed(() => {
  // Aynı türden kaynakları grupla — UI'da daha okunur
  return (rec: VerificationRecord | null): Record<string, VerificationSource[]> => {
    if (!rec) return {};
    const map: Record<string, VerificationSource[]> = {};
    for (const s of rec.sources) {
      (map[s.type] ??= []).push(s);
    }
    return map;
  };
});
</script>

<template>
  <BaseModal
    :open="open"
    :title="t('verification.title') + (id ? ` — ${id}` : '')"
    size="lg"
    @close="$emit('close')"
  >
    <div v-if="record" class="space-y-4">
      <!-- Genel seviye + zarar kaydı -->
      <div class="flex items-start gap-3 flex-wrap">
        <SourceBadge :level="record.verification_level" />
        <span class="text-xs text-fg-muted">
          {{ t('verification.level_hint') }}
        </span>
      </div>

      <div
        v-if="record.harm_record"
        class="flex items-start gap-2 p-3 bg-priority-medium/10 border border-priority-medium/30 rounded text-sm"
      >
        <AlertTriangle class="w-4 h-4 text-priority-medium shrink-0 mt-0.5" />
        <div>
          <div class="font-semibold text-fg">{{ t('verification.harm_record') }}</div>
          <div class="text-fg-muted">{{ record.harm_record }}</div>
        </div>
      </div>

      <!-- Kaynak listesi -->
      <div class="space-y-3">
        <h3 class="text-xs uppercase tracking-wider text-fg-subtle">
          {{ t('verification.sources') }} ({{ record.sources.length }})
        </h3>

        <div
          v-for="(srcs, type) in grouped(record)"
          :key="type"
          class="space-y-1.5"
        >
          <div class="text-xs font-semibold text-fg-muted uppercase">
            {{ typeLabel(type as SourceType) }}
          </div>
          <div
            v-for="(src, i) in srcs"
            :key="i"
            class="border border-border rounded p-2.5 space-y-1.5"
          >
            <div class="flex items-start justify-between gap-2">
              <div class="flex items-center gap-2 text-sm">
                <span :class="['font-medium', stanceColor(src.stance)]">
                  {{ stanceLabel(src.stance) }}
                </span>
                <span
                  v-if="src.stars"
                  class="inline-flex items-center gap-0.5 text-xs text-fg-muted"
                >
                  <Star class="w-3 h-3" /> {{ src.stars.toLocaleString() }}
                </span>
              </div>
              <a
                v-if="src.url"
                :href="src.url"
                target="_blank"
                rel="noopener"
                class="inline-flex items-center gap-1 text-xs text-accent hover:underline shrink-0"
              >
                {{ t('verification.open') }}
                <ExternalLink class="w-3 h-3" />
              </a>
            </div>
            <div v-if="src.note" class="text-sm text-fg leading-relaxed">{{ src.note }}</div>
            <div v-if="src.reference" class="text-xs text-fg-muted font-mono">
              {{ src.reference }}
            </div>
            <div
              v-if="src.last_verified"
              class="text-[10px] text-fg-subtle"
            >
              {{ t('verification.last_verified') }}: {{ src.last_verified }}
            </div>
          </div>
        </div>
      </div>

      <div
        v-if="record.last_audit_date"
        class="text-xs text-fg-subtle pt-2 border-t border-border"
      >
        {{ t('verification.last_audit') }}: {{ record.last_audit_date }}
      </div>
    </div>

    <div v-else class="text-sm text-fg-muted text-center py-6">
      {{ t('verification.no_record') }}
    </div>
  </BaseModal>
</template>
