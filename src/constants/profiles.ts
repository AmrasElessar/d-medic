import type { ProfileKind } from '@/types';

export interface ProfileMeta {
  kind: ProfileKind;
  labelKey: string;
  descKey: string;
  recommendedFor: string;
}

export const PROFILES: ProfileMeta[] = [
  {
    kind: 'basic',
    labelKey: 'profile.basic',
    descKey: 'profile.basic_desc',
    recommendedFor: '8 GB+ RAM, yeni nesil donanım',
  },
  {
    kind: 'moderate',
    labelKey: 'profile.moderate',
    descKey: 'profile.moderate_desc',
    recommendedFor: '4-8 GB RAM, çoğu sistem',
  },
  {
    kind: 'aggressive',
    labelKey: 'profile.aggressive',
    descKey: 'profile.aggressive_desc',
    recommendedFor: '2-4 GB RAM, eski sistem',
  },
  {
    kind: 'custom',
    labelKey: 'profile.custom',
    descKey: 'profile.custom_desc',
    recommendedFor: 'İleri kullanıcı',
  },
];
