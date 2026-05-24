import type { LocalizedText } from './diagnostic';

export type GuideStepType = 'cmd' | 'bios' | 'info' | 'manual' | 'link';

export type GuideRisk = 'NONE' | 'LOW' | 'MEDIUM' | 'HIGH';

export interface GuideStep {
  id: number;
  type: GuideStepType;
  title: LocalizedText;
  body?: LocalizedText;
  command?: string;
  guide_link?: string;
  success_message?: string;
  fail_action?: 'stop' | 'warn' | 'continue';
}

/** Prerequisite eski (key) veya yeni (LocalizedText) formatta gelebilir. */
export type Prerequisite = string | LocalizedText;

export interface Guide {
  id: string;
  title: LocalizedText;
  priority: 'YÜKSEK' | 'ORTA' | 'DÜŞÜK' | 'KRİTİK' | string;
  estimated_time: string;
  risk: GuideRisk;
  risk_note?: LocalizedText;
  prerequisites: Prerequisite[];
  steps: GuideStep[];
  verification?: {
    command: string;
    success_pattern: string;
  };
  microsoft_doc?: string;
}
