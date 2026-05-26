// Backend `models::defrag` ile aynı sözleşme (serde snake_case).

export interface VolumeInfo {
  letter: string;
  file_system: string;
  media_type: string;
  total_bytes: number;
  free_bytes: number;
  cluster_bytes: number;
  total_clusters: number;
  free_clusters: number;
  defrag_supported: boolean;
}

export interface FileFrag {
  path: string;
  fragments: number;
  size_bytes: number;
}

export interface FragmentationReport {
  letter: string;
  fragmentation_percent: number;
  total_files: number;
  fragmented_files: number;
  most_fragmented: FileFrag[];
  elapsed_ms: number;
  recommendation: string;
}

export type CellState = 'free' | 'used' | 'fragmented' | 'unmovable' | 'moving';

export interface ClusterMap {
  letter: string;
  cols: number;
  rows: number;
  cells: CellState[];
  clusters_per_cell: number;
}

export type DefragMode = 'analyze_only' | 'quick' | 'full' | 'free_space_consolidate';

export interface DefragProgress {
  job_id: string;
  phase: string;
  current_file: string | null;
  clusters_moved: number;
  clusters_total: number;
  files_processed: number;
  percent: number;
}
