export interface ServiceStateRecord {
  name: string;
  startup_type: string;
  status: string;
}

export interface Snapshot {
  id: string;
  timestamp: string;
  description: string;
  restore_point_created: boolean;
  registry_export_paths: string[];
  service_states: ServiceStateRecord[];
}

/** rollback_snapshot dönüş tipi. registry_imports[i] = (path, success). */
export interface RollbackReport {
  snapshot_id: string;
  registry_imports: [string, boolean][];
  services_restored: number;
}
