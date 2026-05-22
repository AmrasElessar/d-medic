export type ProfileKind = 'basic' | 'moderate' | 'aggressive' | 'custom';

export interface ProfileDefinition {
  kind: ProfileKind;
  label_tr: string;
  label_en: string;
  description_tr: string;
  description_en: string;
  included_action_ids: string[];
}
