export type Priority = 'critical' | 'high' | 'medium' | 'low';

export type ActionType = 'automatic' | 'guided' | 'reboot' | 'not_possible';

export type Category =
  | 'performance'
  | 'stability'
  | 'security'
  | 'compatibility'
  | 'storage'
  | 'network'
  | 'power'
  | 'data';

export type RiskLevel = 'none' | 'low' | 'medium' | 'high';

export interface LocalizedText {
  tr: string;
  en: string;
}

export type EstimatedGain =
  | { kind: 'ram_mb'; value: number }
  | { kind: 'boot_pct'; value: number }
  | { kind: 'cpu_pct'; value: number }
  | { kind: 'disk_mb'; value: number }
  | { kind: 'stability' }
  | { kind: 'data_safety' }
  | { kind: 'none' };

export interface Finding {
  id: string;
  category: Category;
  priority: Priority;
  action_type: ActionType;
  title: LocalizedText;
  description: LocalizedText;
  estimated_gain: EstimatedGain;
  risk: RiskLevel;
  reboot_required: boolean;
  action_id: string | null;
  guide_id: string | null;
  evidence: unknown;
}

export type ScanKind = 'quick' | 'deep';

export interface ScanResult {
  scan_id: string;
  kind: ScanKind;
  started_at: string;
  finished_at: string;
  findings: Finding[];
}
