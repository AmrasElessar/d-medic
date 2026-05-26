// Backend `models::uninstall` ile aynı sözleşme (serde snake_case).

export type ProgramKind = 'win32' | 'uwp';

export interface InstalledProgram {
  id: string;
  name: string;
  publisher: string | null;
  version: string | null;
  kind: ProgramKind;
  install_location: string | null;
  uninstall_string: string | null;
  quiet_uninstall_string: string | null;
  size_bytes: number | null;
  install_date: string | null;
  icon_base64: string | null;
  is_system_component: boolean;
}

export interface UninstallReport {
  program_id: string;
  launched: boolean;
  completed: boolean;
  exit_code: number | null;
  message: string;
}

export type LeftoverKind = 'file' | 'folder' | 'reg_key' | 'reg_value';
export type LeftoverConfidence = 'safe' | 'probable' | 'possible';

export interface LeftoverItem {
  id: string;
  kind: LeftoverKind;
  confidence: LeftoverConfidence;
  path: string;
  value_name: string | null;
  size_bytes: number | null;
  reason: string;
  default_selected: boolean;
}

export interface LeftoverScanResult {
  program_id: string;
  items: LeftoverItem[];
  scanned_roots: string[];
  registry_hits: number;
  file_hits: number;
}

export interface RemovalItemResult {
  item_id: string;
  path: string;
  success: boolean;
  message: string | null;
}

export interface RemovalReport {
  quarantine_id: string;
  removed: number;
  failed: number;
  freed_bytes: number;
  results: RemovalItemResult[];
}

export interface QuarantineEntry {
  id: string;
  created_at: string;
  program_label: string;
  file_count: number;
  reg_export_count: number;
  total_bytes: number;
  expires_in_days: number;
}
