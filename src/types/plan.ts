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

/** Tek action sonucu (backend ActionOutcome — error.rs JSON shape'i). */
export interface ActionOutcome {
  action_id: string;
  success: boolean;
  message: string;
  reboot_required: boolean;
  details: unknown | null;
}

/** apply_action dönüş tipi — outcome + öncesinde oluşturulan snapshot id. */
export interface ActionResult {
  snapshot_id: string | null;
  outcome: ActionOutcome;
}

/** apply_plan dönüş tipi — plan başına TEK snapshot + sıralı outcome listesi. */
export interface PlanResult {
  snapshot_id: string | null;
  outcomes: ActionOutcome[];
}
