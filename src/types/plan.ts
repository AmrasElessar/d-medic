import type { Finding } from './diagnostic';
import type { ProfileKind } from './profile';

export type PlanItemStatus =
  | 'pending'
  | 'running'
  | 'success'
  | 'failed'
  | 'skipped';

export interface PlanItem {
  finding_id: string;
  action_id: string;
  status: PlanItemStatus;
  started_at: string | null;
  finished_at: string | null;
  message: string | null;
}

export interface ExecutionPlan {
  plan_id: string;
  profile: ProfileKind;
  selected_findings: Finding[];
  items: PlanItem[];
  snapshot_id: string | null;
  reboot_required: boolean;
}
