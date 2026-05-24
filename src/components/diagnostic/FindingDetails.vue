<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { FileText, BookOpen, ExternalLink } from 'lucide-vue-next';
import type { Finding } from '@/types';

interface Props {
  finding: Finding;
}
const props = defineProps<Props>();

defineEmits<{ (e: 'open-guide', id: string): void }>();

const { t, locale } = useI18n();

const fullDescription = computed(
  () => props.finding.description[locale.value as 'tr' | 'en'],
);

// Evidence içeriğini key-value (object) ve liste (array) olarak normalize et.
// Backend her zaman JSON object döner — string/number değerler i18n'e tabi
// değil (teknik veri), array'ler bullet liste olarak gösterilir.
interface EvidenceField {
  key: string;
  /** Render edilecek değer — string veya array */
  display: { kind: 'scalar'; value: string } | { kind: 'list'; items: string[] };
}

const evidenceFields = computed<EvidenceField[]>(() => {
  const ev = props.finding.evidence;
  if (!ev || typeof ev !== 'object') return [];

  const fields: EvidenceField[] = [];
  for (const [key, raw] of Object.entries(ev as Record<string, unknown>)) {
    if (Array.isArray(raw)) {
      // Array of strings/objects → bullet list
      const items = raw.map((v) => (typeof v === 'string' ? v : JSON.stringify(v)));
      fields.push({ key, display: { kind: 'list', items } });
    } else if (typeof raw === 'object' && raw !== null) {
      // Nested object → JSON string (nadir)
      fields.push({ key, display: { kind: 'scalar', value: JSON.stringify(raw) } });
    } else {
      // Primitive
      fields.push({ key, display: { kind: 'scalar', value: String(raw) } });
    }
  }
  return fields;
});

/** Evidence key'i için i18n etiketi — yoksa raw key (snake_case okunabilir). */
function evidenceLabel(key: string): string {
  const i18nKey = `evidence.${key}`;
  // @ts-expect-error vue-i18n typing
  if (t.te?.(i18nKey)) return t(i18nKey);
  // raw key'i okunabilir hale getir: total_installed → "total installed"
  return key.replace(/_/g, ' ');
}
</script>

<template>
  <div class="space-y-3 pt-2 mt-2 border-t border-border">
    <!-- Tam açıklama (kart üstünde truncated) -->
    <div>
      <h5 class="text-xs uppercase tracking-wider text-fg-subtle mb-1">
        {{ t('finding.description') }}
      </h5>
      <p class="text-sm text-fg leading-relaxed whitespace-pre-line">
        {{ fullDescription }}
      </p>
    </div>

    <!-- Evidence (varsa) -->
    <div v-if="evidenceFields.length">
      <h5 class="text-xs uppercase tracking-wider text-fg-subtle mb-1.5 flex items-center gap-1.5">
        <FileText class="w-3.5 h-3.5" />
        {{ t('finding.evidence') }}
      </h5>
      <dl class="text-sm space-y-1.5">
        <div
          v-for="field in evidenceFields"
          :key="field.key"
          class="grid grid-cols-[max-content_1fr] gap-x-3 gap-y-0.5 items-start"
        >
          <dt class="text-fg-muted text-xs uppercase tracking-wide whitespace-nowrap pt-0.5">
            {{ evidenceLabel(field.key) }}
          </dt>
          <dd v-if="field.display.kind === 'scalar'" class="text-fg font-mono text-xs break-all">
            {{ field.display.value }}
          </dd>
          <dd v-else class="col-start-1 col-span-2 mt-1">
            <ul class="text-xs font-mono text-fg list-disc list-inside space-y-0.5 max-h-48 overflow-y-auto bg-bg-subtle p-2 rounded">
              <li v-for="(item, i) in field.display.items" :key="i" class="break-all">{{ item }}</li>
            </ul>
          </dd>
        </div>
      </dl>
    </div>

    <!-- Kılavuz linki (varsa) -->
    <div v-if="finding.guide_id" class="pt-1">
      <button
        type="button"
        class="inline-flex items-center gap-1.5 text-xs text-accent hover:underline"
        @click="$emit('open-guide', finding.guide_id!)"
      >
        <BookOpen class="w-3.5 h-3.5" />
        {{ t('finding.open_guide') }}
        <ExternalLink class="w-3 h-3" />
      </button>
    </div>
  </div>
</template>
